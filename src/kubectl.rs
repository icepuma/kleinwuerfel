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

                    thread::spawn(
                        move || match self_clone.port_forward(&helmchart, &receiver) {
                            std::result::Result::Ok(_) => {}
                            Err(err) => println!("{}", err),
                        },
                    );
                }
                Err(err) => println!("{}", err),
            }
        }

        receiver.recv()?;

        Ok(())
    }

    fn resolve_namespace(&self, helmchart: &Helmchart) -> anyhow::Result<Option<String>> {
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
            return Ok(Some(namespace.to_string()));
        } else {
            let service_output = Command::new(&self.kubectl_binary_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("get")
            .arg("services")
            .arg("-o")
            .arg(format!("jsonpath={{.items[?(@.metadata.annotations.meta\\.helm\\.sh/release-name==\"{}\")].metadata.namespace}}", &helmchart.name))
            .spawn()?
            .wait_with_output()?;

            if let Some(namespace) = String::from_utf8(service_output.stdout)?
                .split_whitespace()
                .collect::<Vec<&str>>()
                .first()
            {
                return Ok(Some(namespace.to_string()));
            }
        }

        Ok(None)
    }

    fn port_forward(&self, helmchart: &Helmchart, receiver: &Receiver<()>) -> anyhow::Result<()> {
        if helmchart.ports.is_empty() {
            return Ok(());
        }

        if let Some(namespace) = self.resolve_namespace(helmchart)? {
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
                arguments.push(namespace);
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
