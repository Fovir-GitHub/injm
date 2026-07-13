use clap::Parser;

use crate::core::types::OutputID;

#[derive(Parser)]
#[command(
    name = "injm",
    about = "Inject stdin content into marked regions between `injm begin` and `injm end` comments.",
    version
)]
pub struct Cli {
    #[arg(short, long, num_args = 1..)]
    pub input: Vec<String>,

    #[arg(short, long, num_args = 1..)]
    pub output: Vec<String>,

    #[arg(long)]
    pub dry_run: bool,

    #[arg(long)]
    pub id: Vec<OutputID>,
}
