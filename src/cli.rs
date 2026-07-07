use clap::Parser;
use std::path::PathBuf;

use crate::marker::MarkerID;

#[derive(Parser)]
#[command(
    name = "injm",
    about = "Inject stdin content into marked regions between `injm begin` and `injm end` comments.",
    version
)]
pub struct Cli {
    #[arg(short, long)]
    pub output: PathBuf,

    #[arg(long)]
    pub dry_run: bool,

    #[arg(long)]
    pub id: Vec<MarkerID>,
}
