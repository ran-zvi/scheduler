use std::net::TcpStream;
use std::convert::TryFrom;
use std::io::prelude::*;
use serde::{Serialize, Deserialize};
use bincode;
use crate::task::{Task, TaskState};


#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Message {
    IsReady,
    ReadyForTask,
    Invalid,
    Kill,
    Task(TaskKind)
}


#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TaskKind {
    Kill,
    State(TaskState),
    InProgress,
    Run(String)
}


#[derive(Debug, Serialize, Deserialize)]
pub enum HandShake {
    Unreachable,
    Acknowledge,
    Send,
    Begin,
    Abort
}
