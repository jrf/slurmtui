use std::process::Command;

pub struct Job {
    pub job_id: String,
    pub name: String,
    pub partition: String,
    pub state: String,
    pub cpus: u32,
    pub memory: String,
    pub elapsed: String,
    pub time_limit: String,
    pub reason_or_nodelist: String,
    pub user: String,
}

pub struct PartitionInfo {
    pub partition: String,
    pub avail: String,
    pub time_limit: String,
    pub nodes: u32,
    pub state: String,
    pub cpus_per_node: u32,
    pub memory_mb: u64,
    pub gres: String,
    pub nodelist: String,
}

pub struct HistoryEntry {
    pub job_id: String,
    pub job_name: String,
    pub partition: String,
    pub state: String,
    pub elapsed: String,
    pub cpu_time: String,
    pub max_rss: String,
    pub exit_code: String,
}

pub struct JobDetail {
    pub fields: Vec<(String, String)>,
}

pub struct SubmitForm {
    pub job_name: String,
    pub script_path: String,
    pub partition: String,
    pub cpus: String,
    pub memory: String,
    pub time_limit: String,
    pub gpu_count: String,
    pub output_file: String,
    pub extra_args: String,
    pub active_field: usize,
    pub editing: bool,
    pub available_partitions: Vec<String>,
}

impl SubmitForm {
    pub fn new() -> Self {
        Self {
            job_name: String::new(),
            script_path: String::new(),
            partition: String::new(),
            cpus: "1".to_string(),
            memory: "4G".to_string(),
            time_limit: "1:00:00".to_string(),
            gpu_count: "0".to_string(),
            output_file: "slurm-%j.out".to_string(),
            extra_args: String::new(),
            active_field: 0,
            editing: false,
            available_partitions: Vec::new(),
        }
    }

    pub const FIELD_COUNT: usize = 9;

    pub fn field_label(&self, index: usize) -> &str {
        match index {
            0 => "Script Path",
            1 => "Job Name",
            2 => "Partition",
            3 => "CPUs",
            4 => "Memory",
            5 => "Time Limit",
            6 => "GPU Count",
            7 => "Output File",
            8 => "Extra Args",
            _ => "",
        }
    }

    pub fn field_value(&self, index: usize) -> &str {
        match index {
            0 => &self.script_path,
            1 => &self.job_name,
            2 => &self.partition,
            3 => &self.cpus,
            4 => &self.memory,
            5 => &self.time_limit,
            6 => &self.gpu_count,
            7 => &self.output_file,
            8 => &self.extra_args,
            _ => "",
        }
    }

    pub fn field_value_mut(&mut self, index: usize) -> Option<&mut String> {
        match index {
            0 => Some(&mut self.script_path),
            1 => Some(&mut self.job_name),
            2 => Some(&mut self.partition),
            3 => Some(&mut self.cpus),
            4 => Some(&mut self.memory),
            5 => Some(&mut self.time_limit),
            6 => Some(&mut self.gpu_count),
            7 => Some(&mut self.output_file),
            8 => Some(&mut self.extra_args),
            _ => None,
        }
    }

    pub fn to_command_string(&self) -> String {
        let mut parts = vec!["sbatch".to_string()];
        if !self.job_name.is_empty() {
            parts.push(format!("--job-name={}", self.job_name));
        }
        if !self.partition.is_empty() {
            parts.push(format!("--partition={}", self.partition));
        }
        if !self.cpus.is_empty() {
            parts.push(format!("--cpus-per-task={}", self.cpus));
        }
        if !self.memory.is_empty() {
            parts.push(format!("--mem={}", self.memory));
        }
        if !self.time_limit.is_empty() {
            parts.push(format!("--time={}", self.time_limit));
        }
        if !self.gpu_count.is_empty() && self.gpu_count != "0" {
            let gpu_type = if self.partition.contains("h100") {
                "h100"
            } else {
                "a100"
            };
            parts.push(format!("--gres=gpu:{}:{}", gpu_type, self.gpu_count));
        }
        if !self.output_file.is_empty() {
            parts.push(format!("--output={}", self.output_file));
        }
        if !self.extra_args.is_empty() {
            parts.push(self.extra_args.clone());
        }
        if !self.script_path.is_empty() {
            parts.push(self.script_path.clone());
        }
        parts.join(" ")
    }
}

fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run {}: {}", cmd, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{} failed: {}", cmd, stderr.trim()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn fetch_jobs(filter_user: Option<&str>) -> Result<Vec<Job>, String> {
    let format_str = "--format=%i|%j|%P|%T|%C|%m|%M|%l|%R|%u".to_string();
    let mut args = vec![format_str.as_str(), "--noheader"];
    let user_flag;
    if let Some(user) = filter_user {
        user_flag = format!("--user={}", user);
        args.push(&user_flag);
    }
    let output = run_command("squeue", &args)?;
    let jobs = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(parse_job_line)
        .collect();
    Ok(jobs)
}

fn parse_job_line(line: &str) -> Option<Job> {
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() < 10 {
        return None;
    }
    Some(Job {
        job_id: parts[0].trim().to_string(),
        name: parts[1].trim().to_string(),
        partition: parts[2].trim().to_string(),
        state: parts[3].trim().to_string(),
        cpus: parts[4].trim().parse().unwrap_or(0),
        memory: parts[5].trim().to_string(),
        elapsed: parts[6].trim().to_string(),
        time_limit: parts[7].trim().to_string(),
        reason_or_nodelist: parts[8].trim().to_string(),
        user: parts[9].trim().to_string(),
    })
}

pub fn fetch_partitions() -> Result<Vec<PartitionInfo>, String> {
    let output = run_command(
        "sinfo",
        &[
            "--format=%20P|%6a|%10l|%5D|%6t|%8c|%10m|%20G|%N",
            "--noheader",
        ],
    )?;
    let partitions = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(parse_partition_line)
        .collect();
    Ok(partitions)
}

fn parse_partition_line(line: &str) -> Option<PartitionInfo> {
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() < 9 {
        return None;
    }
    Some(PartitionInfo {
        partition: parts[0].trim().to_string(),
        avail: parts[1].trim().to_string(),
        time_limit: parts[2].trim().to_string(),
        nodes: parts[3].trim().parse().unwrap_or(0),
        state: parts[4].trim().to_string(),
        cpus_per_node: parts[5].trim().parse().unwrap_or(0),
        memory_mb: parts[6].trim().replace('+', "").parse().unwrap_or(0),
        gres: parts[7].trim().to_string(),
        nodelist: parts[8].trim().to_string(),
    })
}

pub fn fetch_history(user: &str, start_time: &str) -> Result<Vec<HistoryEntry>, String> {
    let user_flag = format!("--user={}", user);
    let start_flag = format!("--starttime={}", start_time);
    let output = run_command(
        "sacct",
        &[
            "--parsable2",
            "--noheader",
            "--format=JobID,JobName%30,Partition%15,State%12,Elapsed,CPUTime,MaxRSS,ExitCode",
            &user_flag,
            &start_flag,
        ],
    )?;
    let entries = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter(|line| {
            let job_id = line.split('|').next().unwrap_or("");
            !job_id.contains('.')
        })
        .filter_map(parse_history_line)
        .collect();
    Ok(entries)
}

fn parse_history_line(line: &str) -> Option<HistoryEntry> {
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() < 8 {
        return None;
    }
    Some(HistoryEntry {
        job_id: parts[0].to_string(),
        job_name: parts[1].to_string(),
        partition: parts[2].to_string(),
        state: parts[3].to_string(),
        elapsed: parts[4].to_string(),
        cpu_time: parts[5].to_string(),
        max_rss: parts[6].to_string(),
        exit_code: parts[7].to_string(),
    })
}

pub fn fetch_job_detail(job_id: &str) -> Result<JobDetail, String> {
    let output = run_command("scontrol", &["show", "job", job_id])?;
    let mut fields = Vec::new();
    for line in output.lines() {
        for token in line.split_whitespace() {
            if let Some(eq_pos) = token.find('=') {
                let key = token[..eq_pos].to_string();
                let value = token[eq_pos + 1..].to_string();
                fields.push((key, value));
            }
        }
    }
    Ok(JobDetail { fields })
}

pub fn cancel_job(job_id: &str) -> Result<(), String> {
    run_command("scancel", &[job_id])?;
    Ok(())
}

pub fn submit_job(form: &SubmitForm) -> Result<String, String> {
    let mut args: Vec<String> = Vec::new();
    if !form.job_name.is_empty() {
        args.push(format!("--job-name={}", form.job_name));
    }
    if !form.partition.is_empty() {
        args.push(format!("--partition={}", form.partition));
    }
    if !form.cpus.is_empty() {
        args.push(format!("--cpus-per-task={}", form.cpus));
    }
    if !form.memory.is_empty() {
        args.push(format!("--mem={}", form.memory));
    }
    if !form.time_limit.is_empty() {
        args.push(format!("--time={}", form.time_limit));
    }
    if !form.gpu_count.is_empty() && form.gpu_count != "0" {
        let gpu_type = if form.partition.contains("h100") {
            "h100"
        } else {
            "a100"
        };
        args.push(format!("--gres=gpu:{}:{}", gpu_type, form.gpu_count));
    }
    if !form.output_file.is_empty() {
        args.push(format!("--output={}", form.output_file));
    }
    if !form.extra_args.is_empty() {
        for arg in form.extra_args.split_whitespace() {
            args.push(arg.to_string());
        }
    }
    if form.script_path.is_empty() {
        return Err("Script path is required".to_string());
    }
    args.push(form.script_path.clone());

    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = run_command("sbatch", &args_refs)?;
    Ok(output.trim().to_string())
}

pub fn fetch_partition_names() -> Result<Vec<String>, String> {
    let output = run_command("sinfo", &["--format=%P", "--noheader"])?;
    let mut names: Vec<String> = output
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    names.sort();
    names.dedup();
    Ok(names)
}
