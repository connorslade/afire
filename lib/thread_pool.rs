//! A thread pool implementation.
//! Used for handling multiple connections at once.

use std::{
    panic,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    },
    thread::{self, JoinHandle, ThreadId},
};

use crate::{
    internal::{misc::any_string, sync::ForceLockMutex},
    trace,
};

/// Messages that can be handled by the pool's workers.
enum Message {
    /// Stops the worker.
    Kill,
    /// A job to be executed by the worker.
    Job(Box<dyn FnOnce() + 'static + Send>),
}

/// A thread pool.
pub struct ThreadPool {
    /// The number of threads in the pool.
    threads: AtomicUsize,

    /// Handle to each worker thread.
    workers: Workers,

    /// The channel used to send messages to the workers.
    sender: Mutex<mpsc::Sender<Message>>,
    /// The channel used to receive messages to the workers.
    receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
}

#[derive(Clone)]
pub struct Workers {
    inner: Arc<Mutex<Vec<Worker>>>,
}

impl Workers {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn push(&self, worker: Worker) {
        self.inner.force_lock().push(worker);
    }

    fn remove(&self, id: usize) -> Option<()> {
        let mut list = self.inner.force_lock();
        let idx = list.iter().position(|x| x.id == id)?;
        list.remove(idx);
        Some(())
    }

    fn find(&self, handle: ThreadId) -> Option<usize> {
        self.inner
            .force_lock()
            .iter()
            .find(|x| x.handle.as_ref().unwrap().thread().id() == handle)
            .map(|x| x.id)
    }

    fn join_all(&self) {
        self.inner.force_lock().iter_mut().for_each(|x| {
            if let Some(handle) = x.handle.take() {
                handle.join().unwrap();
            }
        })
    }
}

/// A worker thread.
/// Contains a handle to the thread, and an id.
struct Worker {
    id: usize,
    handle: Option<JoinHandle<()>>,
}

impl ThreadPool {
    /// Creates a new thread pool with the specified number of threads.
    /// Panics if `size` is 0.
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, rx) = mpsc::channel();
        let workers = Workers::new();

        let receiver = Arc::new(Mutex::new(rx));
        for _ in 0..size {
            workers.push(Worker::new(Arc::clone(&receiver), workers.clone()));
        }

        Self {
            threads: AtomicUsize::new(size),
            sender: Mutex::new(sender),
            workers: Workers::new(),
            receiver,
        }
    }

    pub fn new_empty() -> Self {
        let (sender, rx) = mpsc::channel();
        Self {
            threads: AtomicUsize::new(0),
            sender: Mutex::new(sender),
            receiver: Arc::new(Mutex::new(rx)),
            workers: Workers::new(),
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

    /// Returns the index of the thread calling this function.
    /// Returns `None` if the thread is not a worker thread.
    pub fn current_thread(&self) -> Option<usize> {
        let thread = thread::current();
        self.workers.find(thread.id())
    }

    pub fn resize(&self, size: usize) {
        assert!(size > 0);
        trace!(Level::Debug, "Resizing thread pool to {}", size);
        let threads = self.threads();
        if size == threads {
            return;
        }

        // Spawn or remove  workers
        if size > threads {
            let to_add = size - threads;
            for _ in 0..to_add {
                self.increase();
            }
        } else {
            let to_remove = threads - size;
            for _ in 0..to_remove {
                self.decrease();
            }
        }
    }

    pub fn increase(&self) {
        trace!(Level::Debug, "Increasing thread pool size by 1");
        self.workers
            .push(Worker::new(self.receiver.clone(), self.workers.clone()));
        self.threads.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrease(&self) {
        trace!(Level::Debug, "Decreasing thread pool size by 1");
        let sender = self.sender.force_lock();
        sender.send(Message::Kill).unwrap();
        self.threads.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Worker {
    /// Creates a new worker thread.
    fn new(rx: Arc<Mutex<mpsc::Receiver<Message>>>, workers: Workers) -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

        let handle = thread::Builder::new()
            .name(format!("afire Worker {id}"))
            .spawn(move || loop {
                let job = rx.force_lock().recv().unwrap();
                match job {
                    Message::Job(job) => {
                        let result = panic::catch_unwind(panic::AssertUnwindSafe(job));
                        if let Err(err) = result {
                            trace!(
                                Level::Error,
                                "Worker thread #{} panicked: '{}'",
                                id,
                                any_string(err)
                            );
                        }
                    }
                    Message::Kill => {
                        workers.remove(id);
                        break;
                    }
                }
            })
            .expect("Error creating worker thread");

        Self {
            id,
            handle: Some(handle),
        }
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

        self.workers.join_all();
        trace!(Level::Debug, "Thread pool shut down");
    }
}
