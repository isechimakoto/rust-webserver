use std::{sync::{Arc, Mutex, mpsc}, thread};

trait FnBox {
  fn call_box(self: Box<Self>);
}

impl <F: FnOnce()> FnBox for F {
    fn call_box(self: Box<Self>) {
        (*self)()
    }
}

struct Worker {
  id: usize,
  thread: thread::JoinHandle<()>,
}

impl Worker {
  fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
    let thread = thread::spawn(move || {
      loop {
        let job = reciever.lock().unwrap().recv().unwrap();
        println!("Worker {} got a job; executing.", id);
        job.call_box();
      }
    });

    Worker { id, thread }
  }
}

type Job = Box<dyn FnBox + Send + 'static>;

pub struct ThreadPool {
  workers: Vec<Worker>,
  sender: mpsc::Sender<Job>,
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

    let (sender, reciever) = mpsc::channel();

    let reciever = Arc::new(Mutex::new(reciever));

    let mut workers = Vec::with_capacity(size);

    for id in 0..size  {
        workers.push(Worker::new(id, Arc::clone(&reciever)));
    }

    ThreadPool {
      workers,
      sender,
    }
  }

  pub fn execute<F>(&self, f: F)
    where F: FnOnce() + Send + 'static
  {
    let job = Box::new(f);
    self.sender.send(job).unwrap();
  }
}