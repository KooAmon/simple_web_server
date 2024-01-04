use uuid::Uuid;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::logger::{log, LogLevel};

#[derive(Debug)]
pub enum PoolCreationError {
    InvalidSize,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize, serverid: Uuid) -> Result<ThreadPool, PoolCreationError> {

        if size <= 0 { return Err(PoolCreationError::InvalidSize); }

        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));


        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver), serverid));
        }

        return Ok(ThreadPool { workers, sender: Some(sender) });
    }

    pub fn execute<F>(&self, f: F)
    where F: FnOnce() + Send + 'static, {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            log(LogLevel::Trace, &Uuid::nil(), &Uuid::nil(), format!("Shutting down worker {}", worker.id).as_str());
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>, serverid: Uuid) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    log(LogLevel::Trace, &serverid, &Uuid::nil(), format!("Worker {} got a job.", id).as_str());
                    job();
                    log(LogLevel::Trace, &serverid, &Uuid::nil(), format!("Worker {} finished a job", id).as_str());
                },
                Err(_) => {
                    log(LogLevel::Trace, &serverid, &Uuid::nil(), format!("Worker {} shutting down", id).as_str());
                    break;

                }
            }
        });

        return Worker { id, thread: Some(thread) };
    }
}