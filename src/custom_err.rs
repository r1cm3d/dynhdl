use std::error::Error;
use std::fmt::{Formatter, Result, Display};

#[derive(Debug)]
pub enum DynHdlErr {
    Parsing { item: String, err_msg: String },
    PKNotFound { pk_name: String, item: String },
    GetItem { pk: String, table: String, err_msg: String },
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
            DynHdlErr::GetItem { pk, table, err_msg } => {
                write!(f, "not possible to retrieve item with Partition Key ({}) of Table ({}). Error: {}", pk, table, err_msg)
            }
        }
    }
}