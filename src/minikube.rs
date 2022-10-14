use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::model::Configuration;

pub struct Minikube {
    configuration: Configuration,
    minikube_binary_path: PathBuf,
}

impl Minikube {
    pub fn new(configuration: &Configuration, minikube_binary_path: &PathBuf) -> Self {
        Minikube {
            configuration: configuration.clone(),
            minikube_binary_path: minikube_binary_path.to_owned(),
        }
    }

    pub fn start(&self) -> anyhow::Result<()> {
        let mut arguments: Vec<String> = vec![];
        arguments.push("start".to_string());

        if let Some(minikube) = &self.configuration.minikube {
            let cpus = minikube
                .cpus
                .map(|cpu| cpu.to_string())
                .unwrap_or_else(|| "4".to_string());
            let memory = minikube
                .memory
                .map(|memory| memory.to_string())
                .unwrap_or_else(|| "8192".to_string());

            arguments.push("--cpus".to_string());
            arguments.push(cpus);
            arguments.push("--memory".to_string());
            arguments.push(memory);
        } else {
            arguments.push("--cpus".to_string());
            arguments.push("4".to_string());
            arguments.push("--memory".to_string());
            arguments.push("8192".to_string());
        }

        Command::new(&self.minikube_binary_path)
            .args(&arguments)
            .spawn()?
            .wait()?;

        if let Some(minikube) = &self.configuration.minikube {
            for addon in &minikube.addons {
                Command::new(&self.minikube_binary_path)
                    .arg("addons")
                    .arg("enable")
                    .arg(addon)
                    .spawn()?
                    .wait()?;
            }
        }

        Ok(())
    }

    pub fn cleanup(&self) -> anyhow::Result<()> {
        Command::new(&self.minikube_binary_path)
            .arg("delete")
            .spawn()?
            .wait()?;

        Ok(())
    }

    pub fn is_running(&self) -> anyhow::Result<bool> {
        let output = Command::new(&self.minikube_binary_path)
            .arg("status")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?
            .wait_with_output()?;

        Ok(output.status.success())
    }
}
