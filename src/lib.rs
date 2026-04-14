use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

pub struct ThreadPool{
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, _f: F)
    where 
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(_f);

        match self.sender.as_ref() {
            Some(send_ref) => match send_ref.send(job) {
                Ok(result) => result,
            Err(error) => panic!("Error sending job: {error:?}"),
            },
            None => println!("Failed to resolve sender ref"),
        };
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = match receiver.lock() {
                    Ok(lock) => lock.recv(),
                    Err(error) => panic!("Error locking receiver: {error:?}"),
                };

                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");

                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            }
        });

        Worker { id, thread }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers.drain(..) {
            println!("Shutting down worker {}", worker.id);

            match worker.thread.join() {
                Ok(result) => result,
                Err(error) => panic!("Failed to join thread: {error:?}"),
            };
        }
    }
}