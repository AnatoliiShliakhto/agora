---
name: verify
description: Run the local CI gate (`just verify`) and triage failures using the known-pitfall list
disable-model-invocation: true
---

# Verify

Run `just verify`. It stops at the first failing gate; fix and rerun until green. Report the
final status and any gate that was skipped.

## Triage by gate

- **fmt-check**: run `just fmt` (nightly rustfmt + taplo). "line formatted, but exceeded maximum
  width" inside a macro body means rustfmt cannot wrap it: shorten the line by hand.
- **lint**: read the lint name. Fix the code first; `#[expect(lint, reason = "...")]` only when
  the lint is wrong for this site. `unfulfilled_lint_expectations` means the `expect` is now
  unnecessary — remove it. Never touch the workspace lint table in a feature PR.
- **lint-features**: clippy over every feature of every crate. A failure here that `lint` did
  not catch means feature-gated code (`#[cfg(feature = "…")]`) is broken in some combination, or
  a `dev-dependency` is used from non-test code — `--no-dev-deps` removes them, so the import
  stops resolving. Note that this recipe deliberately runs **without `--locked`**: `--no-dev-deps`
  rewrites `Cargo.toml` while it runs, and `--locked` would abort on the resulting resolve. The
  real lockfile is restored unchanged; CI asserts that with `git diff --exit-code Cargo.lock`.
- **test**: nextest prints the failing test; rerun it alone with
  `just test -p <crate> <name>`. For `insta` mismatches, inspect the diff; accept with
  `INSTA_UPDATE=always just test -p <crate>` only if the change is intended and note it in the PR.
- **test-doc**: doctests run separately from nextest; a failing example is a documentation bug.
- **doc**: `-D warnings` catches broken intra-doc links; a stale trait-bound error after editing
  derives is a cargo cache issue — `cargo clean -p <crate>` and rerun.
- **deny**: `advisories` → bump or `ignore` with the RUSTSEC id and a reason in `deny.toml`;
  `licenses` → add to `allow` only if the license is permissive and compatible with MIT;
  `bans` → duplicate versions are warnings, wildcards are errors.
- **typos**: fix the word or add a domain term to `typos.toml` `extend-words`.
- **bench-build**: benches compile with the same lints; fix like `lint`.

Do not mark a task done while any gate is red or skipped.

## Editing docs and code with scripts

A `python3 - <<EOF` patch script whose `str.replace` anchor does not match is a **silent no-op**,
and a following `echo ok` still prints. Never trust the echo: wrap every replacement in a helper
that asserts the anchor exists exactly once, and `grep` the result afterwards. The same applies
to `sed -i`.
