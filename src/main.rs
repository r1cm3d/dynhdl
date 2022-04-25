#![allow(unused)]

mod custom_err;

use std::borrow::Borrow;
use std::fmt::{Display, Error, Formatter};
use std::result::Result;
use std::process::exit;
use std::collections::HashMap;
use clap::Parser;
use serde_json::{Result as SerdeResult, Value};
use log::{debug, error, info, LevelFilter};
use simple_logger::SimpleLogger;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{Client, Error as DynError};
use aws_sdk_dynamodb::client::fluent_builders::Query;
use aws_sdk_dynamodb::model::{AttributeValue, Get};
use custom_err::DynHdlErr::{Parsing, PKNotFound, GetItem, TooManyRecords};
use custom_err::DynHdlErr;

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
    let raw_item = cli.item;
    let pk_name = cli.pk;
    info!("Arguments have successfully parsed into Table ({}), Partition \
    Key ({}) and Item ({})", table, pk_name, raw_item);

    info!("Parsing Item ({}) JSON", raw_item);
    let item: Result<HashMap<String, Value>, serde_json::Error> = serde_json::from_str(&raw_item);
    let item = match item {
        Ok(item) => item,
        Err(err) => return Err(Parsing { item: raw_item.to_string(), err_msg: err.to_string() })
    };
    info!("Item JSON parsed into ({})", raw_item);

    info!("Retrieving Partition Key ({}) of Item ({}).", pk_name, raw_item);
    let pk = &item.get(&pk_name);
    let pk = match pk {
        Some(pk) => pk,
        None => return Err(PKNotFound { pk_name, item: raw_item.to_string() })
    };
    info!("Partition Key ({}) has successfully retrieved of Item ({}).", pk, raw_item);

    // TODO: group AWS stuff into its own module.
    let region_provider = RegionProviderChain::default_provider()
        .or_else(DEFAULT_REGION);
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    let query = get_item(&client, &table, &pk_name,
                         pk)
        .send()
        .await;
    let query = match query {
        Ok(query) => query,
        Err(err) => return Err(GetItem { pk_name: pk_name.to_string(), pk: pk.to_string(), table: table.to_string(), err_msg: err.to_string() })
    };

    match query.count {
        0 => {
            info!("No item found in Table ({}) with Partition Key ({}). Creating a new one with PutItem request.", table, pk);
            add_item(&client, &table, item);
        }
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

// FIXME: this function is buggy.
async fn add_item(client: &Client, table: &str, v: HashMap<String, Value>) -> Result<(), Error> {
    //let user_av = AttributeValue::S(username.into());
    //let type_av = AttributeValue::S(p_type.into());
    //let age_av = AttributeValue::S(age.into());
    //let first_av = AttributeValue::S(first.into());
    //let last_av = AttributeValue::S(last.into());

    let request = client.put_item().table_name(table);

    // TODO: Test it
    // - Add issue to implement tree of items recursively.
    let request = v.iter()
        .map(|x| {
            let (k, v) = x;
            return if v.is_number() {
                (k, AttributeValue::N(v.to_string()))
            } else {
                (k, AttributeValue::S(v.as_str().unwrap().to_string()))
            };
        })
        .map(|x| {
            let (k, v) = x;
            return request.borrow().clone().item(k, v).clone();
        }).last().unwrap();

    //let request = client
    //    .put_item()
    //    .table_name(table)
    //    .item("username", user_av)
    //    .item("account_type", type_av)
    //    .item("age", age_av)
    //    .item("first_name", first_av)
    //    .item("last_name", last_av);

    // FIXME: change it after make request work.
    request.send().await.expect("err");

    Ok(())
}

