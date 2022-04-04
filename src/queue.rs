use std::collections::VecDeque;
use serde::{Serialize, Deserialize};


pub struct ReadWriteQueue<W, R> {
    writes: VecDeque<W>,
    reads: VecDeque<R>
}

impl<W, R> ReadWriteQueue<W,R> {
    pub fn from_iters(r_iter: impl IntoIterator<Item=R>, w_iter: impl IntoIterator<Item=W>) -> ReadWriteQueue<W,R> {
        ReadWriteQueue {
            writes: VecDeque::from_iter(w_iter),
            reads: VecDeque::from_iter(r_iter)
        }
    }

    pub fn writes_empty(&self) -> bool {
        self.writes.is_empty()
    }

    pub fn reads_empty(&self) -> bool {
        self.reads.is_empty()
    }

    pub fn push_to_writes(&mut self, w: W) {
        self.writes.push_back(w);
    }

    pub fn push_to_reads(&mut self, r: R) {
        self.reads.push_back(r);
    }

    pub fn pop_from_writes(&mut self) -> Option<W> {
        self.writes.pop_front()
    }

    pub fn pop_from_reads(&mut self) -> Option<R> {
        self.reads.pop_front()
    }
}
