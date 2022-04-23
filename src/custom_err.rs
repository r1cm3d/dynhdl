use std::error::Error;
use std::fmt::{Formatter, Result, Display};

#[derive(Debug)]
pub enum DynHdlErr {
    ParsingErr { item: String, err_msg: String },
    PKNotFoundErr { pk_name: String, item: String },
    GetItemErr { pk: String, table: String, err_msg: String },
}

impl Error for DynHdlErr {}

impl Display for DynHdlErr {
    fn fmt(&self, f: &mut Formatter)
           -> Result {
        match self {
            // TODO: Implement it
            DynHdlErr::ParsingErr { item, err_msg } => write!(f, "unknown error with code {}.", item),
            DynHdlErr::PKNotFoundErr { pk_name, item } => {
                write!(f, "Sit by a lake")
            },
            DynHdlErr::GetItemErr { pk, table, err_msg } => {
                write!(f, format!("not possible to retrieve item with Partition Key ({}) of Table ({}). Error: {}", pk, table, err_msg))
            }
        }
    }
}