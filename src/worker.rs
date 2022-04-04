use std::{
    net::TcpStream,
    mem::take,
    io::prelude,
    io,
    sync::{Arc, Mutex, mpsc},
    cell::RefCell,
    time::Duration,
    thread
};
use serde::{Deserialize,Serialize};
use crate::{
    traits::{WriteObject, ReadObject, SyncWrite, SyncRead}, 
    message::{Message, TaskKind}, 
    identity, queue::ReadWriteQueue,
    task::{Task, TaskState}
};


#[derive(Debug, Deserialize, Serialize)]
enum MessageKind {
    Read(Message),
    Write(Message)
}


pub struct Worker {
    stream: TcpStream,
    is_ready: bool,
    current_task: RefCell<Option<Task>>
}


impl Worker {
    pub fn new(ip_addr: &str, port: &str) -> Result<Self, io::Error> {
        let mut stream = TcpStream::connect(format!("{}:{}", ip_addr, port)).expect("Unable to connect to host");
        stream.set_nonblocking(true).expect("Unable to set stream as non blocking");
        if let Ok(_) = stream.write_object(identity::Identity::Worker) {
            println!("Connected to manager at {}:{}", ip_addr, port);
            let temp_v: Vec<Message> = vec![];
            Ok(Worker { 
                stream, 
                is_ready: true,
                current_task: RefCell::new(None)
            })
        }
        else {
            Err(io::ErrorKind::ConnectionRefused.into())
        }
    }


    fn communicate_is_ready(&mut self) -> Result<(), io::Error> {
        if self.is_ready {
            self.stream.write_object(Message::ReadyForTask)?;
            Ok(())
        }
        else {
            self.stream.write_object(Message::Task(TaskKind::InProgress))?;
            Ok(())
        }
    }

    fn run_task(&mut self, cmd: String) -> Result<(), io::Error> {
        let mut task = Task::new(cmd);
        task.start();
        self.current_task = RefCell::new(Some(task));
        Ok(())
    }

    fn process_read_message(&mut self, message: Message) -> () {
        println!("Processing message: {:?}", message);
        match message {
            Message::IsReady => self.communicate_is_ready().expect("communicate_is_ready failed"),
            Message::Task(TaskKind::Run(cmd)) => self.run_task(cmd).expect("Failed to run task"),
            _ => ()
        }
    }

    fn process_write_message(&mut self, message: Message) -> () {
        println!("Processing message: {:?}", message);
        match message {
            Message::Task(TaskKind::State(state)) => {
                self.stream.write_object(message).expect("Failed to send task state");
            }
            _ => ()
        };
        println!("Finished processing message");
    }

    fn reads_generator(stream: Arc<Mutex<TcpStream>> , sender: Arc<Mutex<mpsc::Sender<MessageKind>>>) {
        let mut sender = sender.lock().unwrap(); 
        let mut stream = stream.lock().unwrap();

        loop { 
            if let Some(msg) = stream.read_object() {
                sender.send(MessageKind::Read(msg)); 
            }
            thread::sleep(Duration::from_millis(100));
        }
    }


    pub fn run(&mut self) -> Result<(), io::Error> {
        let (sender, receiver) = mpsc::channel();
        let reader_sender = Arc::new(Mutex::new(sender.clone()));
        let read_stream = Arc::new(Mutex::new(self.stream.try_clone().expect("Failed to clone stream")));
        let reader_handle = thread::spawn(move || {
            Worker::reads_generator(read_stream ,reader_sender as Arc<Mutex<mpsc::Sender<MessageKind>>>);
        });


        loop {

            match receiver.recv_timeout(Duration::from_millis(10)) {
                Ok(MessageKind::Read(m)) => {
                    match m {
                        Message::Kill => break,
                        _ => self.process_read_message(m)
                    }
                },
                Ok(MessageKind::Write(m)) => {
                    match m {
                        Message::Kill => break,
                        _ => self.process_write_message(m)
                    }
                },
                Err(_) => ()

            }
            println!("Checking if I have task");
            let mut task_done = false;
            {
                let mut refcell = self.current_task.borrow_mut();
                let mut task = refcell.as_mut(); 
                match task {
                    Some(t) => match t.poll_command_state() {
                        Ok(task_state) if task_state == TaskState::Running => {
                            sender.send(MessageKind::Write(Message::Task(TaskKind::State(task_state))));
                        },
                        Ok(task_state) if task_state == TaskState::Succeeded => {
                            sender.send(MessageKind::Write(Message::Task(TaskKind::State(task_state))));
                            task_done = true;
                        },
                        _ => (),
                    },
                    None => ()
                }
            }
            thread::sleep(Duration::from_secs(1));
            if task_done {
                self.current_task = RefCell::new(None);
                self.is_ready = true;
            }
        }

        reader_handle.join();

        Ok(())
    }

}
