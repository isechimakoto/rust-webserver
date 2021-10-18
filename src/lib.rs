use std::{sync::{Arc, Mutex, mpsc}, thread};

enum Message {
  NewJob(Job),
  Terminate,
}

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
  thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
  fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
    let thread = thread::spawn(move || {
      loop {
        let message = reciever.lock().unwrap().recv().unwrap();

        match message {
          Message::NewJob(job) => {
              println!("Worker {} got a job; executing.", id);

              job.call_box();
          },
          Message::Terminate => {
              println!("Worker {} was told to terminate.", id);
              break;
          }
        }
      }
    });

    Worker { id, thread: Some(thread) }
  }
}

type Job = Box<dyn FnBox + Send + 'static>;

pub struct ThreadPool {
  workers: Vec<Worker>,
  sender: mpsc::Sender<Message>,
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
    self.sender.send(Message::NewJob(job)).unwrap();
  }
}

impl Drop for ThreadPool {
  fn drop(&mut self) {
    println!("Sending terminate message to all workers.");

    for _ in &mut self.workers {
      self.sender.send(Message::Terminate).unwrap();
    }

    println!("Shutting down all workers.");

    for worker in &mut self.workers {
      println!("Shutting down worker {}", worker.id);

      if let Some(thread) = worker.thread.take() {
        thread.join().unwrap();
      }
    }
  }
}