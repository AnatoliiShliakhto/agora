# Performance log

Append-only record of benchmark results and tuning decisions. Every row names the machine and
the commit; numbers without them are not comparable and are not recorded.

Machine line format: `<CPU model> · <nproc> cores · governor <name> · rustc <version>`.

| Date       | Commit   | Machine | Bench                      | Median before → after | p99 before → after | Note                                           |
|------------|----------|---------|----------------------------|-----------------------|--------------------|------------------------------------------------|
| 2026-08-26 | *(uncommitted)* | — | `book_side/push_remove_1k` | baseline pending | — | first bench, `VecDeque` levels; P2.1 replaces them with an arena |
