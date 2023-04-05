use std::{
    thread::self,
    sync::{Arc, mpsc, Mutex},
};

mod tests;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::SyncSender<Job>,
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let lock = receiver.lock()
                    .expect("Worker Unable to get the receiver lock.");
                let message = lock.recv();
                match message {
                    Ok(job) => {
                        println!("Worker {} got a job; executing.", id);
                        job();
                    }
                    Err(_) => {
                        println!("Worker {} received a broken channel.", id);
                        break;
                    }
                };
            }
        });
        Worker { id, thread }
    }
}

impl ThreadPool {
    pub fn new(size: usize, buffer_size: usize) -> ThreadPool {
        assert!(size > 0, "size must be positive.");
        let (sender, receiver) = mpsc::sync_channel(buffer_size);
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&mut self, f: F)
        where
            F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}