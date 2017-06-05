use crossbeam::{self, sync};

use std::thread;
use std::sync::mpsc;

enum Message<'a> {
    Job(Box<FnBox + 'a + Send>),
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

type WorkQueue<'a> = sync::MsQueue<mpsc::SyncSender<Message<'a>>>;

pub struct Scope<'a> {
    available: WorkQueue<'a>,
    handles: Vec<thread::JoinHandle<()>>
}

impl<'a> Scope<'a> {
    pub fn spawn<F>(&self, f: F) where
        F: FnOnce() + 'a + Send
    {
        let worker = self.available.pop();
        let fnbox: Box<FnBox + 'a + Send> = Box::new(f);
        worker.send(Message::Job(fnbox))
            .unwrap();
    }
}

fn worker<'a>(workqueue: &WorkQueue<'a>) {
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

pub fn scoped_pool<'a, F, R>(threads: usize, f: F) -> R where
    F: FnOnce(&Scope<'a>) -> R
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
