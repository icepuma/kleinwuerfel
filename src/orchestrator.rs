use std::process::{Command, Stdio};

use crate::{
    helm::Helm,
    model::{Configuration, Helmchart, Minikube, Registry},
};
use anyhow::Ok;
use lazy_static::lazy_static;
use regex::Regex;
use which::which;

pub struct Orchestrator {
    minikube: Option<Minikube>,
}

lazy_static! {
    static ref ENV_VAR_REGEX: Regex = Regex::new(r"\$\{env\.(?P<env_var>[a-zA-Z0-9_]+)\}").unwrap();
}

impl Orchestrator {
    pub fn new(configuration: &Configuration) -> Orchestrator {
        Orchestrator {
            minikube: configuration.minikube.as_ref().cloned(),
        }
    }

    pub fn start(&self) -> anyhow::Result<()> {
        let minikube = which("minikube")?;

        let mut arguments: Vec<String> = vec![];
        arguments.push("start".to_string());

        if let Some(minikube) = &self.minikube {
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

        Command::new(&minikube).args(&arguments).spawn()?.wait()?;

        if let Some(minikube_config) = &self.minikube {
            for addon in &minikube_config.addons {
                Command::new(&minikube)
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
        let minikube = which("minikube")?;

        Command::new(minikube).arg("delete").spawn()?.wait()?;

        Ok(())
    }

    pub fn is_running(&self) -> anyhow::Result<bool> {
        let minikube = which("minikube")?;
        let status = Command::new(minikube)
            .arg("status")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?
            .wait_with_output()?;

        Ok(status.status.success())
    }

    fn extract_env_var_name(&self, input: &str) -> anyhow::Result<String> {
        let env_var_name = ENV_VAR_REGEX
            .captures(input)
            .and_then(|capture| {
                capture
                    .name("env_var")
                    .map(|env_var| env_var.as_str().to_string())
            })
            .ok_or_else(|| {
                anyhow::anyhow!("Reading env var name from input='{}' didn't work.", input)
            })?;

        Ok(env_var_name)
    }

    fn replace_with_env_var(&self, input: &str) -> anyhow::Result<String> {
        let env_var_name = self.extract_env_var_name(input)?;

        let env_var = std::env::var(&env_var_name).map_err(|_| {
            anyhow::anyhow!(
                "The env var '{}' is not defined, please provide it via 'export {}=\"<your-desired-value>\"'",
                &env_var_name,
                &env_var_name
            )
        })?;

        Ok(env_var)
    }

    pub fn deploy(&self, helmchart: &Helmchart, registries: &[Registry]) -> anyhow::Result<()> {
        let registry = registries
            .iter()
            .find(|registry| registry.name == helmchart.registry)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Registry '{}' not specified in config file. Please provide one.",
                    helmchart.registry
                )
            })?;

        let username = self.replace_with_env_var(&registry.username)?;
        let password = self.replace_with_env_var(&registry.password)?;

        let helm = Helm::new(username, password);

        helm.login(&registry.helm_repo_url)?; // TODO: login only once per registry
        helm.add_repo(&helmchart.repo, &registry.helm_repo_url)?;
        helm.upgrade(&helmchart.repo, &helmchart.name)?;

        if !helmchart.ports.is_empty() {
            //self.port_forward(&helmchart)?;
            println!(
                "Port forwarding is currently disabled, but will be part of a future release!"
            );
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn port_forward(&self, helmchart: &Helmchart) -> anyhow::Result<()> {
        if helmchart.ports.is_empty() {
            return Ok(());
        }

        let kubectl = which("kubectl")?;

        let namespace_output = Command::new(&kubectl)
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
            let service_output = Command::new(&kubectl)
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
                Command::new(&kubectl)
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
