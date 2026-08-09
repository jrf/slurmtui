use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_JOBS_REFRESH_SECONDS: u64 = 10;
const DEFAULT_NODES_REFRESH_SECONDS: u64 = 30;
const DEFAULT_IDLE_PAUSE_SECONDS: u64 = 120;
const DEFAULT_LOG_FOLLOW_SECONDS: u64 = 2;
const DEFAULT_COMMAND_TIMEOUT_SECONDS: u64 = 15;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobFilterSetting {
    Mine,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryRangeSetting {
    Today,
    Week,
    Month,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub jobs_refresh_interval: Duration,
    pub nodes_refresh_interval: Duration,
    pub idle_pause_interval: Duration,
    pub log_follow_interval: Duration,
    pub command_timeout: Duration,
    pub default_partition: Option<String>,
    pub default_job_filter: JobFilterSetting,
    pub default_history_range: HistoryRangeSetting,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            jobs_refresh_interval: Duration::from_secs(DEFAULT_JOBS_REFRESH_SECONDS),
            nodes_refresh_interval: Duration::from_secs(DEFAULT_NODES_REFRESH_SECONDS),
            idle_pause_interval: Duration::from_secs(DEFAULT_IDLE_PAUSE_SECONDS),
            log_follow_interval: Duration::from_secs(DEFAULT_LOG_FOLLOW_SECONDS),
            command_timeout: Duration::from_secs(DEFAULT_COMMAND_TIMEOUT_SECONDS),
            default_partition: None,
            default_job_filter: JobFilterSetting::Mine,
            default_history_range: HistoryRangeSetting::Week,
        }
    }
}

pub fn load() -> Config {
    config_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|content| parse(&content))
        .unwrap_or_default()
}

fn config_path() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|root| root.join("slurmtui").join("config.toml"))
}

fn parse(content: &str) -> Option<Config> {
    let values = content.parse::<toml::Value>().ok()?;
    let defaults = Config::default();
    Some(Config {
        jobs_refresh_interval: positive_seconds(&values, "jobs_refresh_seconds")
            .unwrap_or(defaults.jobs_refresh_interval),
        nodes_refresh_interval: positive_seconds(&values, "nodes_refresh_seconds")
            .unwrap_or(defaults.nodes_refresh_interval),
        idle_pause_interval: positive_seconds(&values, "idle_pause_seconds")
            .unwrap_or(defaults.idle_pause_interval),
        log_follow_interval: positive_seconds(&values, "log_follow_seconds")
            .unwrap_or(defaults.log_follow_interval),
        command_timeout: positive_seconds(&values, "command_timeout_seconds")
            .unwrap_or(defaults.command_timeout),
        default_partition: nonempty_string(&values, "default_partition"),
        default_job_filter: match string(&values, "default_job_filter") {
            Some("all") => JobFilterSetting::All,
            _ => defaults.default_job_filter,
        },
        default_history_range: match string(&values, "default_history_range") {
            Some("today") => HistoryRangeSetting::Today,
            Some("month") => HistoryRangeSetting::Month,
            _ => defaults.default_history_range,
        },
    })
}

fn positive_seconds(values: &toml::Value, key: &str) -> Option<Duration> {
    let seconds = values.get(key)?.as_integer()?;
    (seconds > 0).then(|| Duration::from_secs(seconds as u64))
}

fn string<'a>(values: &'a toml::Value, key: &str) -> Option<&'a str> {
    values.get(key)?.as_str().map(str::trim)
}

fn nonempty_string(values: &toml::Value, key: &str) -> Option<String> {
    string(values, key)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_runtime_defaults() {
        let config = parse(
            r#"
jobs_refresh_seconds = 5
nodes_refresh_seconds = 20
idle_pause_seconds = 90
log_follow_seconds = 3
command_timeout_seconds = 25
default_partition = "gpu"
default_job_filter = "all"
default_history_range = "month"
theme = "~/themes/moon.toml"
"#,
        )
        .expect("config should parse");

        assert_eq!(config.jobs_refresh_interval, Duration::from_secs(5));
        assert_eq!(config.nodes_refresh_interval, Duration::from_secs(20));
        assert_eq!(config.idle_pause_interval, Duration::from_secs(90));
        assert_eq!(config.log_follow_interval, Duration::from_secs(3));
        assert_eq!(config.command_timeout, Duration::from_secs(25));
        assert_eq!(config.default_partition.as_deref(), Some("gpu"));
        assert_eq!(config.default_job_filter, JobFilterSetting::All);
        assert_eq!(config.default_history_range, HistoryRangeSetting::Month);
    }

    #[test]
    fn invalid_values_fall_back_individually() {
        let config = parse(
            r#"
jobs_refresh_seconds = 0
nodes_refresh_seconds = "fast"
default_partition = "  "
default_job_filter = "other"
default_history_range = "year"
"#,
        )
        .expect("config should parse");

        assert_eq!(config, Config::default());
    }
}
