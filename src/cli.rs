use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "injm", about = "Inject content into marked region", version)]
pub struct Cli {
    #[arg(short, long)]
    pub output: PathBuf,
}
