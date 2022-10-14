use std::path::PathBuf;

use crate::{
    helm::Helm,
    kubectl::Kubectl,
    minikube::Minikube,
    model::{Configuration, ContainerRegistry, HelmChartRepo, Helmchart},
};
use anyhow::Ok;
use colored::Colorize;
use lazy_static::lazy_static;
use regex::Regex;
use url::Url;

pub struct Orchestrator {
    configuration: Configuration,
    minikube_binary_path: PathBuf,
    helm_binary_path: PathBuf,
    kubectl_binary_path: PathBuf,
}

lazy_static! {
    static ref ENV_VAR_REGEX: Regex = Regex::new(r"\$\{env\.(?P<env_var>[a-zA-Z0-9_]+)\}").unwrap();
}

impl Orchestrator {
    pub fn new(
        configuration: &Configuration,
        minikube_binary_path: PathBuf,
        helm_binary_path: PathBuf,
        kubectl_binary_path: PathBuf,
    ) -> Orchestrator {
        Orchestrator {
            configuration: configuration.to_owned(),
            minikube_binary_path,
            helm_binary_path,
            kubectl_binary_path,
        }
    }

    pub fn start(&self) -> anyhow::Result<()> {
        Minikube::new(&self.configuration, &self.minikube_binary_path).start()
    }

    pub fn cleanup(&self) -> anyhow::Result<()> {
        Minikube::new(&self.configuration, &self.minikube_binary_path).cleanup()
    }

    pub fn is_running(&self) -> anyhow::Result<bool> {
        Minikube::new(&self.configuration, &self.minikube_binary_path).is_running()
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

    pub fn deploy(
        &self,
        helmchart: &Helmchart,
        container_registries: &[ContainerRegistry],
        helm_chart_repos: &[HelmChartRepo],
    ) -> anyhow::Result<()> {
        println!(
            "{}",
            format!("Deploy helm chart '{}'", &helmchart.name)
                .bold()
                .underline()
        );

        let container_registry = container_registries
            .iter()
            .find(|registry| registry.name == helmchart.container_registry)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Container registry '{}' not specified in config file. Please provide one.",
                    helmchart.container_registry
                )
            })?;

        let helm_chart_repo = helm_chart_repos
            .iter()
            .find(|helm_chart_repo| helm_chart_repo.name == helmchart.helm_chart_repo)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Helm chart repo '{}' not specified in config file. Please provide one.",
                    helmchart.container_registry
                )
            })?;

        let helm_chart_repo_username = self.replace_with_env_var(&helm_chart_repo.username)?;
        let helm_chart_repo_password = self.replace_with_env_var(&helm_chart_repo.password)?;

        let container_registry_username =
            self.replace_with_env_var(&container_registry.username)?;
        let container_registry_password =
            self.replace_with_env_var(&container_registry.password)?;

        let helm = Helm::new(
            helm_chart_repo_username,
            helm_chart_repo_password,
            container_registry_username,
            container_registry_password,
            &self.helm_binary_path,
        );

        // Login failed
        if !helm.login(&helm_chart_repo.url)? {
            let relogin_url = Url::parse(&helm_chart_repo.url)?;

            println!(
                r###"Cannot login to helm repo '{}'. Skip further deployment:

Your credentials might be wrong or the credentials rely on some OIDC mechanism and your session is expired.
e.g. Harbor in combination with OIDC providers forces you to relogin to have valid credentials.

Maybe {} is the URL where you can relogin.
"###,
                &helm_chart_repo.url,
                &format!(
                    "{}://{}",
                    &relogin_url.scheme(),
                    &relogin_url.host_str().unwrap_or_default()
                ),
            );
        } else {
            helm.add_repo(&helmchart.repo, &helm_chart_repo.url)?;
            helm.upgrade(&helmchart.repo, &helmchart.name)?;

            if !helmchart.ports.is_empty() {
                let _kubectl = Kubectl::new(&self.kubectl_binary_path);

                println!(
                    "Port forwarding is currently disabled, but will be part of a future release!"
                );
            }
        }

        println!();

        Ok(())
    }
}
