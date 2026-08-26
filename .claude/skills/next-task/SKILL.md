---
name: next-task
description: Pick the next task from docs/PLAN.md and drive it to a green `just verify`; pass a task id (e.g. P2.4) to force one
disable-model-invocation: true
---

# Next task

Executes one task from `docs/PLAN.md` end to end.

## Select

1. If an argument like `P2.4` is given, use it. Otherwise take the first unchecked task whose
   `Depends on` list is fully checked, earliest phase first.
2. Check that nobody else is on it: `git branch -a`, open PRs, and the task's own wording. If
   your setup has a session-notes tool, check it too (see `CLAUDE.local.md`).
3. Restate the task in two lines: deliverables and acceptance. If either is unclear, read the
   phase intro, `docs/DOMAIN.md` and the relevant ADR before asking.

## Design (only if the task touches semantics, invariants or threading)

Work through ELIMINATE → SIMPLIFY → REUSE → CREATE: what can be removed, solved with less
code, or reused before anything new is written; what invariant the change must keep; which
tests prove it. Write the calling code first. If a decision constrains the future, draft the
ADR now (`/adr`).

## Implement

- Branch `p<phase>-<n>-<slug>`.
- Tests first for bugs and invariants (`proptest` for properties, unit tests for
  `docs/DOMAIN.md` examples, `insta` for formats).
- Follow `CODESTYLE.md`; no `#[allow]`; documentation on every `pub` item.
- Hot path: run `/hot-path-review` on your own diff; add a bench.

## Finish

1. `just verify` green. Paste failures verbatim if not, and fix them.
2. Tick the checkbox in `docs/PLAN.md`; add follow-up tasks you discovered.
3. Update `CHANGELOG.md` under Unreleased.
4. Commit (imperative summary, no trailing period, one task) and open the PR with the template.
5. Leave the trail in the repo, not in your head: the ticked checkbox, any follow-up tasks, the
   `CHANGELOG.md` entry and — if you decided something — the ADR.
