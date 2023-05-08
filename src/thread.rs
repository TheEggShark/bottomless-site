use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};


#[allow(dead_code)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        (0..size).for_each(|id| workers.push(Worker::new(id, Arc::clone(&receiver))));

        ThreadPool {
            workers,
            sender,
        }
    }

    pub fn execute<F>(&self, function: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(function);

        self.sender.send(job).unwrap();
    }
}

#[allow(dead_code)]
struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {id} got a job; executing.");

            job();
        });

        Worker {
            id, 
            thread,
        }
    }
}