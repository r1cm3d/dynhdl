#![allow(unused)]

use std::process::exit;
use clap::Parser;
use serde_json::{Result, Value};
use log::{error, info};
use simple_logger::SimpleLogger;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{Client, Error};

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

const DEFAULT_REGION:&str = "sa-east-1";

#[tokio::main]
async fn main() {
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

    info!("Preparing to list all tables");
    let region_provider = RegionProviderChain::default_provider()
        .or_else(DEFAULT_REGION);
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    let resp_res = client.list_tables().send().await;
    if resp_res.is_err() {
        error!("Not possible to retrieve tables. Error {}.", resp_res.unwrap_err());
        exit(1)
    }
    let list_tables= resp_res.unwrap();
    let names = list_tables.table_names().unwrap_or_default();

    names.iter()
        .for_each(|t| info!("Table named ({}) retrieved.", t));
}
