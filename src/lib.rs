use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

// ThreadPool是一个类似于java的ThreadPoolExecutor的程序
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::SyncSender<Message>,
    max_size: usize,
    rx: Arc<Mutex<mpsc::Receiver<Message>>>,
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
                    println!("Worker {} got a job; executing.", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                }
            }
        });
        Worker { id, thread }
    }
}

impl ThreadPool {
    pub fn new(size: usize, max_size: usize, buffer_size: usize) -> ThreadPool {
        assert!(size > 0);
        assert!(size <= max_size);
        let (sender, receiver) = mpsc::sync_channel(buffer_size);
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers,
            sender,
            max_size,
            rx: receiver,
        }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        // self.sender.send(Message::NewJob(job)).unwrap();
        let send_result = self.sender.try_send(Message::NewJob(job));
        if let Err(err) = send_result {
            println!("queue size is full, start create new worker");
            if self.workers.len() < self.max_size {
                self.workers.push(Worker::new(self.workers.len(), Arc::clone(&self.rx)))
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut pool = ThreadPool::new(4, 10, 10);
        for _ in 0..10 {
            pool.execute(move || {
                sleep(Duration::from_secs(1));
            });
        }
        drop(pool);
        sleep(Duration::from_secs(5));
    }

    #[test]
    fn test_buffer_size() {
        let mut pool = ThreadPool::new(4, 6, 0);
        for _ in 0..10 {
            pool.execute(move || {
                sleep(Duration::from_secs(1));
            });
        }
        drop(pool);
        sleep(Duration::from_secs(5));
    }
}
