use std::fs;

use clap::Parser;
use cli::{Options, SubCommand};
use colored::Colorize;
use which::which;

use crate::{helm::Helm, model::Configuration, orchestrator::Orchestrator};

mod cli;
mod helm;
mod kubectl;
mod minikube;
mod model;
mod orchestrator;

fn main() -> anyhow::Result<()> {
    let options = Options::parse();

    let content = fs::read_to_string(options.config).map_err(|_| {
        anyhow::anyhow!(
            "Can't read config file. Please provide a proper location, like ./kleinwuerfel.toml"
        )
    })?;

    let minikube_binary_path = which("minikube").map_err(|_| {
        anyhow::anyhow!("The binary 'minikube' is missing in your $PATH. Installation guide: https://minikube.sigs.k8s.io/docs/start/")
    })?;

    let helm_binary_path = which("helm").map_err(|_| {
        anyhow::anyhow!("The binary 'helm' is missing in your $PATH. Installation guide: https://helm.sh/docs/intro/install/")
    })?;

    let kubectl_binary_path = which("kubectl").map_err(|_| {
        anyhow::anyhow!("The binary 'kubectl' is missing in your $PATH. Installation guide: https://kubernetes.io/docs/tasks/tools/")
    })?;

    let configuration = toml::from_str::<Configuration>(&content)?;

    let orchestrator = Orchestrator::new(
        &configuration,
        minikube_binary_path,
        helm_binary_path,
        kubectl_binary_path,
    );

    match options.subcommand {
        SubCommand::Start => {
            println!("{}", "Bootstrap minikube".bold().underline());

            if let Ok(true) = orchestrator.is_running() {
                println!("Minikube is already running! Skip start sequence...");
            } else {
                orchestrator.start()?;
            }

            println!();

            let registries = configuration.registry.unwrap_or_default();

            if let Some(helmcharts) = configuration.helmchart {
                for helm_chart in helmcharts {
                    orchestrator.deploy(&helm_chart, &registries)?;
                }
            }

            println!("{}", "Deployed helm charts".bold().underline());

            Helm::list()?;
        }
        SubCommand::Cleanup => {
            orchestrator.cleanup()?;
        }
    }

    Ok(())
}
