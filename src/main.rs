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
use crate::DynHdlErr::{ParsingErr, PKNotFoundErr, GetItemErr};

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
    match exec().await {
        Ok(_) => {
            exit(exitcode::OK);
        }
        Err(e) => {
            error!("{}", e);
            match e {
                DynHdlErr::ParsingErr { item, err_msg } => exit(exitcode::USAGE),
                _ => exit(exitcode::USAGE)
            }
        }
    }
}

async fn exec() -> Result<(), DynHdlErr> {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();
    let cli = Cli::parse();

    let table = cli.table;
    let item = cli.item;
    let pk_name = cli.pk;
    info!("Arguments have successfully parsed into Table ({}), Partition \
    Key ({}) and Item ({})", table, pk_name, item);

    info!("Parsing Item ({}) JSON", item);
    let parse_res = serde_json::from_str(&item);
    if parse_res.is_err() {
        //error!("Not possible to parse item. Error: {}.", parse_res.unwrap_err());
        let err = parse_res.unwrap_err();
        return Err(ParsingErr { item, err_msg: err.to_string()});
    }
    let item: Value = parse_res.unwrap();
    info!("Item JSON parsed into ({})", item);

    info!("Retrieving Partition Key ({}) of Item ({}).", pk_name, item);
    let pk = &item[&pk_name];
    if pk.is_null() {
        // FIXME: Compilation error.
        return Err(PKNotFoundErr { pk_name, item: item. });
        // return Err(Error::new(format!("Cannot find Partition Key ({}) of \
         //                              Item ({}). Exiting.", pk_name, item)));
    }
    info!("Partition Key ({}) has successfully retrieved of Item ({}).", pk, item);

    let region_provider = RegionProviderChain::default_provider()
        .or_else(DEFAULT_REGION);
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    let get_item_res = get_item(&client, &table, &pk_name,
                                &pk.to_string())
        .send()
        .await;
    if get_item_res.is_err() {
        // error!("Not possible to retrieve item with Partition Key ({}) of Table ({}).\
        //Error: {}", pk_name, table, get_item_res.unwrap_err());
        let err = get_item_res.unwrap_err();

        // FIXME: Compilation error.
        return Err(GetItemErr { pk: pk.to_string(), table, err_msg: err.to_string()});
        //return parse_res.map_err(|err| Error::new(format!("Not possible to retrieve item with Partition Key ({}) of Table ({}).\
       //Error: {}", pk_name, table, err.unwrap_err())));
    }
    let query = get_item_res.unwrap();
    if query.count == 0 {
        info!("No item found in Table ({}) with Partition Key ({}). Creating a new one with PutItem request.", table, pk)
    } else {
        let items = query.items.unwrap();
        let i = items.iter().next().unwrap();
        info!("Item ({:?}) found in Table ({}) with Partition Key ({}).", i, table, pk_name)
    }

    Ok(())
}

fn get_item(client: &Client, table_name: &str, pk_name: &str, pk: &str) -> Query {
    // FIXME: Checking why pk has double quotes and remove it if necessary.
    debug!("Querying by {}", pk.to_string());
    client
        .query()
        .table_name(table_name)
        .key_condition_expression("#pk = :pk")
        .expression_attribute_names("#pk", pk_name.to_string())
        .expression_attribute_values(":pk", AttributeValue::S(pk.to_string()))
}

