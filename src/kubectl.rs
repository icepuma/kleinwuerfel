use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::model::Helmchart;

pub struct Kubectl {
    kubectl_binary_path: PathBuf,
}

impl Kubectl {
    pub fn new(kubectl_binary_path: &PathBuf) -> Self {
        Kubectl {
            kubectl_binary_path: kubectl_binary_path.to_owned(),
        }
    }

    #[allow(dead_code)]
    fn port_forward(&self, helmchart: &Helmchart) -> anyhow::Result<()> {
        if helmchart.ports.is_empty() {
            return Ok(());
        }

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
                Command::new(&self.kubectl_binary_path)
                    .arg("port-forward")
                    .arg("--namespace")
                    .arg(&namespace)
                    .arg(format!("service/{}", service))
                    .arg(
                        helmchart
                            .ports
                            .iter()
                            .map(|port| format!("{}", port))
                            .collect::<Vec<String>>()
                            .join(" "),
                    )
                    .spawn()?;
            } else {
                println!("Cannot resolve service. No port-forward possible...")
            }
        } else {
            println!("Cannot resolve namespace. No port-forward possible...")
        }

        Ok(())
    }
}
