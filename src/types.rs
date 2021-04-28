use crossbeam::channel::{bounded, Receiver, Sender};
use rand::Rng;
use std::{thread, time::Duration};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Follower,
    Candidate,
    Leader,
    Dead,
}

pub enum ControlMessage {
    Up,
    Down,
    Apply(i32),
    Disconnect,
    Connect,
}

#[derive(Debug)]
pub struct ReplicaStatus {
    pub id: usize,
    pub state: State,
    pub connected: bool,
    pub value: i32,
    pub term: usize,
    pub commit_index: usize,
    pub last_applied: usize,
    pub log: Vec<Log>,
}

#[derive(Clone, Debug)]
pub enum Message {
    AppendEntryRequest {
        from_id: usize,
        term: usize,
        prev_log_index: usize,
        prev_log_term: usize,
        entries: Vec<Log>,
        commit_index: usize,
    },
    AppendEntryResponse {
        from_id: usize,
        term: usize,
        success: bool,
        last_index: usize,
    },
    RequestVoteRequest {
        from_id: usize,
        term: usize,
        last_log_index: usize,
        last_log_term: usize,
    },
    RequestVoteResponse {
        from_id: usize,
        term: usize,
        vote_granted: bool,
    },
}

#[derive(Clone, Debug)]
pub struct Peer {
    pub id: usize,
    tx: Sender<Message>,
    drop_prob: usize,
}

impl Peer {
    pub fn new(id: usize, tx: Sender<Message>, percent_probability_message_drop: usize) -> Peer {
        Peer {
            id: id,
            tx: tx,
            drop_prob: percent_probability_message_drop,
        }
    }

    pub fn send(&self, message: Message) {
        let mut rng = rand::thread_rng();
        let val = rng.gen_range(0..=100);
        if val >= self.drop_prob {
            self.tx.send(message).unwrap();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Log {
    pub index: usize,
    pub delta: i32,
    pub term: usize,
}

pub struct HeartbeatTimer {
    timeout: Duration,
    rx: Receiver<()>,
}

impl HeartbeatTimer {
    pub fn new(timeout: Duration) -> HeartbeatTimer {
        let (tx, rx) = bounded(1);

        thread::spawn(move || {
            thread::sleep(timeout);
            tx.send(()).unwrap();
        });

        HeartbeatTimer {
            timeout: timeout,
            rx: rx,
        }
    }

    pub fn renew(&mut self) {
        let (tx, rx) = bounded(1);
        let timeout = self.timeout;
        thread::spawn(move || {
            thread::sleep(timeout);
            tx.send(()).unwrap();
        });

        self.rx = rx;
    }

    pub fn fired(&mut self) -> bool {
        match self.rx.try_recv() {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
