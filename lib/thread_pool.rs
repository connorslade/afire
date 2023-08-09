//! A thread pool implementation.
//! Used for handling multiple connections at once.

use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc, Barrier, Mutex,
    },
    thread::{self, JoinHandle},
};

use crate::{internal::sync::ForceLockMutex, trace};

/// Messages that can be handled by the pool's workers.
enum Message {
    /// Stops the worker.
    Kill,
    /// Stops the worker and waits for a barrier.
    KillWait(Arc<Barrier>),
    /// A job to be executed by the worker.
    Job(Box<dyn FnOnce() + 'static + Send>),
}

/// A thread pool.
pub struct ThreadPool {
    /// The number of threads in the pool.
    threads: AtomicUsize,
    /// Next ID to use for a worker.
    worker_id: AtomicUsize,
    /// Handle to each worker thread.
    workers: Mutex<Vec<Worker>>,
    /// The channel used to send messages to the workers.
    sender: Mutex<mpsc::Sender<Message>>,
    /// The channel used to receive messages to the workers.
    receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
}

/// A worker thread.
/// Contains a handle to the thread, and an id.
struct Worker {
    _id: usize,
    handle: Option<JoinHandle<()>>,
    dead: Arc<AtomicBool>,
}

impl ThreadPool {
    /// Creates a new thread pool with the specified number of threads.
    /// Panics if `size` is 0.
    pub(crate) fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, rx) = mpsc::channel();
        let mut workers = Vec::with_capacity(size);

        let receiver = Arc::new(Mutex::new(rx));
        for i in 0..size {
            workers.push(Worker::new(i, Arc::clone(&receiver)));
        }

        Self {
            threads: AtomicUsize::new(size),
            worker_id: AtomicUsize::new(size),
            sender: Mutex::new(sender),
            workers: Mutex::new(workers),
            receiver,
        }
    }

    pub(crate) fn new_empty() -> Self {
        let (sender, rx) = mpsc::channel();
        Self {
            threads: AtomicUsize::new(0),
            worker_id: AtomicUsize::new(0),
            sender: Mutex::new(sender),
            workers: Mutex::new(Vec::new()),
            receiver: Arc::new(Mutex::new(rx)),
        }
    }

    /// Executes a job on the thread pool.
    pub fn execute(&self, f: impl FnOnce() + 'static + Send) {
        let job = Message::Job(Box::new(f));
        self.sender.force_lock().send(job).unwrap();
    }

    pub fn threads(&self) -> usize {
        self.threads.load(Ordering::Relaxed)
    }

    pub fn resize(&self, size: usize) {
        assert!(size > 0);
        trace!(Level::Debug, "Resizing thread pool to {}", size);
        let threads = self.threads();
        if size == threads {
            return;
        }

        // Spawn new workers
        if size > threads {
            let to_add = size - threads;
            let mut workers = self.workers.force_lock();
            for _ in 0..to_add {
                let id = self.worker_id.fetch_add(1, Ordering::Relaxed);
                workers.push(Worker::new(id, self.receiver.clone()));
            }
            self.threads.store(size, Ordering::Relaxed);
            return;
        }

        // Remove workers
        let to_remove = threads - size;
        let sender = self.sender.force_lock();

        // Kill workers
        let barrier = Arc::new(Barrier::new(to_remove + 1));
        (0..to_remove).for_each(|_| sender.send(Message::KillWait(barrier.clone())).unwrap());
        barrier.wait();

        // Remove dead workers
        let mut workers = self.workers.force_lock();
        workers.retain(|worker| !worker.is_dead());
        self.threads.store(size, Ordering::Relaxed);
    }
}

impl Worker {
    /// Creates a new worker thread.
    fn new(id: usize, rx: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let dead = Arc::new(AtomicBool::new(false));
        let this_dead = dead.clone();
        let handle = thread::Builder::new()
            .name(format!("Worker {id}"))
            .spawn(move || loop {
                let job = rx.force_lock().recv().unwrap();
                match job {
                    Message::Job(job) => job(),
                    Message::KillWait(barrier) => {
                        this_dead.store(true, Ordering::Relaxed);
                        barrier.wait();
                        break;
                    }
                    Message::Kill => {
                        this_dead.store(true, Ordering::Relaxed);
                        break;
                    }
                }
            })
            .expect("Error creating worker thread");

        Self {
            _id: id,
            handle: Some(handle),
            dead,
        }
    }

    fn is_dead(&self) -> bool {
        self.dead.load(Ordering::Relaxed)
    }
}

impl Drop for ThreadPool {
    /// Stops all workers with a [`Message::Kill`] message, and waits for them to finish.
    fn drop(&mut self) {
        trace!(Level::Debug, "Shutting down thread pool. (On Drop)");
        let sender = self.sender.force_lock();
        for _ in 0..self.threads() {
            sender.send(Message::Kill).unwrap();
        }

        for worker in self.workers.force_lock().iter_mut() {
            if let Some(thread) = worker.handle.take() {
                thread.join().unwrap();
            }
        }
        trace!(Level::Debug, "Thread pool shut down");
    }
}
