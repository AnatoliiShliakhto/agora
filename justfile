# Task runner. `just` lists recipes; `just verify` is the full local CI gate.

export JUST_NO_PAGER := "1"
export RUST_BACKTRACE := "1"

native := '--config build.rustflags=["-C","target-cpu=native"]'
# `cargo fuzz` defaults to a musl target that is rarely installed; always build for the host.
host_target := `rustc -vV | sed -n 's/^host: //p'`

default:
    @just --list --unsorted

# Install the dev tools CI expects (nextest, deny, hack, typos, taplo, llvm-cov, fuzz).
setup:
    cargo binstall -y cargo-nextest cargo-deny cargo-hack typos-cli taplo-cli cargo-llvm-cov cargo-fuzz
    # `rust-src` is required by miri, which builds its own standard library.
    rustup toolchain install nightly --component rustfmt miri rust-src

# Format everything (nightly rustfmt for `rustfmt.toml` unstable options, taplo for TOML).
fmt:
    cargo +nightly fmt --all
    RUST_LOG=error taplo fmt

# Verify formatting without touching files.
fmt-check:
    cargo +nightly fmt --all -- --check
    RUST_LOG=error taplo fmt --check

# Type-check every crate, target and feature.
check:
    cargo check --workspace --all-targets --all-features --locked

# Clippy with warnings as errors (same flags as CI).
lint *args:
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings {{args}}

# Clippy over every feature of every crate; `--no-dev-deps` also catches a dev-dependency
# leaking into non-test code. No `--locked`: `--no-dev-deps` rewrites `Cargo.toml` while it
# runs, so cargo needs to resolve a lockfile for that temporary manifest, which `--locked`
# forbids. The real `Cargo.lock` is restored untouched — CI asserts that.
lint-features:
    cargo hack clippy --workspace --each-feature --no-dev-deps -- -D warnings

# Run tests (nextest if installed; `just test -p agora-matching book::` narrows scope).
test *args:
    @if command -v cargo-nextest >/dev/null 2>&1; then \
        cargo nextest run --workspace --all-features --locked {{args}}; \
    else \
        cargo test --workspace --all-features --locked {{args}}; \
    fi

# Doctests are not run by nextest; run them separately.
test-doc:
    cargo test --workspace --all-features --locked --doc

# Build docs with warnings as errors; `just doc --open` to browse.
doc *args:
    RUSTDOCFLAGS="-D warnings --cfg docsrs" cargo doc --workspace --all-features --no-deps --locked {{args}}

# Licenses, advisories, banned crates, sources.
deny:
    cargo deny check

# Spelling across sources and docs.
typos:
    typos

# Criterion benches with host-CPU codegen. `just bench matching` limits to one crate.
bench crate="" *args:
    cargo bench {{native}} --all-features {{ if crate != "" { "-p agora-" + crate } else { "--workspace" } }} {{args}}

# Compile benches without running (CI smoke test).
bench-build:
    cargo bench --workspace --all-features --no-run --locked

# Coverage report (lcov + HTML in `target/llvm-cov`).
coverage:
    cargo llvm-cov nextest --workspace --all-features --lcov --output-path target/lcov.info
    cargo llvm-cov report --html

# Run lock-free / unsafe modules under Miri.
miri *args:
    cargo +nightly miri test -p agora-resilience -p agora-runtime {{args}}

# Run a fuzz target: `just fuzz matching_stream`.
fuzz target *args:
    cd fuzz && cargo +nightly fuzz run --target {{host_target}} {{target}} {{args}}

# Full local CI gate — the same checks, in the same order, as `.github/workflows/ci.yml`.
# Run before every push.
verify: fmt-check lint lint-features test test-doc doc deny typos bench-build
    @echo "verify: all gates passed"

# Run the exchange server with the dev config.
run *args:
    cargo run -p agora-server -- {{args}}

# Run the load generator against a running server.
loadgen *args:
    cargo run --release -p agora-loadgen -- {{args}}

# Bring up local infra (NATS JetStream, Prometheus, Grafana) via podman/docker compose.
infra-up:
    cd deploy && (podman compose up -d 2>/dev/null || docker compose up -d)

infra-down:
    cd deploy && (podman compose down 2>/dev/null || docker compose down)

# Remove build artifacts.
clean:
    cargo clean
