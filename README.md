# slurmtui

A terminal UI for managing Slurm HPC cluster jobs, built with Rust.

## Features

- **Jobs** — View, filter, inspect, hold, release, requeue, stop, continue, signal, and cancel jobs
- **Nodes** — View partition and node status across the cluster
- **Submit** — Build and submit jobs with a form interface and live command preview
- **History** — Browse completed job history with time-range filtering
- **Responsive operations** — Run Slurm commands outside the UI thread with a configurable timeout

## Install

```bash
# Requires Rust toolchain
cargo install --path .

# Or with just:
just install
```

## Usage

```bash
slurmtui
```

## Configuration

All settings are optional and load from `~/.config/slurmtui/config.toml`.
Durations are positive whole seconds; invalid values fall back individually.

```toml
jobs_refresh_seconds = 10
nodes_refresh_seconds = 30
idle_pause_seconds = 120
log_follow_seconds = 2
command_timeout_seconds = 15

default_partition = "gpu"
default_job_filter = "mine"       # "mine" or "all"
default_history_range = "week"    # "today", "week", or "month"
```

## Themes

SlurmTUI uses `tokyo-night-moon` by default. Press `t` to open the theme picker;
moving through the list previews each theme, `Enter` keeps it for the current
session, and `Esc` or `q` restores the previous theme. Picker changes never
rewrite `config.toml`; set the startup theme directly in
`~/.config/slurmtui/config.toml`:

```toml
theme = "~/.config/themes/catppuccin-mocha.toml"
theme_catalog = "~/.config/themes/catalog.toml"
```

`theme` is loaded directly. `theme_catalog` contains an explicit `themes = [...]`
array used by the picker. SlurmTUI never scans a theme directory. A theme can
add a `[slurm]` section with `background`, `selection`,
`text`, `text_dim`, `text_muted`, `hint`, `border`, `heading`, `completing`,
`key`, `success`, `completed`, `pending`, `accent`, `warning`, `error`, or
`metric` roles.

## Key Bindings

| Key | Action |
|-----|--------|
| `Tab` / `1-4` | Switch tabs |
| `j/k` / Up/Down | Navigate |
| `Enter` | Job detail / Edit field |
| `a` | Open state-aware job actions |
| `d` | Cancel selected job |
| `/` | Search/filter |
| `f` | Toggle filter (My Jobs/All Jobs, time range) |
| `r` | Manual refresh |
| `t` | Open theme picker |
| `Ctrl+s` | Submit job (Submit tab) |
| `q` | Quit |

## Requirements

- Slurm CLI tools (`squeue`, `sinfo`, `sacct`, `scontrol`, `sbatch`, `scancel`)
- A terminal with Unicode support
