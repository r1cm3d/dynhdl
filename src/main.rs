#![allow(unused)]

mod custom_err;

use std::fmt::{Display, Error, Formatter};
use std::result::Result;
use std::process::exit;
use clap::Parser;
use serde_json::{Result as SerdeResult, Value};
use log::{debug, error, info, LevelFilter};
use simple_logger::SimpleLogger;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{Client, Error as DynError};
use aws_sdk_dynamodb::client::fluent_builders::Query;
use aws_sdk_dynamodb::model::{AttributeValue, Get};
use custom_err::DynHdlErr;
use crate::DynHdlErr::{Parsing, PKNotFound, GetItem};

const DEFAULT_REGION: &str = "sa-east-1";

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

#[tokio::main]
async fn main() -> () {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();
    let cli = Cli::parse();

    match exec(cli).await {
        Ok(_) => {
            exit(exitcode::OK);
        }
        Err(e) => {
            error!("Error: {}", e);
            match e {
                DynHdlErr::Parsing { item, err_msg } => exit(exitcode::DATAERR),
                DynHdlErr::PKNotFound { pk_name, item } => exit(exitcode::DATAERR),
                DynHdlErr::GetItem { pk_name, pk, table, err_msg } => exit(exitcode::IOERR),
                DynHdlErr::TooManyRecords { pk_name, pk, table } => exit(exitcode::USAGE),
            }
        }
    }
}

async fn exec(cli: Cli) -> Result<(), DynHdlErr> {
    let table = cli.table;
    let item = cli.item;
    let pk_name = cli.pk;
    info!("Arguments have successfully parsed into Table ({}), Partition \
    Key ({}) and Item ({})", table, pk_name, item);

    info!("Parsing Item ({}) JSON", item);
    let parse_res = serde_json::from_str(&item);
    if parse_res.is_err() {
        let err = parse_res.unwrap_err();
        return Err(Parsing { item: item.to_string(), err_msg: err.to_string() });
    }
    let item: Value = parse_res.unwrap();
    info!("Item JSON parsed into ({})", item);

    info!("Retrieving Partition Key ({}) of Item ({}).", pk_name, item);
    let pk = &item[&pk_name];
    if pk.is_null() {
        return Err(PKNotFound { pk_name, item: item.to_string() });
    }
    info!("Partition Key ({}) has successfully retrieved of Item ({}).", pk, item);

    let region_provider = RegionProviderChain::default_provider()
        .or_else(DEFAULT_REGION);
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    let get_item_res = get_item(&client, &table, &pk_name,
                                pk)
        .send()
        .await;
    if get_item_res.is_err() {
        let err = get_item_res.unwrap_err();

        return Err(GetItem { pk_name, pk: pk.to_string(), table: table.to_string(), err_msg: err.to_string() });
    }
    let query = get_item_res.unwrap();
    match query.count {
        0 => info!("No item found in Table ({}) with Partition Key ({}). Creating a new one with PutItem request.", table, pk),
        1 => {
            let items = query.items.unwrap();
            let i = items.iter().next().unwrap();
            info!("Item ({:?}) found in Table ({}) with Partition Key ({}).", i, table, pk_name)
        }
        _ => {
            return Err(DynHdlErr::TooManyRecords { pk_name, pk: pk.to_string(), table });
        }
    }

    Ok(())
}

fn get_item(client: &Client, table: &str, pk_name: &str, pk: &Value) -> Query {
    info!("Querying by key ({}) as PK ({}) in Table ({}).", pk, pk_name, table);
    let pk = match pk.is_number() {
        true => AttributeValue::N(pk.to_string()),
        false => AttributeValue::S(pk.as_str().unwrap().to_string()),
    };

    client
        .query()
        .table_name(table)
        .key_condition_expression("#pk = :pk")
        .expression_attribute_names("#pk", pk_name.to_string())
        .expression_attribute_values(":pk", pk)
}

