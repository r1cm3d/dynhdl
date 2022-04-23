#![allow(unused)]

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long)]
    table: String,

    #[clap(short, long)]
    item: String,
}

fn main() {
    let cli = Cli::parse();
    println!("{:?}", cli);
}
