#![allow(unused)]

use std::process::exit;
use clap::Parser;
use serde_json::{Result, Value};
use log::{error, info};
use simple_logger::SimpleLogger;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long)]
    table: String,

    #[clap(short, long)]
    item: String,

    #[clap(long)]
    pk: String,
}

fn main() {
    SimpleLogger::new().init().unwrap();
    let cli = Cli::parse();

    let table = cli.table;
    let item = cli.item;
    let pk = cli.pk;
    info!("Arguments have successfully parsed into Table ({}), Partition \
    Key ({}) and Item ({})", table, pk, item);

    info!("Parsing Item ({}) JSON", item);
    let parse_res = serde_json::from_str(&item);
    if parse_res.is_err() {
        error!("Not possible to parse item. Error: {}.", parse_res.unwrap_err());
        exit(1);
    }
    let item: Value = parse_res.unwrap();
    info!("Item JSON parsed into ({})", item);
}
