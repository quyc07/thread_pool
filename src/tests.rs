#[cfg(test)]
mod tests {
    use std::thread::sleep;
    use std::time::Duration;
    use crate::*;

    #[test]
    fn test_thread_pool() {
        let mut pool = ThreadPool::new(4, 10);
        for _ in 0..10 {
            pool.execute(move || {
                sleep(Duration::from_secs(1));
            });
        }
        sleep(Duration::from_secs(3));
        drop(pool);
    }

    #[test]
    fn test_buffer_size() {
        let mut pool = ThreadPool::new(4, 0);
        for i in 0..10 {
            pool.execute(move || {
                println!("send {}", i);
                sleep(Duration::from_millis(1));
            });
        }
        drop(pool);
        sleep(Duration::from_secs(3));
    }

    #[test]
    fn test_thread_pool_size() {

        let n_workers = 4;
        let n_jobs = 8;
        let mut pool = ThreadPool::new(n_workers, 100);

        let (tx, rx) = mpsc::channel();
        for _ in 0..n_jobs {
            let tx = tx.clone();
            pool.execute(move|| {
                tx.send(1).expect("channel will be there waiting for the pool");
            });
        }

        assert_eq!(rx.iter().take(n_jobs).fold(0, |a, b| a + b), 8);
    }
}
