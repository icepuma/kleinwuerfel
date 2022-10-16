use std::{
    path::PathBuf,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

use anyhow::Ok;
use colored::Colorize;
use crossbeam_channel::{bounded, Receiver};

use crate::model::{Configuration, Helmchart};

#[derive(Debug, Clone)]
pub struct Kubectl {
    configuration: Configuration,
    kubectl_binary_path: PathBuf,
}

impl Kubectl {
    pub fn new(configuration: &Configuration, kubectl_binary_path: &PathBuf) -> Self {
        Kubectl {
            configuration: configuration.to_owned(),
            kubectl_binary_path: kubectl_binary_path.to_owned(),
        }
    }

    pub fn port_forward_all_helmcharts(&self) -> anyhow::Result<()> {
        if let Some(helmcharts) = &self.configuration.helmchart {
            println!();
            println!("{}", "Forwarding ports".bold().underline());
            println!("Press Ctrl+C to stop the port forwarding.");
            println!();

            self.port_forwarding(helmcharts)?;
        }

        Ok(())
    }

    fn port_forwarding(&self, helmcharts: &[Helmchart]) -> anyhow::Result<()> {
        let mut handles = vec![];

        let (sender, receiver) = bounded(0);

        let amount_of_receivers = helmcharts.len();

        ctrlc::set_handler(move || {
            for _ in 0..amount_of_receivers {
                match sender.send(()) {
                    std::result::Result::Ok(_) => {}
                    Err(err) => println!("{}", err),
                }
            }

            std::process::exit(0);
        })?;

        let shared_self = Arc::new(Mutex::new(self.clone()));

        for helmchart in helmcharts {
            let receiver = receiver.clone();

            match shared_self.lock() {
                std::result::Result::Ok(shared_self) => {
                    let self_clone = shared_self.clone();
                    let helmchart = helmchart.clone();

                    handles.push(thread::spawn(move || {
                        match self_clone.port_forward(&helmchart, &receiver) {
                            std::result::Result::Ok(_) => {}
                            Err(err) => println!("{}", err),
                        }
                    }));
                }
                Err(err) => println!("{}", err),
            }
        }

        for handle in handles {
            match handle.join() {
                std::result::Result::Ok(_) => {}
                Err(_) => println!("Cannot join child thread"),
            }
        }

        receiver.recv()?;

        Ok(())
    }

    fn port_forward(&self, helmchart: &Helmchart, receiver: &Receiver<()>) -> anyhow::Result<()> {
        if helmchart.ports.is_empty() {
            return Ok(());
        }

        // query namespace
        let namespace_output = Command::new(&self.kubectl_binary_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("get")
        .arg("namespaces")
        .arg("-o")
        .arg(format!("jsonpath={{.items[?(@.metadata.annotations.meta\\.helm\\.sh/release-name==\"{}\")].metadata.name}}", &helmchart.name))
        .spawn()?
        .wait_with_output()?;

        if let Some(namespace) = String::from_utf8(namespace_output.stdout)?
            .split_whitespace()
            .collect::<Vec<&str>>()
            .first()
        {
            // query service
            let service_output = Command::new(&self.kubectl_binary_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("--namespace")
            .arg(&namespace)
            .arg("get")
            .arg("service")
            .arg("-o")
            .arg(format!("jsonpath={{.items[?(@.metadata.annotations.meta\\.helm\\.sh/release-name==\"{}\")].metadata.name}}", &helmchart.name))
            .spawn()?
            .wait_with_output()?;

            if let Some(service) = String::from_utf8(service_output.stdout)?
                .split_whitespace()
                .collect::<Vec<&str>>()
                .first()
            {
                let mut arguments = vec![];
                arguments.push("port-forward".to_string());
                arguments.push("--namespace".to_string());
                arguments.push(namespace.to_string());
                arguments.push(format!("service/{}", service));

                for port in &helmchart.ports {
                    arguments.push(format!(":{}", port));
                }

                let mut child = Command::new(&self.kubectl_binary_path)
                    .args(&arguments)
                    .spawn()?;

                receiver.recv()?;

                child.kill()?;
            } else {
                println!("Cannot resolve service. No port-forward possible...")
            }
        } else {
            println!("Cannot resolve namespace. No port-forward possible...")
        }

        Ok(())
    }
}
