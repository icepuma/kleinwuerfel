use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, about, version)]
pub struct Options {
    #[arg(short, long, default_value = "./kleinwuerfel.toml")]
    pub config: String,

    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    /// Spin up the minikube environment and deploy the given set of helmcharts
    Up,

    /// Shut down minikube environment
    Down,
}
