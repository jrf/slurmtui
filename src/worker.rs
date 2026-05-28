use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::thread::{self, JoinHandle};

use crate::slurm::{self, HistoryEntry, Job, PartitionInfo};

pub enum Request {
    FetchJobs { seq: u64, filter_user: Option<String> },
    FetchPartitions { seq: u64 },
    FetchHistory { seq: u64, user: String, start: String },
    FetchPartitionNames { seq: u64 },
    Shutdown,
}

pub enum Response {
    Jobs { seq: u64, result: Result<Vec<Job>, String> },
    Partitions { seq: u64, result: Result<Vec<PartitionInfo>, String> },
    History { seq: u64, result: Result<Vec<HistoryEntry>, String> },
    PartitionNames { seq: u64, result: Result<Vec<String>, String> },
}

pub struct Worker {
    tx: Sender<Request>,
    rx: Receiver<Response>,
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn spawn() -> Self {
        let (req_tx, req_rx) = channel::<Request>();
        let (resp_tx, resp_rx) = channel::<Response>();

        let handle = thread::spawn(move || run(req_rx, resp_tx));

        Self {
            tx: req_tx,
            rx: resp_rx,
            handle: Some(handle),
        }
    }

    pub fn send(&self, req: Request) {
        let _ = self.tx.send(req);
    }

    pub fn try_recv(&self) -> Option<Response> {
        match self.rx.try_recv() {
            Ok(r) => Some(r),
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => None,
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.tx.send(Request::Shutdown);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

fn run(rx: Receiver<Request>, tx: Sender<Response>) {
    while let Ok(req) = rx.recv() {
        match req {
            Request::Shutdown => break,
            Request::FetchJobs { seq, filter_user } => {
                let result = slurm::fetch_jobs(filter_user.as_deref());
                let _ = tx.send(Response::Jobs { seq, result });
            }
            Request::FetchPartitions { seq } => {
                let result = slurm::fetch_partitions();
                let _ = tx.send(Response::Partitions { seq, result });
            }
            Request::FetchHistory { seq, user, start } => {
                let result = slurm::fetch_history(&user, &start);
                let _ = tx.send(Response::History { seq, result });
            }
            Request::FetchPartitionNames { seq } => {
                let result = slurm::fetch_partition_names();
                let _ = tx.send(Response::PartitionNames { seq, result });
            }
        }
    }
}
