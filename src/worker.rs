use std::net::TcpStream;
use std::io::prelude;
use std::io;
use crate::traits::{WriteObject, ReadObject};
use crate::message::Message;
use crate::identity;

pub struct Worker {
    stream: TcpStream,
    is_ready: bool
}

impl Worker {
    pub fn new(ip_addr: &str, port: &str) -> Result<Self, io::Error> {
        let mut stream = TcpStream::connect(format!("{}:{}", ip_addr, port)).unwrap();
        if let Ok(_) = stream.write_object(identity::Identity::Worker) {
            println!("Connected to manager at {}:{}", ip_addr, port);
            Ok(Worker { stream, is_ready: true })
        }
        else {
            Err(io::ErrorKind::ConnectionRefused.into())
        }
    }


    fn signal_ready_for_task(&mut self) -> Result<(), io::Error> {
        match self.stream.read_object() {
            Some(Message::IsReady) => {
                if self.is_ready {
                    self.stream.write_object(Message::ReadyForTask)?;
                    Ok(())
                }
                else {
                    self.stream.write_object(Message::TaskInProgress)?;
                    Ok(())
                }
            }, 
            _ => Err(io::ErrorKind::InvalidData.into())
        }
    }


    pub fn run(&mut self) -> Result<(), io::Error> {
        let mut buffer = [0; 1024];
        loop {
            self.signal_ready_for_task()?;
        }

        Ok(())
    }

}
