# AGENTS.md

Entry point for AI coding agents other than Claude Code (Codex, Gemini, Cursor, ...). Claude
reads `CLAUDE.md`, which is the long form of this file; the rules are the same.

- Read `docs/DOMAIN.md`, `docs/ARCHITECTURE.md`, `docs/PLAN.md`, `CODESTYLE.md` before
  changing code. Work is picked from `docs/PLAN.md` by task id.
- Build/lint/test with `just`: `just verify` is the gate and must pass before a PR.
- Rules the lints enforce and reviewers check: no floats outside `telemetry`/`loadgen`; checked
  arithmetic in finance crates; deterministic aggregates; no locks on the shard command path;
  `#[expect(lint, reason = "...")]` instead of `#[allow]`; every `pub` item documented
  (verb for functions, noun for types, intra-doc links).
- Formatting needs nightly rustfmt: `cargo +nightly fmt --all` (or `just fmt`).
- Commit messages: imperative, capitalised, no trailing period; one plan task per PR.
- Checklists live in `.claude/skills/*/SKILL.md`; they are plain markdown and apply to any agent.
- The repo is **IDE- and tool-agnostic**: it assumes only a Rust toolchain, `just`, and the dev
  tools `just setup` installs. Anything specific to your machine — local MCP servers, IDE
  bridges, personal indexes — goes in `CLAUDE.local.md` and `.claude/settings.local.json`, both
  gitignored. Never commit them, and never make a shared instruction depend on a tool other
  contributors may not have.
