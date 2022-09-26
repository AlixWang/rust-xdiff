use clap::Parser;
use xdiff::cli::Args;
fn main() {
    let args = Args::parse();
    println!("Hello, world! {:?}", args);
}
