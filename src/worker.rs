use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

use crate::slurm::{self, HistoryEntry, Job, PartitionInfo};

#[allow(clippy::enum_variant_names)]
pub enum Request {
    FetchJobs { seq: u64, filter_user: Option<String> },
    FetchPartitions { seq: u64 },
    FetchHistory { seq: u64, user: String, start: String },
    FetchPartitionNames { seq: u64 },
}

pub enum Response {
    Jobs { seq: u64, result: Result<Vec<Job>, String> },
    Partitions { seq: u64, result: Result<Vec<PartitionInfo>, String> },
    History { seq: u64, result: Result<Vec<HistoryEntry>, String> },
    PartitionNames { seq: u64, result: Result<Vec<String>, String> },
}

pub struct Worker {
    jobs_tx: Sender<Request>,
    partitions_tx: Sender<Request>,
    history_tx: Sender<Request>,
    partition_names_tx: Sender<Request>,
    rx: Receiver<Response>,
}

impl Worker {
    pub fn spawn() -> Self {
        let (resp_tx, resp_rx) = channel::<Response>();

        let jobs_tx = spawn_worker(resp_tx.clone());
        let partitions_tx = spawn_worker(resp_tx.clone());
        let history_tx = spawn_worker(resp_tx.clone());
        let partition_names_tx = spawn_worker(resp_tx);

        Self {
            jobs_tx,
            partitions_tx,
            history_tx,
            partition_names_tx,
            rx: resp_rx,
        }
    }

    pub fn send(&self, req: Request) {
        let _ = match &req {
            Request::FetchJobs { .. } => self.jobs_tx.send(req),
            Request::FetchPartitions { .. } => self.partitions_tx.send(req),
            Request::FetchHistory { .. } => self.history_tx.send(req),
            Request::FetchPartitionNames { .. } => self.partition_names_tx.send(req),
        };
    }

    pub fn try_recv(&self) -> Option<Response> {
        self.rx.try_recv().ok()
    }
}

fn spawn_worker(tx: Sender<Response>) -> Sender<Request> {
    let (req_tx, req_rx) = channel::<Request>();
    thread::spawn(move || run(req_rx, tx));
    req_tx
}

fn run(rx: Receiver<Request>, tx: Sender<Response>) {
    while let Ok(req) = rx.recv() {
        let resp = match req {
            Request::FetchJobs { seq, filter_user } => Response::Jobs {
                seq,
                result: slurm::fetch_jobs(filter_user.as_deref()),
            },
            Request::FetchPartitions { seq } => Response::Partitions {
                seq,
                result: slurm::fetch_partitions(),
            },
            Request::FetchHistory { seq, user, start } => Response::History {
                seq,
                result: slurm::fetch_history(&user, &start),
            },
            Request::FetchPartitionNames { seq } => Response::PartitionNames {
                seq,
                result: slurm::fetch_partition_names(),
            },
        };
        if tx.send(resp).is_err() {
            break;
        }
    }
}
