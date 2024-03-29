use log::trace;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

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
    pub fn new(size: usize) -> Result<ThreadPool, PoolCreationError> {

        if size <= 0 { return Err(PoolCreationError::InvalidSize); }

        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));


        for id in 0..size {
            trace!("Creating worker {}", id);
            workers.push(Worker::new(id, Arc::clone(&receiver)));
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
            trace!("Shutting down worker {}", worker.id);
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
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        trace!("Worker {} created", id);
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    trace!("Worker {} got a job.", id);
                    job();
                    trace!("Worker {} finished a job", id);
                },
                Err(_) => {
                    trace!("Worker {} shutting down", id);
                    break;

                }
            }
        });

        return Worker { id, thread: Some(thread) };
    }
}