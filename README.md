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
