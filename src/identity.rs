use serde::{Serialize, Deserialize};
use bincode;

use crate::result::Result;


#[derive(Serialize, Deserialize, Debug)]
pub enum Identity {
    Worker
}


impl Into<Vec<u8>> for Identity {
    fn into(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}

impl From<Vec<u8>> for Identity {

    fn from(i: Vec<u8>) -> Self {
        bincode::deserialize(&i[..]).unwrap()
    }
}

