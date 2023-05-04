//! A thread pool implementation.
//! Used for handling multiple connections at once.

use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::internal::common::ForceLock;

/// Messages that can be handled by the pool's workers.
enum Message {
    /// Stops the worker.
    Kill,
    /// A job to be executed by the worker.
    Job(Box<dyn FnOnce() + 'static + Send>),
}

/// A thread pool.
pub(crate) struct ThreadPool {
    /// The number of threads in the pool.
    threads: usize,
    /// Handle to each worker thread.
    workers: Vec<Worker>,
    /// The channel used to send messages to the workers.
    sender: mpsc::Sender<Message>,
}

/// A worker thread.
/// Contains a handle to the thread, and an id.
struct Worker {
    _id: usize,
    handle: Option<JoinHandle<()>>,
}

impl ThreadPool {
    /// Creates a new thread pool with the specified number of threads.
    /// Panics if `size` is 0.
    pub(crate) fn new(size: usize) -> Self {
        assert!(size > 0);

        let (tx, rx) = mpsc::channel();
        let mut workers = Vec::with_capacity(size);

        let receiver = Arc::new(Mutex::new(rx));
        for i in 0..size {
            workers.push(Worker::new(i, Arc::clone(&receiver)));
        }

        Self {
            threads: size,
            sender: tx,
            workers,
        }
    }

    /// Executes a job on the thread pool.
    pub(crate) fn execute(&self, f: impl FnOnce() + 'static + Send) {
        let job = Message::Job(Box::new(f));
        self.sender.send(job).unwrap();
    }
}

impl Worker {
    /// Creates a new worker thread.
    fn new(id: usize, rx: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let handle = thread::Builder::new()
            .name(format!("Worker {id}"))
            .spawn(move || loop {
                let job = rx.force_lock().recv().unwrap();
                match job {
                    Message::Job(job) => job(),
                    Message::Kill => break,
                }
            })
            .expect("Error creating worker thread");

        Self {
            _id: id,
            handle: Some(handle),
        }
    }
}

impl Drop for ThreadPool {
    /// Stops all workers with a [`Message::Kill`] message, and waits for them to finish.
    fn drop(&mut self) {
        for _ in 0..self.threads {
            self.sender.send(Message::Kill).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.handle.take() {
                thread.join().unwrap();
            }
        }
    }
}
