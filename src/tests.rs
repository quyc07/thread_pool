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
        drop(pool);
        sleep(Duration::from_secs(3));
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
}
