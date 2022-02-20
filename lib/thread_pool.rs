use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};

enum Message {
    Kill,
    Job(Box<dyn FnOnce() + 'static + Send>),
}

pub(crate) struct ThreadPool {
    threads: usize,
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

struct Worker {
    _id: usize,
    handle: Option<JoinHandle<()>>,
}

impl ThreadPool {
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

    pub(crate) fn execute<F>(&self, f: F)
    where
        F: FnOnce() + 'static + Send,
    {
        let job = Message::Job(Box::new(f));
        self.sender.send(job).unwrap();
    }
}

impl Worker {
    fn new(id: usize, rx: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let handle = thread::spawn(move || loop {
            let job = rx.lock().unwrap().recv().unwrap();
            match job {
                Message::Job(job) => job(),
                Message::Kill => break,
            }
        });

        Self {
            _id: id,
            handle: Some(handle),
        }
    }
}

impl Drop for ThreadPool {
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
