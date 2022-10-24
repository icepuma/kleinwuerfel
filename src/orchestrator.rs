use std::path::PathBuf;

use crate::{
    helm::Helm,
    kubectl::Kubectl,
    minikube::Minikube,
    model::{Configuration, HelmChartRepo, Helmchart},
};
use anyhow::Ok;
use colored::Colorize;
use url::Url;

pub struct Orchestrator {
    configuration: Configuration,
    helm_binary_path: PathBuf,
    minikube: Minikube,
    kubectl: Kubectl,
}

impl Orchestrator {
    pub fn new(
        configuration: &Configuration,
        minikube_binary_path: &PathBuf,
        helm_binary_path: &PathBuf,
        kubectl_binary_path: &PathBuf,
    ) -> Orchestrator {
        Orchestrator {
            configuration: configuration.to_owned(),
            helm_binary_path: helm_binary_path.to_owned(),
            minikube: Minikube::new(configuration, minikube_binary_path),
            kubectl: Kubectl::new(configuration, kubectl_binary_path),
        }
    }

    pub fn start(&self) -> anyhow::Result<()> {
        self.minikube.start()
    }

    pub fn cleanup(&self) -> anyhow::Result<()> {
        self.minikube.cleanup()
    }

    pub fn is_running(&self) -> anyhow::Result<bool> {
        self.minikube.is_running()
    }

    pub fn deploy(
        &self,
        helmchart: &Helmchart,
        helm_chart_repos: &[HelmChartRepo],
    ) -> anyhow::Result<()> {
        println!(
            "{}",
            format!("Deploy helm chart '{}'", &helmchart.name)
                .bold()
                .underline()
        );

        let helm_chart_repo = helm_chart_repos
            .iter()
            .find(|helm_chart_repo| helm_chart_repo.name == helmchart.helm_chart_repo)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Helm chart repo '{}' not specified in config file. Please provide one.",
                    helmchart.helm_chart_repo
                )
            })?;

        let helm = Helm::new(
            helm_chart_repo,
            &self.configuration.default_values,
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
            helm.add_repo(&helm_chart_repo.name, &helm_chart_repo.url)?;
            helm.upgrade(&helmchart.helm_chart_repo, helmchart)?;
        }

        println!();

        Ok(())
    }

    pub fn list_deployed_helmcharts(&self) -> anyhow::Result<()> {
        println!("{}", "Deployed helm charts".bold().underline());

        Helm::list()
    }

    pub fn port_forward_all_helmcharts(&self) -> anyhow::Result<()> {
        self.kubectl.port_forward_all_helmcharts()
    }
}
