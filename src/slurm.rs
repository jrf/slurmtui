use std::fs;
use std::path::Path;
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
    pub num_nodes: u32,
    pub gpus_per_node: u32,
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
    pub nodes: String,
    pub cpus: String,
    pub memory: String,
    pub time_limit: String,
    pub gpu_count: String,
    pub output_file: String,
    pub error_file: String,
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
            nodes: String::new(),
            cpus: String::new(),
            memory: String::new(),
            time_limit: String::new(),
            gpu_count: String::new(),
            output_file: String::new(),
            error_file: String::new(),
            extra_args: String::new(),
            active_field: 0,
            editing: false,
            available_partitions: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.job_name.clear();
        self.script_path.clear();
        self.partition.clear();
        self.nodes.clear();
        self.cpus.clear();
        self.memory.clear();
        self.time_limit.clear();
        self.gpu_count.clear();
        self.output_file.clear();
        self.error_file.clear();
        self.extra_args.clear();
        self.active_field = 0;
        self.editing = false;
    }

    pub const FIELD_COUNT: usize = 11;

    pub fn field_label(&self, index: usize) -> &str {
        match index {
            0 => "Script Path",
            1 => "Job Name",
            2 => "Partition",
            3 => "Nodes",
            4 => "CPUs",
            5 => "Memory",
            6 => "Time Limit",
            7 => "GPU Count",
            8 => "Output File",
            9 => "Error File",
            10 => "Extra Args",
            _ => "",
        }
    }

    pub fn field_value(&self, index: usize) -> &str {
        match index {
            0 => &self.script_path,
            1 => &self.job_name,
            2 => &self.partition,
            3 => &self.nodes,
            4 => &self.cpus,
            5 => &self.memory,
            6 => &self.time_limit,
            7 => &self.gpu_count,
            8 => &self.output_file,
            9 => &self.error_file,
            10 => &self.extra_args,
            _ => "",
        }
    }

    pub fn field_value_mut(&mut self, index: usize) -> Option<&mut String> {
        match index {
            0 => Some(&mut self.script_path),
            1 => Some(&mut self.job_name),
            2 => Some(&mut self.partition),
            3 => Some(&mut self.nodes),
            4 => Some(&mut self.cpus),
            5 => Some(&mut self.memory),
            6 => Some(&mut self.time_limit),
            7 => Some(&mut self.gpu_count),
            8 => Some(&mut self.output_file),
            9 => Some(&mut self.error_file),
            10 => Some(&mut self.extra_args),
            _ => None,
        }
    }

    pub fn apply_directives(&mut self, d: &ParsedDirectives) {
        if let Some(v) = &d.job_name {
            self.job_name = v.clone();
        }
        if let Some(v) = &d.partition {
            self.partition = v.clone();
        }
        if let Some(v) = &d.nodes {
            self.nodes = v.clone();
        }
        if let Some(v) = &d.cpus {
            self.cpus = v.clone();
        }
        if let Some(v) = &d.memory {
            self.memory = v.clone();
        }
        if let Some(v) = &d.time_limit {
            self.time_limit = v.clone();
        }
        if let Some(v) = &d.gpu_count {
            self.gpu_count = v.clone();
        }
        if let Some(v) = &d.output_file {
            self.output_file = v.clone();
        }
        if let Some(v) = &d.error_file {
            self.error_file = v.clone();
        }
        if !d.extras.is_empty() {
            let joined = d.extras.join(" ");
            self.extra_args = if self.extra_args.is_empty() {
                joined
            } else {
                format!("{} {}", self.extra_args, joined)
            };
        }
    }

    pub fn to_command_string(&self) -> String {
        let mut parts = vec!["sbatch".to_string()];
        for arg in build_flag_args(self) {
            parts.push(shell_quote(&arg));
        }
        if !self.script_path.is_empty() {
            parts.push(shell_quote(&self.script_path));
        }
        parts.join(" ")
    }
}

/// Build the `sbatch` flag arguments (everything except the leading `sbatch`
/// and the trailing script path) from a form.
///
/// This is the single source of truth shared by the live command preview
/// (`SubmitForm::to_command_string`) and the actual submission (`submit_job`),
/// so the two can never drift apart.
fn build_flag_args(form: &SubmitForm) -> Vec<String> {
    let partition = form.partition.trim_end_matches('*');
    let mut args: Vec<String> = Vec::new();
    if !form.job_name.is_empty() {
        args.push(format!("--job-name={}", form.job_name));
    }
    if !partition.is_empty() {
        args.push(format!("--partition={}", partition));
    }
    if !form.nodes.is_empty() {
        args.push(format!("--nodes={}", form.nodes));
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
        args.push(format!("--gres=gpu:{}", form.gpu_count));
    }
    if !form.output_file.is_empty() {
        args.push(format!("--output={}", form.output_file));
    }
    if !form.error_file.is_empty() {
        args.push(format!("--error={}", form.error_file));
    }
    if !form.extra_args.is_empty() {
        args.extend(tokenize_args(&form.extra_args));
    }
    args
}

/// Split a free-form extra-args string into individual argv tokens, honoring
/// single quotes, double quotes and backslash escapes so that values
/// containing spaces (e.g. `--comment="my job"`) survive as a single argument.
fn tokenize_args(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut has_token = false;
    let mut in_single = false;
    let mut in_double = false;
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' if !in_single => {
                if let Some(next) = chars.next() {
                    cur.push(next);
                    has_token = true;
                }
            }
            '\'' if !in_double => {
                in_single = !in_single;
                has_token = true;
            }
            '"' if !in_single => {
                in_double = !in_double;
                has_token = true;
            }
            c if c.is_whitespace() && !in_single && !in_double => {
                if has_token {
                    tokens.push(std::mem::take(&mut cur));
                    has_token = false;
                }
            }
            c => {
                cur.push(c);
                has_token = true;
            }
        }
    }
    if has_token {
        tokens.push(cur);
    }
    tokens
}

/// Quote a single argument for safe, copy-pasteable display in the command
/// preview. Values containing shell-significant characters are wrapped in
/// single quotes; simple values are left untouched.
fn shell_quote(arg: &str) -> String {
    if arg.is_empty() {
        return "''".to_string();
    }
    let safe = arg
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "-_=/.:,+@%".contains(c));
    if safe {
        return arg.to_string();
    }
    let mut out = String::with_capacity(arg.len() + 2);
    out.push('\'');
    for c in arg.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
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
    let format_str = "--format=%i|%j|%P|%T|%C|%m|%M|%l|%R|%u|%D|%b".to_string();
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
    let num_nodes = parts.get(10).map(|s| s.trim().parse().unwrap_or(0)).unwrap_or(0);
    let gpus_per_node = parts
        .get(11)
        .map(|s| parse_gpus_per_node(s.trim()))
        .unwrap_or(0);
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
        num_nodes,
        gpus_per_node,
    })
}

/// Parse the trailing integer GPU count from a squeue %b (tres_per_node) field.
///
/// Examples:
///   "gres:gpu:h200:2" -> 2
///   "gres:gpu:2"      -> 2
///   "gres/gpu:1"      -> 1
///   "N/A" / ""        -> 0
/// Multiple gres entries are comma-separated; we sum the GPU ones.
fn parse_gpus_per_node(field: &str) -> u32 {
    if field.is_empty() || field == "N/A" || field == "(null)" {
        return 0;
    }
    let mut total: u32 = 0;
    for entry in field.split(',') {
        let entry = entry.trim();
        let lower = entry.to_ascii_lowercase();
        if !(lower.starts_with("gres:gpu") || lower.starts_with("gres/gpu") || lower.starts_with("gpu:")) {
            continue;
        }
        if let Some(tail) = entry.rsplit(':').next() {
            // Trim Slurm's optional "(IDX:...)" suffix if present.
            let tail = tail.split('(').next().unwrap_or(tail);
            if let Ok(n) = tail.parse::<u32>() {
                total = total.saturating_add(n);
            }
        }
    }
    total
}

pub fn fetch_partitions() -> Result<Vec<PartitionInfo>, String> {
    let output = run_command(
        "sinfo",
        &[
            "--format=%P|%a|%l|%D|%t|%c|%m|%G|%N",
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
            "--format=JobID,JobName,Partition,State,Elapsed,CPUTime,MaxRSS,ExitCode",
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LogKind {
    StdOut,
    StdErr,
}

impl LogKind {
    pub fn label(self) -> &'static str {
        match self {
            LogKind::StdOut => "stdout",
            LogKind::StdErr => "stderr",
        }
    }

    pub fn flip(self) -> Self {
        match self {
            LogKind::StdOut => LogKind::StdErr,
            LogKind::StdErr => LogKind::StdOut,
        }
    }
}

pub fn fetch_log_path(job_id: &str, kind: LogKind) -> Result<String, String> {
    let key = match kind {
        LogKind::StdOut => "StdOut",
        LogKind::StdErr => "StdErr",
    };
    if let Ok(detail) = fetch_job_detail(job_id) {
        for (k, v) in &detail.fields {
            if k == key && !v.is_empty() && v != "(null)" {
                return Ok(v.clone());
            }
        }
    }
    let format_arg = format!("--format=JobID,{}", key);
    let job_arg = format!("--jobs={}", job_id);
    let output = run_command(
        "sacct",
        &["--parsable2", "--noheader", &format_arg, &job_arg],
    )?;
    for line in output.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 2 {
            continue;
        }
        let id = parts[0];
        if id.contains('.') {
            continue;
        }
        let path = parts[1].trim();
        if !path.is_empty() && path != "(null)" {
            return Ok(path.to_string());
        }
    }
    Err(format!("no {} path found for job {}", key, job_id))
}

pub fn read_log_tail(path: &Path, max_lines: usize, max_bytes: u64) -> Result<String, String> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(path)
        .map_err(|e| format!("open {}: {}", path.display(), e))?;
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let start = len.saturating_sub(max_bytes);
    if start > 0 {
        file.seek(SeekFrom::Start(start))
            .map_err(|e| format!("seek: {}", e))?;
    }
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|e| format!("read: {}", e))?;
    let text = String::from_utf8_lossy(&buf).into_owned();
    let trimmed = if start > 0 {
        if let Some(nl) = text.find('\n') {
            &text[nl + 1..]
        } else {
            &text
        }
    } else {
        &text
    };
    let cleaned: Vec<String> = trimmed.lines().map(clean_log_line).collect();
    let start_idx = cleaned.len().saturating_sub(max_lines);
    Ok(cleaned[start_idx..].join("\n"))
}

fn clean_log_line(line: &str) -> String {
    let after_cr = line.rsplit('\r').next().unwrap_or("");
    strip_ansi(after_cr)
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\x1b' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('[') => {
                // CSI: consume until a final byte (0x40-0x7E)
                while let Some(&ch) = chars.peek() {
                    chars.next();
                    if ('@'..='~').contains(&ch) {
                        break;
                    }
                }
            }
            Some(']') => {
                // OSC: consume until BEL or ST (ESC \)
                while let Some(ch) = chars.next() {
                    if ch == '\x07' {
                        break;
                    }
                    if ch == '\x1b' {
                        if let Some(&'\\') = chars.peek() {
                            chars.next();
                        }
                        break;
                    }
                }
            }
            Some(_) => {
                // Other 2-byte escapes (e.g. ESC =, ESC >). Already consumed.
            }
            None => {}
        }
    }
    out
}

pub fn cancel_job(job_id: &str) -> Result<(), String> {
    run_command("scancel", &[job_id])?;
    Ok(())
}

pub fn submit_job(form: &SubmitForm) -> Result<String, String> {
    if form.script_path.is_empty() {
        return Err("Script path is required".to_string());
    }
    let mut args = build_flag_args(form);
    args.push(form.script_path.clone());

    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = run_command("sbatch", &args_refs)?;
    Ok(output.trim().to_string())
}

#[derive(Default)]
pub struct ParsedDirectives {
    pub job_name: Option<String>,
    pub partition: Option<String>,
    pub nodes: Option<String>,
    pub cpus: Option<String>,
    pub memory: Option<String>,
    pub time_limit: Option<String>,
    pub gpu_count: Option<String>,
    pub output_file: Option<String>,
    pub error_file: Option<String>,
    pub extras: Vec<String>,
    pub count: usize,
}

pub fn parse_sbatch_directives(path: &Path) -> Result<ParsedDirectives, String> {
    let contents = fs::read_to_string(path)
        .map_err(|e| format!("read {}: {}", path.display(), e))?;
    let mut out = ParsedDirectives::default();
    for raw in contents.lines() {
        let line = raw.trim_start();
        if line.is_empty() {
            continue;
        }
        if !line.starts_with('#') {
            break;
        }
        let after_hash = line.trim_start_matches('#').trim_start();
        if !after_hash.starts_with("SBATCH") {
            continue;
        }
        let args = after_hash["SBATCH".len()..].trim();
        if args.is_empty() {
            continue;
        }
        parse_directive_args(args, &mut out);
    }
    Ok(out)
}

fn parse_directive_args(args: &str, out: &mut ParsedDirectives) {
    let tokens: Vec<&str> = args.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let tok = tokens[i];
        if let Some(rest) = tok.strip_prefix("--") {
            if let Some((key, value)) = rest.split_once('=') {
                apply_long(key, value, out);
                i += 1;
                continue;
            }
            if i + 1 < tokens.len() {
                apply_long(rest, tokens[i + 1], out);
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if let Some(rest) = tok.strip_prefix('-') {
            if rest.len() == 1 {
                if i + 1 < tokens.len() {
                    apply_short(rest, tokens[i + 1], out);
                    i += 2;
                    continue;
                }
                i += 1;
                continue;
            }
            let (flag, value) = rest.split_at(1);
            apply_short(flag, value, out);
            i += 1;
            continue;
        }
        out.extras.push(tok.to_string());
        i += 1;
    }
}

fn strip_quotes(v: &str) -> &str {
    let t = v.trim();
    if t.len() >= 2 {
        let bytes = t.as_bytes();
        let first = bytes[0];
        let last = bytes[t.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return &t[1..t.len() - 1];
        }
    }
    t
}

fn apply_long(key: &str, value: &str, out: &mut ParsedDirectives) {
    let v = strip_quotes(value);
    match key {
        "job-name" => {
            out.job_name = Some(v.to_string());
            out.count += 1;
        }
        "partition" => {
            out.partition = Some(v.to_string());
            out.count += 1;
        }
        "nodes" => {
            out.nodes = Some(v.to_string());
            out.count += 1;
        }
        "cpus-per-task" => {
            out.cpus = Some(v.to_string());
            out.count += 1;
        }
        "mem" => {
            out.memory = Some(v.to_string());
            out.count += 1;
        }
        "time" => {
            out.time_limit = Some(v.to_string());
            out.count += 1;
        }
        "gres" => {
            for piece in v.split(',') {
                if let Some(rest) = piece.strip_prefix("gpu:") {
                    let count_str = rest.rsplit(':').next().unwrap_or("");
                    if !count_str.is_empty() && count_str.chars().all(|c| c.is_ascii_digit()) {
                        out.gpu_count = Some(count_str.to_string());
                        out.count += 1;
                    }
                }
            }
        }
        "output" => {
            out.output_file = Some(v.to_string());
            out.count += 1;
        }
        "error" => {
            out.error_file = Some(v.to_string());
            out.count += 1;
        }
        _ => {
            out.extras.push(format!("--{}={}", key, v));
        }
    }
}

fn apply_short(key: &str, value: &str, out: &mut ParsedDirectives) {
    let v = strip_quotes(value);
    match key {
        "J" => {
            out.job_name = Some(v.to_string());
            out.count += 1;
        }
        "p" => {
            out.partition = Some(v.to_string());
            out.count += 1;
        }
        "N" => {
            out.nodes = Some(v.to_string());
            out.count += 1;
        }
        "c" => {
            out.cpus = Some(v.to_string());
            out.count += 1;
        }
        "t" => {
            out.time_limit = Some(v.to_string());
            out.count += 1;
        }
        "o" => {
            out.output_file = Some(v.to_string());
            out.count += 1;
        }
        "e" => {
            out.error_file = Some(v.to_string());
            out.count += 1;
        }
        _ => {
            out.extras.push(format!("-{} {}", key, v));
        }
    }
}

pub fn fetch_partition_names() -> Result<Vec<String>, String> {
    let output = run_command("sinfo", &["--format=%P", "--noheader"])?;
    let mut names: Vec<String> = output
        .lines()
        .map(|l| l.trim().trim_end_matches('*').to_string())
        .filter(|l| !l.is_empty())
        .collect();
    names.sort();
    names.dedup();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn gpus_per_node_variants() {
        assert_eq!(parse_gpus_per_node("gres:gpu:h200:2"), 2);
        assert_eq!(parse_gpus_per_node("gres:gpu:2"), 2);
        assert_eq!(parse_gpus_per_node("gres/gpu:1"), 1);
        assert_eq!(parse_gpus_per_node("gpu:4"), 4);
        assert_eq!(parse_gpus_per_node("gres:gpu:1,gres:gpu:3"), 4);
        assert_eq!(parse_gpus_per_node("gres:mps:100"), 0);
        assert_eq!(parse_gpus_per_node("N/A"), 0);
        assert_eq!(parse_gpus_per_node("(null)"), 0);
        assert_eq!(parse_gpus_per_node(""), 0);
    }

    #[test]
    fn tokenize_handles_quotes_and_escapes() {
        assert_eq!(tokenize_args("--foo bar"), v(&["--foo", "bar"]));
        assert_eq!(tokenize_args("--comment=\"my job\""), v(&["--comment=my job"]));
        assert_eq!(tokenize_args("--a 'b c' --d"), v(&["--a", "b c", "--d"]));
        assert_eq!(tokenize_args("a\\ b"), v(&["a b"]));
        assert_eq!(tokenize_args("   "), v(&[]));
        assert_eq!(tokenize_args(""), v(&[]));
    }

    #[test]
    fn shell_quote_quotes_when_needed() {
        assert_eq!(shell_quote("simple"), "simple");
        assert_eq!(shell_quote("--mem=4G"), "--mem=4G");
        assert_eq!(shell_quote("my job"), "'my job'");
        assert_eq!(shell_quote(""), "''");
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
    }

    #[test]
    fn flag_args_and_preview_stay_in_sync() {
        let mut form = SubmitForm::new();
        form.job_name = "job1".to_string();
        form.partition = "gpu*".to_string();
        form.gpu_count = "2".to_string();
        form.extra_args = "--comment=\"hello world\"".to_string();
        form.script_path = "run.sh".to_string();

        assert_eq!(
            build_flag_args(&form),
            v(&[
                "--job-name=job1",
                "--partition=gpu",
                "--gres=gpu:2",
                "--comment=hello world",
            ])
        );
        assert_eq!(
            form.to_command_string(),
            "sbatch --job-name=job1 --partition=gpu --gres=gpu:2 '--comment=hello world' run.sh"
        );
    }

    #[test]
    fn gpu_count_zero_is_omitted() {
        let mut form = SubmitForm::new();
        form.gpu_count = "0".to_string();
        form.script_path = "r.sh".to_string();
        assert_eq!(form.to_command_string(), "sbatch r.sh");
    }

    #[test]
    fn parse_job_line_ok_and_reject_short() {
        let line = "12345|myjob|gpu|RUNNING|4|16G|01:00|1-00:00:00|node01|alice|1|gres:gpu:2";
        let job = parse_job_line(line).expect("should parse");
        assert_eq!(job.job_id, "12345");
        assert_eq!(job.name, "myjob");
        assert_eq!(job.cpus, 4);
        assert_eq!(job.num_nodes, 1);
        assert_eq!(job.gpus_per_node, 2);
        assert!(parse_job_line("only|three|fields").is_none());
    }

    #[test]
    fn parse_history_line_ok_and_reject_short() {
        let line = "123|job|part|COMPLETED|00:10:00|00:40:00|1024K|0:0";
        let e = parse_history_line(line).expect("should parse");
        assert_eq!(e.job_id, "123");
        assert_eq!(e.state, "COMPLETED");
        assert!(parse_history_line("1|2|3").is_none());
    }
}
