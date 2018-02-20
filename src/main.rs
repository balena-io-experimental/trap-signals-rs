extern crate nix;

use std::thread;
use std::time;
use std::sync::mpsc::{channel, Sender};

use nix::sys::signal::{SigSet, SIGINT, SIGQUIT, SIGTERM, SIGHUP};

#[derive(Debug)]
enum Event {
    Working(usize),
    Completed,
    Terminate,
}

fn process(event_tx: &Sender<Event>) {
    let mut count = 0;

    loop {
        count += 1;
        event_tx.send(Event::Working(count)).unwrap();
        thread::sleep(time::Duration::from_secs(1));

        if count == 10 {
            event_tx.send(Event::Completed).unwrap();
            return;
        }
    }
}

fn trap(event_tx: &Sender<Event>) {
    trap_signals();

    event_tx.send(Event::Terminate).unwrap();
}

fn trap_signals() {
    let mut mask = SigSet::empty();

    mask.add(SIGINT);
    mask.add(SIGQUIT);
    mask.add(SIGTERM);
    mask.add(SIGHUP);
    mask.thread_block().unwrap();

    let sig = mask.wait().unwrap();
    
    println!("Received SIG {:?}", sig);    
}

fn main() {
    let (event_tx, event_rx) = channel();
    let event_tx_clone = event_tx.clone();
    
    thread::spawn(move || {
        process(&event_tx_clone);
    });

    thread::spawn(move || {
        trap(&event_tx);
    });

    loop {
        match event_rx.recv() {
            Ok(event) => {
                match event {
                    Event::Working(count) => println!("Working: {}", count),
                    Event::Completed => {
                        println!("Completed");
                        return;
                    },
                    Event::Terminate => {
                        println!("Terminated");
                        return;
                    },
                }
            },
            Err(e) => {
                panic!("Error: {:?}", e);
            },
        }
        
    }
}
