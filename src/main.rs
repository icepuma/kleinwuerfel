use std::fs;

use clap::Parser;
use cli::{Options, SubCommand};

use crate::{model::Configuration, orchestrator::Orchestrator};

mod cli;
mod model;
mod orchestrator;

fn main() -> anyhow::Result<()> {
    let options = Options::parse();

    let content = fs::read_to_string(options.config)?;
    let configuration = toml::from_str::<Configuration>(&content)?;

    let orchestrator = Orchestrator::new(configuration.minikube);

    match options.subcommand {
        SubCommand::Start => {
            if let Ok(true) = orchestrator.is_running() {
                println!("Minikube is already running! Skip start sequence...");
            } else {
                orchestrator.start()?;
            }

            let registries = configuration.registry.unwrap_or_default();

            if let Some(helmcharts) = configuration.helmchart {
                for helm_chart in helmcharts {
                    orchestrator.deploy(&helm_chart, &registries)?;
                }
            }
        }
        SubCommand::Stop => {
            orchestrator.stop()?;
        }
    }

    Ok(())
}
