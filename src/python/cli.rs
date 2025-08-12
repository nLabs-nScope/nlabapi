use clap::{Parser, Subcommand, Args};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub(super) struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub(super) enum Commands {
    /// Update all detected nLabs
    Update(UpdateArgs),
}

#[derive(Args, Debug)]
pub(super) struct UpdateArgs {
    #[arg(long = "force-downgrade", help = "Force nLab to downgrade firmware to match nlabapi")]
    pub force_downgrade: bool,
}