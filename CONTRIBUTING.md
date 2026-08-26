# Contributing

Thanks for helping build the engine. This page is the short version; `CODESTYLE.md` has the
code rules, `docs/PLAN.md` has the work.

## Before you start

- New to the project? [`CAPSTONE.md`](CAPSTONE.md) (Ukrainian) explains what it is, what you
  will learn and how the work is organised.
- Read `docs/DOMAIN.md`. Most review comments on matching and ledger code are about semantics,
  not Rust.
- Pick a task from `docs/PLAN.md` (first unchecked task whose dependencies are done) or open an
  issue for anything else. Mention the task id in the issue/PR title.
- Run `just setup` once.

## Local setup

The repo is **IDE-agnostic**. It assumes a Rust toolchain (pinned in `rust-toolchain.toml`),
`just`, and the tools `just setup` installs — nothing else. Every operation is a `just` recipe,
so any editor — a JetBrains IDE, VS Code, Zed, Helix or a bare terminal — works the same, and
CI runs those same recipes.

Contributors work with Claude Code, and its shared guidance lives in `CLAUDE.md` (plus
`AGENTS.md` for other agents) and `.claude/skills/`. Anything specific to **your** machine stays
out of the repo:

| Yours, never committed           | What belongs there                                                                                                                   |
|----------------------------------|--------------------------------------------------------------------------------------------------------------------------------------|
| `CLAUDE.local.md`                | local MCP servers, IDE bridges, personal indexes, machine-specific paths and workarounds. Loads automatically alongside `CLAUDE.md`. |
| `.claude/settings.local.json`    | permissions for tools only you have. Its `allow` list merges with the shared `.claude/settings.json`.                                |
| `.idea/`, `.vscode/`, `.zed/`, … | editor state                                                                                                                         |

Both files are in `.gitignore`. A minimal `CLAUDE.local.md` looks like:

```markdown
# CLAUDE.local.md

Machine-specific notes for Claude Code. Not committed.

## Local tooling
- `<my-index-mcp>`: pass `project: agora`; refreshes every ~2 min, so prefer `Read` for files
  edited this session.
- `<my-ide-bridge>`: precise go-to-definition and diagnostics.
```

Rules that keep the workspace shared:

- **Never** put a tool that only you have into `CLAUDE.md`, `AGENTS.md`, a skill, the `justfile`
  or CI — a contributor without it must still be able to follow every instruction.
- Knowledge that must outlive your session goes into the repo (`docs/PLAN.md`, an ADR,
  `CHANGELOG.md`), not into a personal note store.
- If a local tool turns out to be genuinely useful for everyone, propose adding it as an
  optional `just` recipe with a documented fallback — not as a hard dependency.

## Workflow

1. Branch from `main`: `p2-4-ioc-fok` (task id, short slug), or `fix-<slug>` / `docs-<slug>`.
2. Commit messages: English, imperative summary line, capitalised, no trailing period
   (`Add FOK pre-walk to the fill loop`). Body explains *why* when it is not obvious.
3. Keep PRs to one task. Tick the task checkbox in `docs/PLAN.md` in the same PR.
4. `just verify` must pass locally. CI runs the same gates. `main` takes commits only
   through a PR whose **all gates** check is green — that one job gates the other eight.
5. Fill the PR template: what changed, how it was tested, bench numbers if the hot path moved,
   ADR link if a decision was made.
6. One approving review whenever there is a second pair of eyes. The ruleset does not
   require an approval — with a single maintainer it would deadlock the repo — so this
   one is on you. Reviewers use `.claude/skills/domain-check` and
   `.claude/skills/hot-path-review` as checklists; you can pre-empt them.

## What a PR needs

- Tests that fail without the change (unit/property/model as the task says).
- Docs on every new `pub` item, per `CODESTYLE.md`.
- No new `#[allow]`; `#[expect(lint, reason = "...")]` only.
- No floats, no unchecked arithmetic in finance crates, no locks on the command path.
- A bench and numbers in `docs/PERFORMANCE.md` for hot-path changes.
- An ADR in `docs/adr/` for a decision that constrains future work.

## Adding a crate

Use the `new-crate` skill or copy an existing crate: `[lints] workspace = true`, README as
crate doc, the standard `lib.rs` preamble, alphabetical `members` entry in `Cargo.toml`.

## Reporting bugs

Open an issue with the template. For matching or ledger bugs include the command sequence that
reproduces it; if you can, add it as a failing test in the PR that fixes it.

## Security

See `SECURITY.md`.

## Code of conduct

Be kind, be direct, assume good intent, keep discussions about the code.
