# slurmtui

A terminal UI for managing Slurm HPC cluster jobs, built with Rust.

## Features

- **Jobs** — View running/pending jobs, filter by user, search, view details, cancel jobs
- **Nodes** — View partition and node status across the cluster
- **Submit** — Build and submit jobs with a form interface and live command preview
- **History** — Browse completed job history with time-range filtering

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

## Themes

SlurmTUI uses `tokyo-night-moon` by default. To select another shared theme,
set its name in `~/.config/slurmtui/config.toml`:

```toml
theme = "catppuccin-mocha"
```

Themes are loaded from `~/.config/themes/<name>.toml`. An optional
`~/.config/slurmtui/themes/<name>.toml` file is applied afterward for
Slurm-specific overrides. The shared `[colors]` and `[ui]` sections work as-is;
an app override can add a `[slurm]` section with `background`, `selection`,
`text`, `text_dim`, `text_muted`, `hint`, `border`, `heading`, `completing`,
`key`, `success`, `completed`, `pending`, `accent`, `warning`, `error`, or
`metric` roles.

## Key Bindings

| Key | Action |
|-----|--------|
| `Tab` / `1-4` | Switch tabs |
| `j/k` / Up/Down | Navigate |
| `Enter` | Job detail / Edit field |
| `d` | Cancel selected job |
| `/` | Search/filter |
| `f` | Toggle filter (My Jobs/All Jobs, time range) |
| `r` | Manual refresh |
| `Ctrl+s` | Submit job (Submit tab) |
| `q` | Quit |

## Requirements

- Slurm CLI tools (`squeue`, `sinfo`, `sacct`, `scontrol`, `sbatch`, `scancel`)
- A terminal with Unicode support
