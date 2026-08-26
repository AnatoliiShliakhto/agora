---
name: bench
description: Run criterion benches for a crate, compare against the saved baseline and record results in docs/PERFORMANCE.md
disable-model-invocation: true
---

# Bench

Argument: crate short name (`matching`, `eventstore`, `resilience`, ...).

1. Quiet machine: close heavy processes; note `nproc`, CPU model (`lscpu | grep 'Model name'`)
   and governor (`cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor`).
2. Baseline on `main` if none exists: `git stash`/checkout, then
   `just bench <crate> -- --save-baseline main`, restore the branch.
3. Run on the branch: `just bench <crate> -- --baseline main`. criterion prints the change with
   significance; treat >10 % p99 regression as blocking, >5 % as needing a reason in the PR.
4. Record in `docs/PERFORMANCE.md`: date, commit, machine line, bench name, median and p99 before
   and after, one line on why it moved. Keep the table append-only.
5. For deeper analysis (optional, tools vary by platform): build with
   `cargo build --profile profiling -p agora-<crate>`, then profile the binary under
   `target/profiling` with whatever you have — `perf record -g` /
   `perf stat -e cycles,instructions,cache-misses,branch-misses` on Linux, `samply` or
   `cargo flamegraph` anywhere. Record which tool produced the numbers.

Numbers without the machine line and commit are not comparable — do not record them.
