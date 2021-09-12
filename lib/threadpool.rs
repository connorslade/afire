// Define a thread pool to execute tasks in parallel.

use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

pub struct Worker {
    id: usize,
    thread: Option<std::thread::JoinHandle<()>>,
}

pub struct Message {
    pub id: usize,
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
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message { id: 0, task: job }).unwrap();
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Worker {
        let thread = std::thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            message.task();
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &mut self.workers {
            self.sender
                .send(Message::new(0, Box::new(|| {})))
                .unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Message {
    pub fn new(id: usize, task: Box<dyn FnOnce() + Send + 'static>) -> Message {
        Message { id, task }
    }

    // Add task function to run message.
    pub fn task(self) {
        (self.task)();
    }
}
