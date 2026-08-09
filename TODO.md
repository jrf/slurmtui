# TODO

## Now

- [ ] Test all views against live Slurm data #feature

## Next

- [ ] Focused job diagnostics for resources, pending reasons, dependencies, and logs #feature

## Later

- [ ] Job dependency visualization #feature
- [ ] Job templates / saved submit forms #feature
- [ ] Mouse support for tab/row selection #improvement

## Done

- [x] Column auto-sizing based on terminal width #improvement
- [x] Core TUI skeleton with tab navigation #feature
- [x] Jobs view with squeue, search, filter, state colors #feature
- [x] Nodes view with sinfo partition table #feature
- [x] Submit view with form and live command preview #feature
- [x] History view with sacct, time-range filter, search #feature
- [x] Job detail popup via scontrol #feature
- [x] Cancel job with confirmation #feature
- [x] Auto-refresh every 30 seconds #feature
- [x] GPU usage column in Jobs view (total + per-node for multi-node) #feature
- [x] Async slurm calls via worker thread (non-blocking tab switches) #improvement
- [x] Per-source refresh intervals (jobs 10s, nodes 30s, history manual) + idle pause #improvement
- [x] Sorting by column and direction #feature
- [x] Log file viewer with stdout/stderr switching and follow mode #feature
- [x] Non-blocking Slurm actions with bounded command execution #improvement
- [x] Runtime config for refresh, idle pause, defaults, log follow, and command timeout #feature

## Scrapped
