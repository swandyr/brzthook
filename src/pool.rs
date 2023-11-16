use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

use log::{info, warn};

pub(super) struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.;
    pub(super) fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let threads: Vec<Worker> = (0..size)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect();

        Self {
            workers: threads,
            sender: Some(sender),
        }
    }

    /// Execute a function job on a thread
    pub(super) fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }

    pub fn num_threads(&self) -> usize {
        self.workers.len()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            info!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            {
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        info!("Worker {id} got a job; executing.");
                        job();
                    }
                    Err(_) => {
                        warn!("Worker {id} disconnected; shittung down.");
                        break;
                    }
                }
            }
        });

        Self {
            id,
            thread: Some(thread),
        }
    }
}
