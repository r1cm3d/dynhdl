use std::error::Error;
use std::fmt::{Formatter, Result, Display};

#[derive(Debug)]
pub enum DynHdlErr {
    Parsing { item: String, err_msg: String },
    PKNotFound { pk_name: String, item: String },
    GetItem { pk_name: String, pk: String, table: String, err_msg: String },
    TooManyRecords { pk_name: String, pk: String, table: String },
}

impl Error for DynHdlErr {}

impl Display for DynHdlErr {
    fn fmt(&self, f: &mut Formatter)
           -> Result {
        match self {
            DynHdlErr::Parsing { item, err_msg } => {
                write!(f, "a parsing error occurred for Item ({}). Error {}", item, err_msg)
            }
            DynHdlErr::PKNotFound { pk_name, item } => {
                write!(f, "cannot find Partition Key ({}) of Item ({}). Exiting.", pk_name, item)
            }
            DynHdlErr::GetItem { pk_name, pk, table, err_msg } => {
                write!(f, "not possible to retrieve item with Partition Key ({}) named ({}) of Table ({}). Error: {}", pk, pk_name, table, err_msg)
            }
            DynHdlErr::TooManyRecords { pk_name, pk, table } => {
                write!(f, "Partition Key ({}) named ({}) returns more than one record for Table ({}).", pk_name, pk, table)
            }
        }
    }
}