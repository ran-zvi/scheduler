use std::thread::{Thread, JoinHandle};
use std::sync::{Arc, Mutex};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::env;
use std::io::prelude::*;
use std::io;

use std::{
    error,
    fmt
    };

use task::Task;
use message::Message;
use traits::{ReadObject, WriteObject};
use worker::Worker;

mod result;
mod identity;
mod task;
mod message;
mod worker;
mod traits;


#[derive(Debug)]
struct TempError(String);


impl fmt::Display for TempError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?})", &self)
    }

}

impl error::Error for TempError {}



pub struct TaskName(String);


#[derive(Debug)]
struct WorkerHandle {
    id: usize,
    stream: TcpStream,
}

impl WorkerHandle {

    pub fn new(id: usize, mut stream: TcpStream) -> Result<Self, io::Error> {
        match stream.read_object() {
            Some(identity::Identity::Worker) => Ok(WorkerHandle {id, stream}),
            None => Err(io::ErrorKind::Other.into())
        }
    }


    pub fn is_ready(&mut self) -> bool {
        self.stream.write_object(Message::IsReady).unwrap();
        match self.stream.read_object() { 
            Some(Message::ReadyForTask) => true,
            _ => false
        }
    }
}


fn main() -> crate::result::Result<()> {
    let args: Vec<String> = env::args().collect();

    let selection = &args[1];

    match selection.as_str() {
        "--manager" => run_manager(),
        "--worker" => run_worker(),
        _ => Err(Box::new(TempError("Invalid selection".into())))
    }
}


fn run_manager() -> crate::result::Result<()> {
    print!("[Starting Manager]\n");

    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    let mut workers: Vec<WorkerHandle> = vec![];


    for stream in listener.incoming() {
        let stream: TcpStream = stream.unwrap().try_clone().unwrap();
        println!("connection");
        if let Ok(worker) = WorkerHandle::new(workers.len(), stream) {
            println!("Worker joined: {}", worker.id);
            workers.push(worker);
        }

        for worker in workers.iter_mut() {
            if worker.is_ready() {
                println!("Ready for tasks!");
            }
            else {
                println!("Task in progress!");
                }
            }
        }
    Ok(())
}


fn run_worker() -> crate::result::Result<()> {
    print!("[Starting Worker]\n");
        
    let mut worker = Worker::new("localhost","3333")?;
    worker.run()?;
    Ok(())

}
