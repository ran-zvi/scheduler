use std::io::{Read, Write, Error, ErrorKind, BufReader, BufRead};
use std::io;
use std::fmt::Debug;
use serde::{
    Serialize, Deserialize,
    de::DeserializeOwned
};
use crate::message::{Message, HandShake};


pub trait WriteObject<O> {
    fn write_object(&mut self, write: O) -> Result<usize, Error>;
}

pub trait ReadObject<O> {
    fn read_object(&mut self) -> Option<O>;
}

pub trait SyncRead<O> {
    fn sync_read(&mut self) -> Result<O, Error>; 
}

pub trait SyncWrite<O> {
    fn sync_write(&mut self, write: O) -> Result<usize, Error>; 
}

impl<O: Serialize + Debug, T: Write> WriteObject<O> for T {
    fn write_object(&mut self, write: O) -> Result<usize, Error> {
        let bin = bincode::serialize(&write).unwrap();
        self.write(&bin)
    }
}

impl<O: DeserializeOwned + std::fmt::Debug, T: Read> ReadObject<O> for T {
    fn read_object(&mut self) -> Option<O> {
        let mut buffer = [0u8; 1024];
        let mut bytes_read = 0;
        let mut timeout = 10000;
        loop {
            match self.read(&mut buffer) {
                Ok(n) if n > 0 => return Some(bincode::deserialize(&buffer).unwrap()),
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => (),
                Ok(_) => (),
                Err(_) => eprintln!("read_object failed!")
            }
            timeout -= 1;
        }
        None
    }
}

impl<'a ,T, O> SyncRead<O> for T where
    O: Deserialize<'a>,
    T: ReadObject<HandShake> + WriteObject<HandShake> + ReadObject<O> + WriteObject<O>
    {
    fn sync_read(&mut self) -> Result<O, Error> {
        let err = Err(ErrorKind::BrokenPipe.into());
        match self.read_object() {
            Some(HandShake::Begin) => { 
                self.write_object(HandShake::Send);
            },
            _ => {
                self.write_object(HandShake::Abort);
                return err 
            },
        }
        match self.read_object() {
            Some(msg) => {
                self.write_object(HandShake::Acknowledge);
                Ok(msg)
            }
            None => err
        }
    }

}

impl<T, O> SyncWrite<O> for T where
    O: Serialize ,
    T: ReadObject<HandShake> + WriteObject<HandShake> + ReadObject<O> + WriteObject<O>
    {
    fn sync_write(&mut self, write: O) -> Result<usize, Error> {
        let err = Err(ErrorKind::BrokenPipe.into());
        self.write_object(HandShake::Begin).expect("Sync write failed on BEGIN transaction");

        match self.read_object() {
            Some(HandShake::Send) => (),
            _ =>  {
                self.write_object(HandShake::Abort).expect("Failed to send Abort");
                return err
            }
        }
        
        let size = self.write_object(write).expect("Sync write failed on write");

        match self.read_object() {
            Some(HandShake::Acknowledge) => (),
            _ =>  {
                return err
            }
        }
        Ok(size)
    }

}
