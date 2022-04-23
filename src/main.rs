#![allow(unused)]

use clap::Parser;
use serde_json::{Result, Value};

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
    let table = cli.table;
    let item = cli.item;
    let v: Value = serde_json::from_str(&item).expect("Item is not a valid JSON.");

    println!("Item: {:?}, Numeric: {:?}", v["item"], v["numeric_item"]);
}
