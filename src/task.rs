use std::mem::take;
use std::cell::RefCell;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::process::{Command, Child, ExitStatus};
use serde::{Serialize, Deserialize};
use bincode;


#[derive(Debug)]
pub enum TaskError {
    NotRunning,
    FailedWait,
}


impl Display for TaskError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let display = match self {
            TaskError::NotRunning => String::from("NotRunning"),
            TaskError::FailedWait => String::from("FailedWait")
        };
        write!(f, "{}", display)
    }
}

impl Error for TaskError {} 

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum TaskState { 
    New,
    Running,
    Failed,
    Succeeded
}


pub struct Task {
    command: String,
    handle: RefCell<Option<Child>>,
    state: TaskState
}


impl Task {
    pub fn new(command: String) -> Task {
        Task {
            command,
            handle: RefCell::new(None),
            state: TaskState::New
        }
    }

    pub fn start(&mut self) {
        let split_command: Vec<&str> = self.command.split_whitespace().collect();
        let (command, args) = split_command.split_at(1);
        let child = Command::new(command[0])
                              .args(args)
                              .spawn()
                              .expect("Failed starting to run command");

        self.handle = RefCell::new(Some(child));
        self.state = TaskState::Running;
    }

    pub fn poll_command_state(&mut self) -> Result<TaskState, TaskError> { 
        println!("Polling task status");
        match self.state { 
            TaskState::Running => (),
            _ => return Err(TaskError::NotRunning)
        }
            
        match self.handle.borrow_mut().as_mut().unwrap().try_wait() {
            Ok(Some(status)) => { 
                    if status.success() {
                        self.state = TaskState::Succeeded;
                    }
                    else {
                        self.state = TaskState::Failed;
                    }
            },
            Ok(None) => (),
            Err(e) => return Err(TaskError::FailedWait) 
        }

        Ok(self.state)
    }
}

