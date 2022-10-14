use std::fs;

use clap::Parser;
use cli::{Options, SubCommand};
use colored::Colorize;

use crate::{helm::Helm, model::Configuration, orchestrator::Orchestrator};

mod cli;
mod helm;
mod model;
mod orchestrator;

fn main() -> anyhow::Result<()> {
    let options = Options::parse();

    let content = fs::read_to_string(options.config).map_err(|_| {
        anyhow::anyhow!(
            "Can't read config file. Please provide a proper location, like ./kleinwuerfel.toml"
        )
    })?;

    let configuration = toml::from_str::<Configuration>(&content)?;

    let orchestrator = Orchestrator::new(&configuration);

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
