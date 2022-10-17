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
    Up(Up),

    /// Shut down minikube environment
    Down,
}

#[derive(Parser, Debug)]
pub struct Up {
    /// Don't deploy given helm charts and just start with the latest state
    #[arg(short, long)]
    pub no_deploy: bool,
}
