use serde::{Serialize, Deserialize};
use bincode;

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    name: String,
    command: String
}

impl Into<Vec<u8>> for Task {
    fn into(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}


impl From<Vec<u8>> for Task {

    fn from(f: Vec<u8>) -> Task {
        bincode::deserialize(&f[..]).unwrap()
    }
}

