use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

use crate::slurm::{
    self, HistoryEntry, Job, JobAction, JobDetail, LogKind, LogTail, PartitionInfo, SubmitForm,
};

#[allow(clippy::enum_variant_names)]
pub enum Request {
    FetchJobs { seq: u64, filter_user: Option<String> },
    FetchPartitions { seq: u64 },
    FetchHistory { seq: u64, user: String, start: String },
    FetchPartitionNames { seq: u64 },
    FetchJobDetail {
        seq: u64,
        job_id: String,
    },
    FetchLog {
        seq: u64,
        job_id: String,
        kind: LogKind,
        max_lines: usize,
        max_bytes: u64,
    },
    CancelJob {
        seq: u64,
        job_id: String,
    },
    ExecuteJobAction {
        seq: u64,
        job_id: String,
        action: JobAction,
    },
    SubmitJob {
        seq: u64,
        form: Box<SubmitForm>,
    },
}

pub enum Response {
    Jobs { seq: u64, result: Result<Vec<Job>, String> },
    Partitions { seq: u64, result: Result<Vec<PartitionInfo>, String> },
    History { seq: u64, result: Result<Vec<HistoryEntry>, String> },
    PartitionNames { seq: u64, result: Result<Vec<String>, String> },
    JobDetail {
        seq: u64,
        result: Result<JobDetail, String>,
    },
    Log {
        seq: u64,
        job_id: String,
        kind: LogKind,
        result: Result<LogTail, String>,
    },
    CancelJob {
        seq: u64,
        job_id: String,
        result: Result<(), String>,
    },
    JobAction {
        seq: u64,
        job_id: String,
        action: JobAction,
        result: Result<(), String>,
    },
    SubmitJob {
        seq: u64,
        result: Result<String, String>,
    },
}

pub struct Worker {
    jobs_tx: Sender<Request>,
    partitions_tx: Sender<Request>,
    history_tx: Sender<Request>,
    partition_names_tx: Sender<Request>,
    job_detail_tx: Sender<Request>,
    log_tx: Sender<Request>,
    cancel_tx: Sender<Request>,
    job_action_tx: Sender<Request>,
    submit_tx: Sender<Request>,
    rx: Receiver<Response>,
}

impl Worker {
    pub fn spawn() -> Self {
        let (resp_tx, resp_rx) = channel::<Response>();

        let jobs_tx = spawn_worker(resp_tx.clone());
        let partitions_tx = spawn_worker(resp_tx.clone());
        let history_tx = spawn_worker(resp_tx.clone());
        let partition_names_tx = spawn_worker(resp_tx.clone());
        let job_detail_tx = spawn_worker(resp_tx.clone());
        let log_tx = spawn_worker(resp_tx.clone());
        let cancel_tx = spawn_worker(resp_tx.clone());
        let job_action_tx = spawn_worker(resp_tx.clone());
        let submit_tx = spawn_worker(resp_tx);

        Self {
            jobs_tx,
            partitions_tx,
            history_tx,
            partition_names_tx,
            job_detail_tx,
            log_tx,
            cancel_tx,
            job_action_tx,
            submit_tx,
            rx: resp_rx,
        }
    }

    pub fn send(&self, req: Request) {
        let _ = match &req {
            Request::FetchJobs { .. } => self.jobs_tx.send(req),
            Request::FetchPartitions { .. } => self.partitions_tx.send(req),
            Request::FetchHistory { .. } => self.history_tx.send(req),
            Request::FetchPartitionNames { .. } => self.partition_names_tx.send(req),
            Request::FetchJobDetail { .. } => self.job_detail_tx.send(req),
            Request::FetchLog { .. } => self.log_tx.send(req),
            Request::CancelJob { .. } => self.cancel_tx.send(req),
            Request::ExecuteJobAction { .. } => self.job_action_tx.send(req),
            Request::SubmitJob { .. } => self.submit_tx.send(req),
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
            Request::FetchJobDetail { seq, job_id } => Response::JobDetail {
                seq,
                result: slurm::fetch_job_detail(&job_id),
            },
            Request::FetchLog {
                seq,
                job_id,
                kind,
                max_lines,
                max_bytes,
            } => Response::Log {
                seq,
                result: slurm::fetch_log_tail(&job_id, kind, max_lines, max_bytes),
                job_id,
                kind,
            },
            Request::CancelJob { seq, job_id } => Response::CancelJob {
                seq,
                result: slurm::cancel_job(&job_id),
                job_id,
            },
            Request::ExecuteJobAction {
                seq,
                job_id,
                action,
            } => Response::JobAction {
                seq,
                result: slurm::execute_job_action(&job_id, action),
                job_id,
                action,
            },
            Request::SubmitJob { seq, form } => Response::SubmitJob {
                seq,
                result: slurm::submit_job(&form),
            },
        };
        if tx.send(resp).is_err() {
            break;
        }
    }
}
