use std::{
    thread::{Thread, JoinHandle},
    thread,
    sync::{Arc, Mutex},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    env,
    io::prelude::*,
    io,
    error,
    fmt,
    thread::sleep,
    time::Duration
};

use task::{Task, TaskState};
use message::{Message, TaskKind};
use traits::{ReadObject, WriteObject, SyncRead, SyncWrite};
use worker::Worker;

mod result;
mod identity;
mod task;
mod message;
mod worker;
mod traits;
mod queue;


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
            _ => Err(io::ErrorKind::Other.into())
        }
    }


    pub fn is_ready(&mut self) -> bool {
        match self.stream.write_object(Message::IsReady) {
            Ok(_) => println!("Worker {} is ready for tasks", self.id),
            _ => eprintln!("Failed to write to stream")
        }
        match self.stream.read_object() { 
            Some(Message::ReadyForTask) => true,
            _ => false
        }
    }

    pub fn run_task(&mut self, task: String) -> Result<(), io::Error> {
        match self.stream.write_object(Message::Task(TaskKind::Run(task))) {
            Ok(_) => { 
                println!("Task successfully sent to worker: {}", self.id);
                Ok(())
            },
            Err(err) => return Err(err)
        }
    }

    pub fn get_current_task_status(&mut self) -> TaskState {
        let result: Option<Message> = self.stream.read_object();
        match result {
            Some(m) if m == Message::Task(TaskKind::State(TaskState::Running)) => {
                println!("Worker: {} is still running the task", self.id);
                TaskState::Running
            },
            Some(m) if m == Message::Task(TaskKind::State(TaskState::Succeeded)) => {
                println!("Worker: {} finished runnning the task", self.id);
                TaskState::Succeeded
            },
            _ => TaskState::Failed
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
    let mut workers: Arc<Mutex<Vec<WorkerHandle>>> = Arc::new(Mutex::new(vec![]));
    let tasks_: Vec<String> = (1..=5).collect::<Vec<u8>>().iter().map(|_| String::from("sleep 3")).collect();
    let tasks: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(tasks_));
    

    let mut conn_workers = Arc::clone(&workers);
    let mut workers_num = 0;
    let conn_handle = thread::spawn(move || {
        for stream in listener.incoming() {
            let stream: TcpStream = stream.unwrap().try_clone().unwrap();
            stream.set_nonblocking(true).expect("Failed to set non blocking");
            let mut workers = conn_workers.lock().unwrap();
            if let Ok(worker) = WorkerHandle::new(workers.len(), stream) {
                println!("Worker joined: {}", worker.id);
                workers.push(worker);
            }
        }
    });

    let mut task_workers = Arc::clone(&workers);
    let mut shared_tasks = Arc::clone(&tasks);

    let task_handle = thread::spawn(move || {
        loop {
            let mut tasks = shared_tasks.lock().unwrap();
            let mut workers = task_workers.lock().unwrap();
            for worker in workers.iter_mut() {
                if !tasks.is_empty() && worker.is_ready() {
                    if let Some(task) = tasks.pop() { 
                        worker.run_task(task).expect("Failed to send task to worker");
                        println!("{} tasks left in the queue", tasks.len());
                    }
                }
                else {
                    let id = worker.id;
                    println!("Worker - {}: {:?}", id, worker.get_current_task_status());
                }
            }
        }
    });

    conn_handle.join();
    task_handle.join();
    
    Ok(())
}


fn run_worker() -> crate::result::Result<()> {
    print!("[Starting Worker]\n");
        
    let mut worker = Worker::new("localhost","3333")?;
    Worker::run(&mut worker)?;
    Ok(())
}
