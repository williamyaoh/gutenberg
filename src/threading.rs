use crossbeam::{self, sync};

use std::thread;
use std::sync::mpsc;
use std::mem;

enum Message {
    Job(Box<FnBox + Send>),
    NoMoreJobs
}

#[doc(hidden)]
trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<Self>) {
        (*self)();
    }
}

type WorkQueue = sync::MsQueue<mpsc::SyncSender<Message>>;

pub struct Scope {
    available: WorkQueue,
    handles: Vec<thread::JoinHandle<()>>
}

impl Scope {
    fn spawn<'a, F>(&'a self, f: F) where
        F: FnOnce() + 'a + Send
    {
        let worker = self.available.pop();
        let fnbox: Box<FnBox + 'a + Send> = Box::new(f);
        let fnbox: Box<FnBox + Send> = unsafe { mem::transmute(fnbox) };
        worker.send(Message::Job(fnbox))
            .unwrap();
    }
}

fn worker(workqueue: &WorkQueue) {
    let (send, receive) = mpsc::sync_channel(0);
    loop {
        workqueue.push(send.clone());
        let job = receive.recv()
            .unwrap();

        match job {
            Message::Job(fnbox) => fnbox.call_box(),
            Message::NoMoreJobs => break
        };
    }
}

pub fn scoped_pool<F, R>(threads: usize, f: F) -> R where
    F: FnOnce(&Scope) -> R
{
    let mut scope = Scope { 
        available: sync::MsQueue::new(),
        handles: Vec::with_capacity(threads)
    };

    {
        let workqueue = &scope.available;
        for _ in 0..threads {
            let handle;
            unsafe { handle = crossbeam::spawn_unsafe(move || worker(workqueue)); }
            scope.handles.push(handle);
        }
    }

    let result = f(&scope);

    for _ in 0..threads {
        let worker = scope.available.pop();
        worker.send(Message::NoMoreJobs)
            .unwrap();
    }
    for handle in scope.handles { handle.join().unwrap(); }

    result
}
