mod cli;
mod detector;

fn main() {
    let cli = cli::Cli::parse();
    println!("Hello, world!")
}
