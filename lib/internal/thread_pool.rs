//! A resizable thread pool implementation.
//! Used for handling multiple connections at once.
//!
//! You can access the thread pool from within a route handler with [`Context::thread_pool`].
//! With this, you can resize the thread pool, get the current thread id, or execute a job on the thread pool.
//! To get the current thread id, use [`ThreadPool::current_thread`].
//! To execute a job on the thread pool, use [`ThreadPool::execute`].
//! To resize the thread pool, there are a few different functions:
//! - [`ThreadPool::resize_exact`] - Resizes the thread pool to the specified size.
//! - [`ThreadPool::increase`] - Spawns a new worker thread, increasing the thread pool size by 1.
//! - [`ThreadPool::decrease`] - Sends a kill message, decreasing the thread pool size by 1.
//!
//! For more information on how the thread pool works, see the documentation for [`ThreadPool`].

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

/// A thread pool.
///
/// Consists of a number of worker threads, and a channel to send messages to them.
/// When dropping the thread pool, all workers are stopped and joined.
/// When increasing the size of the thread pool, new workers are spawned.
/// When decreasing the size of the thread pool, a kill message is sent on the channel, and when processed, the worker removes itself from the pool.
///
/// Also note each worker has a unique id, which is calculated by incrementing a static counter.
/// This means that even when a worker is removed, the id will not be reused, until the counter overflows I suppose.
///
/// # Example
/// ```
/// # use afire::internal::thread_pool::ThreadPool;
/// let pool = ThreadPool::new_empty();
///
/// pool.increase();
/// pool.execute(|| {
///     println!("Hello from thread pool!");
/// });
/// ```
pub struct ThreadPool {
    /// Handle to each worker thread.
    workers: Workers,
    /// The number of threads in the pool.
    threads: AtomicUsize,

    /// The channel used to send messages to the workers.
    sender: Mutex<mpsc::Sender<Message>>,
    /// The channel used to receive messages to the workers.
    receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
}

/// Messages that can be handled by the pool's workers.
enum Message {
    /// Stops the worker.
    Kill,
    /// A job to be executed by the worker.
    Job(Box<dyn FnOnce() + 'static + Send>),
}

#[derive(Clone)]
struct Workers {
    inner: Arc<Mutex<Vec<Worker>>>,
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

    /// Create a new empty thread pool with zero threads.
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

    /// Returns the number of threads that should be in the pool.
    /// This is not necessarily the number of threads that are in the pool as when resizing, the threads are not immediately removed but the count is immediately updated.
    pub fn threads(&self) -> usize {
        self.threads.load(Ordering::Relaxed)
    }

    /// Returns the number of threads that are in the pool.
    /// This is more accurate than [`ThreadPool::threads`] as it does not update the count until the threads are actually removed.
    /// But it is also slower as it locks the workers mutex to count the threads.
    pub fn threads_exact(&self) -> usize {
        self.workers.inner.force_lock().len()
    }

    /// Returns the index of the thread calling this function.
    /// Returns `None` if the thread is not a worker thread of this thread pool.
    pub fn current_thread(&self) -> Option<usize> {
        let thread = thread::current();
        self.workers.find(thread.id())
    }

    /// Resizes the thread pool to the specified size.
    /// Depending on how the size changes, the [`ThreadPool::increase`] or [`ThreadPool::decrease`] functions are repeatedly called to resize the pool.
    pub fn resize_exact(&self, size: usize) {
        assert!(size > 0);
        trace!(Level::Debug, "Resizing thread pool to {}", size);
        let threads = self.threads();
        if size == threads {
            return;
        }

        // Spawn or remove workers
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

    /// Spawns a new worker thread, increasing the thread pool size by 1.
    pub fn increase(&self) {
        trace!(Level::Debug, "Increasing thread pool size by 1");
        self.workers
            .push(Worker::new(self.receiver.clone(), self.workers.clone()));
        self.threads.fetch_add(1, Ordering::Relaxed);
    }

    /// Sends a kill message to a worker thread, decreasing the thread pool size by 1.
    /// If all workers are busy, this will not force a worker to stop,
    pub fn decrease(&self) {
        trace!(Level::Debug, "Decreasing thread pool size by 1");
        let sender = self.sender.force_lock();
        sender.send(Message::Kill).unwrap();
        self.threads.fetch_sub(1, Ordering::Relaxed);
    }
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
        trace!(Level::Debug, "Worker thread #{id} killed");
        let mut list = self.inner.force_lock();
        let idx = list.iter().position(|x| x.id == id)?;
        list.remove(idx);
        drop(list);
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
        let mut workers = self.inner.force_lock();
        let handles = workers
            .iter_mut()
            .filter_map(|x| x.handle.take())
            .collect::<Vec<_>>();
        drop(workers);

        for handle in handles {
            handle.join().unwrap();
        }
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
                        return;
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
        trace!(
            Level::Debug,
            "Shutting down thread pool, {} threads. (On Drop)",
            self.threads()
        );
        let sender = self.sender.force_lock();
        for _ in self.workers.inner.force_lock().iter() {
            sender.send(Message::Kill).unwrap();
        }
        drop(sender);

        self.workers.join_all();
        trace!(Level::Debug, "Thread pool shut down");
    }
}
