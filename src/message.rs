use std::net::TcpStream;
use std::convert::TryFrom;
use std::io::prelude::*;
use serde::{Serialize, Deserialize};
use bincode;
use crate::task::Task;
use crate::traits::{WriteObject, ReadObject};


#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    RunTask(Task),
    IsReady,
    ReadyForTask,
    TaskInProgress,
    Invalid,
}

impl Into<Vec<u8>> for Message {
    fn into(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}


impl From<Vec<u8>> for Message {
    fn from(m: Vec<u8>) -> Self {
        bincode::deserialize(&m[..]).unwrap() 
    }
}


impl<W> WriteObject<W> for TcpStream where
    W: Serialize + Into<Vec<u8>> { 
    fn write_object(&mut self, write: W) -> Result<usize, std::io::Error> 
    {
        let encoded_message: Vec<u8> = write.into();
        self.write(&encoded_message)
    }
}

impl<'a, R> ReadObject<R> for TcpStream where
    R: Deserialize<'a> + From<Vec<u8>> {
    fn read_object(&mut self) -> Option<R>
    {
        let mut buffer = [0; 1024];
        self.read(&mut buffer).unwrap(); 
        let as_vec: Vec<u8> = buffer.into();
        if let Ok(r) = R::try_from(as_vec) {
            Some(r)
        }
        else {
            None
        }
    }
}

