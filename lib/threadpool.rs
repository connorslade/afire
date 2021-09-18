// Define a thread pool to execute tasks in parallel.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// A thread pool
pub(crate) struct ThreadPool {
    /// Worker threads
    workers: Vec<Worker>,

    /// Sender Channel
    sender: mpsc::Sender<Message>,
}

/// Worker thread
struct Worker {
    thread: Option<std::thread::JoinHandle<()>>,
}

/// A task to be executed in the thread pool
struct Message {
    /// A function to execute
    pub task: Box<dyn FnOnce() + Send + 'static>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        // Make sure the size is not zero
        assert!(size > 0);

        let (tx, rx) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(rx));
        let mut workers = Vec::with_capacity(size);

        for _ in 0..size {
            workers.push(Worker::new(Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: tx,
        }
    }

    /// Execute a task in the thread pool.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message { task: job }).unwrap();
    }
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = std::thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            message.task();
        });

        Worker {
            thread: Some(thread),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &mut self.workers {
            self.sender.send(Message::new(Box::new(|| {}))).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Message {
    pub fn new(task: Box<dyn FnOnce() + Send + 'static>) -> Message {
        Message { task }
    }

    // Add task function to run message.
    pub fn task(self) {
        (self.task)();
    }
}
