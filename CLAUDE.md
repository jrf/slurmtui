# slurmtop

## Overview

Terminal UI for Slurm HPC cluster management, built with Rust + ratatui + crossterm.

## Architecture

- `src/main.rs` — Entry point, terminal setup/teardown, event loop
- `src/app.rs` — App state, tab/popup enums, input dispatch, refresh logic
- `src/event.rs` — Crossterm event polling wrapper with timeout
- `src/slurm.rs` — All Slurm CLI execution and output parsing (squeue, sinfo, sacct, scontrol, sbatch, scancel)
- `src/ui/mod.rs` — Top-level layout, tab bar, status bar
- `src/ui/jobs.rs` — Jobs table (squeue)
- `src/ui/nodes.rs` — Partition/node table (sinfo)
- `src/ui/submit.rs` — Job submission form with live command preview
- `src/ui/history.rs` — Job history table (sacct)
- `src/ui/popup.rs` — Overlays for job detail, confirmation, results

## Build

```
just build       # debug build
just release     # release build
just install     # release build + copy to ~/.cargo/bin
just run         # cargo run
```

## Design Decisions

- **No JSON parsing**: Slurm 21.08.5 on this cluster lacks the `serializer/json` plugin. All output is parsed from `--format` strings with pipe delimiters or `--parsable2`.
- **Synchronous**: No tokio. Slurm commands return in <1s, so `std::process::Command` is used directly. The event loop uses crossterm's `poll()` timeout for auto-refresh.
- **Minimal dependencies**: Only `ratatui` and `crossterm`. No serde, no regex.
