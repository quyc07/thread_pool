use std::{
    thread::self,
    sync::{Arc, mpsc, Mutex},
};

mod tests;

// ThreadPool是一个类似于java的ThreadPoolExecutor的程序
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::SyncSender<Message>,
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

enum Message {
    NewJob(Job),
    Terminate,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    job();
                }
                Message::Terminate => {
                    break;
                }
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
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        println!("Shutting down all workers.");
    }
}
