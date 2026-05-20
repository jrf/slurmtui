use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use ratatui::widgets::TableState;

use crate::slurm::{self, HistoryEntry, Job, JobDetail, PartitionInfo, SubmitForm};

const WALK_CAP: usize = 10_000;
const MATCH_LIMIT: usize = 500;

pub struct PickerEntry {
    pub name: String,
    pub is_dir: bool,
}

pub struct MatchEntry {
    pub path: PathBuf,
    pub display: String,
}

pub struct FilePicker {
    pub current_dir: PathBuf,
    pub entries: Vec<PickerEntry>,
    pub selected: usize,
    pub show_all: bool,
    pub query_active: bool,
    pub query: String,
    pub matches: Vec<MatchEntry>,
    all_files: Vec<PathBuf>,
    all_displays: Vec<String>,
    walked: bool,
    matcher: Matcher,
}

impl FilePicker {
    pub fn new(start: PathBuf) -> Self {
        let mut p = Self {
            current_dir: start,
            entries: Vec::new(),
            selected: 0,
            show_all: false,
            query_active: false,
            query: String::new(),
            matches: Vec::new(),
            all_files: Vec::new(),
            all_displays: Vec::new(),
            walked: false,
            matcher: Matcher::new(Config::DEFAULT.match_paths()),
        };
        p.reload();
        p
    }

    pub fn reload(&mut self) {
        self.entries.clear();
        self.selected = 0;
        let read = match std::fs::read_dir(&self.current_dir) {
            Ok(r) => r,
            Err(_) => return,
        };
        for ent in read.flatten() {
            let name = ent.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            let is_dir = ent.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if !is_dir && !self.show_all && !looks_like_script(&name) {
                continue;
            }
            self.entries.push(PickerEntry { name, is_dir });
        }
        self.entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });
        self.invalidate_walk();
    }

    fn invalidate_walk(&mut self) {
        self.all_files.clear();
        self.all_displays.clear();
        self.walked = false;
        self.matches.clear();
    }

    pub fn enter_selected(&mut self) -> Option<PathBuf> {
        if self.query_active && !self.query.is_empty() {
            let m = self.matches.get(self.selected)?;
            return Some(m.path.clone());
        }
        let ent = self.entries.get(self.selected)?;
        let target = self.current_dir.join(&ent.name);
        if ent.is_dir {
            self.current_dir = std::fs::canonicalize(&target).unwrap_or(target);
            self.reload();
            None
        } else {
            Some(target)
        }
    }

    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            let parent = parent.to_path_buf();
            self.current_dir = std::fs::canonicalize(&parent).unwrap_or(parent);
            self.reload();
        }
    }

    fn visible_len(&self) -> usize {
        if self.query_active && !self.query.is_empty() {
            self.matches.len()
        } else {
            self.entries.len()
        }
    }

    pub fn move_down(&mut self) {
        let n = self.visible_len();
        if n > 0 {
            self.selected = (self.selected + 1) % n;
        }
    }

    pub fn move_up(&mut self) {
        let n = self.visible_len();
        if n > 0 {
            self.selected = if self.selected == 0 { n - 1 } else { self.selected - 1 };
        }
    }

    pub fn jump_top(&mut self) {
        self.selected = 0;
    }

    pub fn jump_bottom(&mut self) {
        let n = self.visible_len();
        if n > 0 {
            self.selected = n - 1;
        }
    }

    pub fn toggle_show_all(&mut self) {
        self.show_all = !self.show_all;
        self.reload();
        if self.query_active {
            self.ensure_walked();
            self.recompute_matches();
        }
    }

    pub fn start_query(&mut self) {
        self.query_active = true;
        self.query.clear();
        self.selected = 0;
        self.ensure_walked();
    }

    pub fn cancel_query(&mut self) {
        self.query_active = false;
        self.query.clear();
        self.matches.clear();
        self.selected = 0;
    }

    pub fn query_push(&mut self, c: char) {
        self.query.push(c);
        self.recompute_matches();
        self.selected = 0;
    }

    pub fn query_pop(&mut self) -> bool {
        // Returns true if query mode should remain active.
        if self.query.pop().is_some() {
            self.recompute_matches();
            self.selected = 0;
            true
        } else {
            self.query_active = false;
            self.matches.clear();
            false
        }
    }

    fn ensure_walked(&mut self) {
        if self.walked {
            return;
        }
        self.all_files = walk_files(&self.current_dir, self.show_all, WALK_CAP);
        self.all_displays = self
            .all_files
            .iter()
            .map(|p| {
                p.strip_prefix(&self.current_dir)
                    .unwrap_or(p)
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        self.walked = true;
    }

    fn recompute_matches(&mut self) {
        self.matches.clear();
        if self.query.is_empty() {
            return;
        }
        let pattern = Pattern::parse(&self.query, CaseMatching::Ignore, Normalization::Smart);
        let mut scored: Vec<(u32, usize)> = Vec::with_capacity(self.all_displays.len());
        let mut buf = Vec::new();
        for (i, display) in self.all_displays.iter().enumerate() {
            let haystack = Utf32Str::new(display, &mut buf);
            if let Some(score) = pattern.score(haystack, &mut self.matcher) {
                scored.push((score, i));
            }
        }
        scored.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        for (_, idx) in scored.into_iter().take(MATCH_LIMIT) {
            self.matches.push(MatchEntry {
                path: self.all_files[idx].clone(),
                display: self.all_displays[idx].clone(),
            });
        }
    }
}

fn walk_files(root: &Path, show_all: bool, cap: usize) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if out.len() >= cap {
            break;
        }
        let read = match std::fs::read_dir(&dir) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for ent in read.flatten() {
            let name = ent.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            if matches!(name.as_str(), "target" | "node_modules" | "__pycache__") {
                continue;
            }
            let path = ent.path();
            let is_dir = ent.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if is_dir {
                stack.push(path);
            } else {
                if !show_all && !looks_like_script(&name) {
                    continue;
                }
                out.push(path);
                if out.len() >= cap {
                    break;
                }
            }
        }
    }
    out
}

fn looks_like_script(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".sh")
        || lower.ends_with(".sbatch")
        || lower.ends_with(".slurm")
        || lower.ends_with(".bash")
}

fn picker_start_dir(script_path: &str) -> PathBuf {
    if !script_path.is_empty() {
        let p = Path::new(script_path);
        if let Some(parent) = p.parent() {
            if parent.as_os_str().is_empty() {
                // path was just a bare filename — use cwd
            } else if parent.is_dir() {
                return std::fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
            }
        }
    }
    std::env::current_dir().unwrap_or_else(|_| {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"))
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Jobs,
    Nodes,
    Submit,
    History,
}

impl Tab {
    pub const ALL: [Tab; 4] = [Tab::Jobs, Tab::Nodes, Tab::Submit, Tab::History];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Jobs => "Jobs",
            Tab::Nodes => "Nodes",
            Tab::Submit => "Submit",
            Tab::History => "History",
        }
    }

    pub fn index(self) -> usize {
        match self {
            Tab::Jobs => 0,
            Tab::Nodes => 1,
            Tab::Submit => 2,
            Tab::History => 3,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum JobFilter {
    MyJobs,
    AllJobs,
}

pub enum Popup {
    None,
    JobDetail(JobDetail),
    ConfirmCancel { job_id: String },
    SubmitConfirm,
    SubmitResult { success: bool, message: String },
    FilePicker(FilePicker),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HistoryRange {
    Today,
    Week,
    Month,
}

impl HistoryRange {
    pub fn label(self) -> &'static str {
        match self {
            HistoryRange::Today => "Today",
            HistoryRange::Week => "Past 7 Days",
            HistoryRange::Month => "Past 30 Days",
        }
    }

    pub fn start_date(self) -> String {
        use std::process::Command;
        let days = match self {
            HistoryRange::Today => "0",
            HistoryRange::Week => "7",
            HistoryRange::Month => "30",
        };
        let output = Command::new("date")
            .args(["-d", &format!("{} days ago", days), "+%Y-%m-%d"])
            .output();
        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            Err(_) => "2000-01-01".to_string(),
        }
    }

    fn next(self) -> Self {
        match self {
            HistoryRange::Today => HistoryRange::Week,
            HistoryRange::Week => HistoryRange::Month,
            HistoryRange::Month => HistoryRange::Today,
        }
    }
}

pub struct App {
    pub active_tab: Tab,
    pub should_quit: bool,

    pub popup: Popup,
    pub popup_scroll: u16,

    pub jobs: Vec<Job>,
    pub jobs_table_state: TableState,
    pub job_filter: JobFilter,
    pub job_search: String,
    pub job_search_active: bool,

    pub partitions: Vec<PartitionInfo>,
    pub nodes_table_state: TableState,

    pub submit_form: SubmitForm,

    pub history: Vec<HistoryEntry>,
    pub history_table_state: TableState,
    pub history_range: HistoryRange,
    pub history_search: String,
    pub history_search_active: bool,

    pub last_refresh: Instant,
    pub refresh_interval: Duration,
    pub username: String,

    pub status_message: Option<(String, Instant)>,
}

impl App {
    pub fn new() -> Self {
        let username = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        let mut app = Self {
            active_tab: Tab::Jobs,
            should_quit: false,

            popup: Popup::None,
            popup_scroll: 0,

            jobs: Vec::new(),
            jobs_table_state: TableState::default(),
            job_filter: JobFilter::MyJobs,
            job_search: String::new(),
            job_search_active: false,

            partitions: Vec::new(),
            nodes_table_state: TableState::default(),

            submit_form: SubmitForm::new(),

            history: Vec::new(),
            history_table_state: TableState::default(),
            history_range: HistoryRange::Week,
            history_search: String::new(),
            history_search_active: false,

            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(10),
            username,

            status_message: None,
        };
        app.refresh_all();
        app
    }

    pub fn time_until_refresh(&self) -> Duration {
        let elapsed = self.last_refresh.elapsed();
        self.refresh_interval.saturating_sub(elapsed)
    }

    pub fn tick(&mut self) {
        if self.last_refresh.elapsed() >= self.refresh_interval {
            self.refresh_active_tab();
            self.last_refresh = Instant::now();
        }
    }

    pub fn refresh_all(&mut self) {
        self.refresh_jobs();
        self.refresh_partitions();
        self.refresh_history();
        self.load_partition_names();
        self.last_refresh = Instant::now();
    }

    fn switch_to_tab(&mut self, tab: Tab) {
        if self.active_tab == tab {
            return;
        }
        self.active_tab = tab;
        self.refresh_active_tab();
    }

    fn refresh_active_tab(&mut self) {
        match self.active_tab {
            Tab::Jobs => self.refresh_jobs(),
            Tab::Nodes => self.refresh_partitions(),
            Tab::Submit => {}
            Tab::History => self.refresh_history(),
        }
        self.last_refresh = Instant::now();
    }

    fn refresh_jobs(&mut self) {
        let filter = match self.job_filter {
            JobFilter::MyJobs => Some(self.username.as_str()),
            JobFilter::AllJobs => None,
        };
        match slurm::fetch_jobs(filter) {
            Ok(jobs) => {
                let count = jobs.len();
                self.jobs = jobs;
                self.set_status(format!("{} jobs loaded", count));
            }
            Err(e) => self.set_status(format!("squeue error: {}", e)),
        }
    }

    fn refresh_partitions(&mut self) {
        match slurm::fetch_partitions() {
            Ok(parts) => self.partitions = parts,
            Err(e) => self.set_status(format!("sinfo error: {}", e)),
        }
    }

    fn refresh_history(&mut self) {
        let start = self.history_range.start_date();
        match slurm::fetch_history(&self.username, &start) {
            Ok(entries) => self.history = entries,
            Err(e) => self.set_status(format!("sacct error: {}", e)),
        }
    }

    fn load_partition_names(&mut self) {
        if let Ok(names) = slurm::fetch_partition_names() {
            self.submit_form.available_partitions = names;
        }
    }

    fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    pub fn status_text(&self) -> Option<&str> {
        if let Some((ref msg, when)) = self.status_message {
            if when.elapsed() < Duration::from_secs(5) {
                return Some(msg);
            }
        }
        None
    }

    pub fn filtered_jobs(&self) -> Vec<&Job> {
        let search = self.job_search.to_lowercase();
        self.jobs
            .iter()
            .filter(|j| {
                if search.is_empty() {
                    return true;
                }
                j.job_id.to_lowercase().contains(&search)
                    || j.name.to_lowercase().contains(&search)
                    || j.partition.to_lowercase().contains(&search)
                    || j.state.to_lowercase().contains(&search)
                    || j.user.to_lowercase().contains(&search)
                    || j.reason_or_nodelist.to_lowercase().contains(&search)
            })
            .collect()
    }

    pub fn filtered_history(&self) -> Vec<&HistoryEntry> {
        let search = self.history_search.to_lowercase();
        self.history
            .iter()
            .filter(|h| {
                if search.is_empty() {
                    return true;
                }
                h.job_id.to_lowercase().contains(&search)
                    || h.job_name.to_lowercase().contains(&search)
                    || h.partition.to_lowercase().contains(&search)
                    || h.state.to_lowercase().contains(&search)
            })
            .collect()
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        // Popup handling takes priority
        if !matches!(self.popup, Popup::None) {
            self.on_key_popup(key);
            return;
        }

        // Search input mode
        if self.job_search_active && self.active_tab == Tab::Jobs {
            self.on_key_search_jobs(key);
            return;
        }
        if self.history_search_active && self.active_tab == Tab::History {
            self.on_key_search_history(key);
            return;
        }

        // Submit form editing mode
        if self.submit_form.editing && self.active_tab == Tab::Submit {
            self.on_key_submit_edit(key);
            return;
        }

        // Global keys
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return;
            }
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                let next = (self.active_tab.index() + 1) % Tab::ALL.len();
                self.switch_to_tab(Tab::ALL[next]);
                return;
            }
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                let prev = (self.active_tab.index() + Tab::ALL.len() - 1) % Tab::ALL.len();
                self.switch_to_tab(Tab::ALL[prev]);
                return;
            }
            KeyCode::Char('1') => {
                self.switch_to_tab(Tab::Jobs);
                return;
            }
            KeyCode::Char('2') => {
                self.switch_to_tab(Tab::Nodes);
                return;
            }
            KeyCode::Char('3') => {
                self.switch_to_tab(Tab::Submit);
                return;
            }
            KeyCode::Char('4') => {
                self.switch_to_tab(Tab::History);
                return;
            }
            KeyCode::Char('r') => {
                self.refresh_active_tab();
                return;
            }
            _ => {}
        }

        // Tab-specific keys
        match self.active_tab {
            Tab::Jobs => self.on_key_jobs(key),
            Tab::Nodes => self.on_key_nodes(key),
            Tab::Submit => self.on_key_submit(key),
            Tab::History => self.on_key_history(key),
        }
    }

    fn on_key_popup(&mut self, key: KeyEvent) {
        match &self.popup {
            Popup::ConfirmCancel { .. } => match key.code {
                KeyCode::Char('y') => {
                    if let Popup::ConfirmCancel { ref job_id } = self.popup {
                        let jid = job_id.clone();
                        match slurm::cancel_job(&jid) {
                            Ok(()) => {
                                self.set_status(format!("Job {} cancelled", jid));
                                self.refresh_jobs();
                            }
                            Err(e) => self.set_status(format!("Cancel failed: {}", e)),
                        }
                    }
                    self.popup = Popup::None;
                }
                KeyCode::Char('n') | KeyCode::Esc | KeyCode::Char('q') => {
                    self.popup = Popup::None;
                }
                _ => {}
            },
            Popup::SubmitConfirm => match key.code {
                KeyCode::Char('y') => {
                    match slurm::submit_job(&self.submit_form) {
                        Ok(msg) => {
                            self.popup = Popup::SubmitResult {
                                success: true,
                                message: msg,
                            };
                            self.refresh_jobs();
                        }
                        Err(e) => {
                            self.popup = Popup::SubmitResult {
                                success: false,
                                message: e,
                            };
                        }
                    }
                }
                KeyCode::Char('n') | KeyCode::Esc | KeyCode::Char('q') => {
                    self.popup = Popup::None;
                }
                _ => {}
            },
            Popup::JobDetail(_) => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => self.popup = Popup::None,
                KeyCode::Char('j') | KeyCode::Down => {
                    self.popup_scroll = self.popup_scroll.saturating_add(1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.popup_scroll = self.popup_scroll.saturating_sub(1);
                }
                _ => {}
            },
            Popup::SubmitResult { .. } => match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                    self.popup = Popup::None;
                }
                _ => {}
            },
            Popup::FilePicker(_) => {
                let query_active = matches!(&self.popup, Popup::FilePicker(p) if p.query_active);
                if query_active {
                    match key.code {
                        KeyCode::Esc => {
                            if let Popup::FilePicker(ref mut p) = self.popup {
                                p.cancel_query();
                            }
                        }
                        KeyCode::Up => {
                            if let Popup::FilePicker(ref mut p) = self.popup {
                                p.move_up();
                            }
                        }
                        KeyCode::Down => {
                            if let Popup::FilePicker(ref mut p) = self.popup {
                                p.move_down();
                            }
                        }
                        KeyCode::Enter => {
                            let picked = if let Popup::FilePicker(ref mut p) = self.popup {
                                p.enter_selected()
                            } else {
                                None
                            };
                            if let Some(path) = picked {
                                self.popup = Popup::None;
                                self.apply_picked_script(path);
                            }
                        }
                        KeyCode::Backspace => {
                            if let Popup::FilePicker(ref mut p) = self.popup {
                                p.query_pop();
                            }
                        }
                        KeyCode::Char(c) => {
                            if let Popup::FilePicker(ref mut p) = self.popup {
                                p.query_push(c);
                            }
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            self.popup = Popup::None;
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            if let Popup::FilePicker(ref mut p) = self.popup {
                                p.move_down();
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if let Popup::FilePicker(ref mut p) = self.popup {
                                p.move_up();
                            }
                        }
                        KeyCode::Char('g') => {
                            if let Popup::FilePicker(ref mut p) = self.popup {
                                p.jump_top();
                            }
                        }
                        KeyCode::Char('G') => {
                            if let Popup::FilePicker(ref mut p) = self.popup {
                                p.jump_bottom();
                            }
                        }
                        KeyCode::Char('h') | KeyCode::Backspace => {
                            if let Popup::FilePicker(ref mut p) = self.popup {
                                p.go_up();
                            }
                        }
                        KeyCode::Char('a') => {
                            if let Popup::FilePicker(ref mut p) = self.popup {
                                p.toggle_show_all();
                            }
                        }
                        KeyCode::Char('/') => {
                            if let Popup::FilePicker(ref mut p) = self.popup {
                                p.start_query();
                            }
                        }
                        KeyCode::Enter => {
                            let picked = if let Popup::FilePicker(ref mut p) = self.popup {
                                p.enter_selected()
                            } else {
                                None
                            };
                            if let Some(path) = picked {
                                self.popup = Popup::None;
                                self.apply_picked_script(path);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Popup::None => {}
        }
    }

    fn apply_picked_script(&mut self, path: PathBuf) {
        let resolved = std::fs::canonicalize(&path).unwrap_or(path);
        self.submit_form.script_path = resolved.to_string_lossy().to_string();
        self.parse_and_apply_directives(&resolved);
    }

    fn parse_and_apply_directives(&mut self, path: &Path) {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "script".to_string());
        match slurm::parse_sbatch_directives(path) {
            Ok(d) => {
                let count = d.count;
                self.submit_form.apply_directives(&d);
                if count > 0 {
                    self.set_status(format!("Loaded {} #SBATCH directive(s) from {}", count, name));
                } else {
                    self.set_status(format!("No #SBATCH directives in {}", name));
                }
            }
            Err(e) => self.set_status(format!("Parse error: {}", e)),
        }
    }

    fn on_key_jobs(&mut self, key: KeyEvent) {
        let job_count = self.filtered_jobs().len();
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if job_count > 0 {
                    let i = self.jobs_table_state.selected().map_or(0, |i| {
                        if i >= job_count - 1 { 0 } else { i + 1 }
                    });
                    self.jobs_table_state.select(Some(i));
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if job_count > 0 {
                    let i = self.jobs_table_state.selected().map_or(0, |i| {
                        if i == 0 { job_count - 1 } else { i - 1 }
                    });
                    self.jobs_table_state.select(Some(i));
                }
            }
            KeyCode::Enter => {
                if let Some(selected) = self.jobs_table_state.selected() {
                    let filtered = self.filtered_jobs();
                    if let Some(job) = filtered.get(selected) {
                        let job_id = job.job_id.clone();
                        match slurm::fetch_job_detail(&job_id) {
                            Ok(detail) => {
                                self.popup_scroll = 0;
                                self.popup = Popup::JobDetail(detail);
                            }
                            Err(e) => self.set_status(format!("scontrol error: {}", e)),
                        }
                    }
                }
            }
            KeyCode::Char('d') => {
                if let Some(selected) = self.jobs_table_state.selected() {
                    let filtered = self.filtered_jobs();
                    if let Some(job) = filtered.get(selected) {
                        self.popup = Popup::ConfirmCancel {
                            job_id: job.job_id.clone(),
                        };
                    }
                }
            }
            KeyCode::Char('/') => {
                self.job_search_active = true;
            }
            KeyCode::Char('f') => {
                self.job_filter = match self.job_filter {
                    JobFilter::MyJobs => JobFilter::AllJobs,
                    JobFilter::AllJobs => JobFilter::MyJobs,
                };
                self.jobs_table_state.select(None);
                self.refresh_jobs();
            }
            _ => {}
        }
    }

    fn on_key_nodes(&mut self, key: KeyEvent) {
        let count = self.partitions.len();
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if count > 0 {
                    let i = self.nodes_table_state.selected().map_or(0, |i| {
                        if i >= count - 1 { 0 } else { i + 1 }
                    });
                    self.nodes_table_state.select(Some(i));
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if count > 0 {
                    let i = self.nodes_table_state.selected().map_or(0, |i| {
                        if i == 0 { count - 1 } else { i - 1 }
                    });
                    self.nodes_table_state.select(Some(i));
                }
            }
            _ => {}
        }
    }

    fn on_key_submit(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.submit_form.active_field =
                    (self.submit_form.active_field + 1) % SubmitForm::FIELD_COUNT;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.submit_form.active_field = (self.submit_form.active_field
                    + SubmitForm::FIELD_COUNT
                    - 1)
                    % SubmitForm::FIELD_COUNT;
            }
            KeyCode::Enter => {
                if self.submit_form.active_field == 2 {
                    // Partition field: cycle through available partitions
                    let parts = &self.submit_form.available_partitions;
                    if !parts.is_empty() {
                        let current = &self.submit_form.partition;
                        let idx = parts.iter().position(|p| p == current).map_or(0, |i| {
                            (i + 1) % parts.len()
                        });
                        self.submit_form.partition = parts[idx].clone();
                    }
                } else {
                    self.submit_form.editing = true;
                }
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.submit_form.script_path.is_empty() {
                    self.set_status("Script path is required".to_string());
                } else {
                    self.popup = Popup::SubmitConfirm;
                }
            }
            KeyCode::Char('b') if self.submit_form.active_field == 0 => {
                let start = picker_start_dir(&self.submit_form.script_path);
                let mut picker = FilePicker::new(start);
                picker.start_query();
                self.popup = Popup::FilePicker(picker);
            }
            KeyCode::Char('c') => {
                self.submit_form.clear();
                self.set_status("Submit form cleared".to_string());
            }
            _ => {}
        }
    }

    fn on_key_submit_edit(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.submit_form.editing = false;
                if self.submit_form.active_field == 0 && !self.submit_form.script_path.is_empty() {
                    let path = PathBuf::from(&self.submit_form.script_path);
                    if path.is_file() {
                        self.parse_and_apply_directives(&path);
                    }
                }
            }
            KeyCode::Char(c) => {
                if let Some(field) = self.submit_form.field_value_mut(self.submit_form.active_field)
                {
                    field.push(c);
                }
            }
            KeyCode::Backspace => {
                if let Some(field) = self.submit_form.field_value_mut(self.submit_form.active_field)
                {
                    field.pop();
                }
            }
            _ => {}
        }
    }

    fn on_key_history(&mut self, key: KeyEvent) {
        let count = self.filtered_history().len();
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if count > 0 {
                    let i = self.history_table_state.selected().map_or(0, |i| {
                        if i >= count - 1 { 0 } else { i + 1 }
                    });
                    self.history_table_state.select(Some(i));
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if count > 0 {
                    let i = self.history_table_state.selected().map_or(0, |i| {
                        if i == 0 { count - 1 } else { i - 1 }
                    });
                    self.history_table_state.select(Some(i));
                }
            }
            KeyCode::Enter => {
                if let Some(selected) = self.history_table_state.selected() {
                    let filtered = self.filtered_history();
                    if let Some(entry) = filtered.get(selected) {
                        let job_id = entry.job_id.clone();
                        match slurm::fetch_job_detail(&job_id) {
                            Ok(detail) => {
                                self.popup_scroll = 0;
                                self.popup = Popup::JobDetail(detail);
                            }
                            Err(e) => self.set_status(format!("scontrol error: {}", e)),
                        }
                    }
                }
            }
            KeyCode::Char('/') => {
                self.history_search_active = true;
            }
            KeyCode::Char('f') => {
                self.history_range = self.history_range.next();
                self.history_table_state.select(None);
                self.refresh_history();
            }
            _ => {}
        }
    }

    fn on_key_search_jobs(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.job_search_active = false;
            }
            KeyCode::Char(c) => {
                self.job_search.push(c);
                self.jobs_table_state.select(None);
            }
            KeyCode::Backspace => {
                self.job_search.pop();
                self.jobs_table_state.select(None);
            }
            _ => {}
        }
    }

    fn on_key_search_history(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.history_search_active = false;
            }
            KeyCode::Char(c) => {
                self.history_search.push(c);
                self.history_table_state.select(None);
            }
            KeyCode::Backspace => {
                self.history_search.pop();
                self.history_table_state.select(None);
            }
            _ => {}
        }
    }
}
