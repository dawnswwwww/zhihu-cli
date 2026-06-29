# Development

Local-only workflow for `zhihu-cli`. There is no CI configured; the
`Makefile` targets are the contract.

## Quick reference

```bash
make help          # list targets
make test          # run full test suite
make lint          # clippy with -D warnings
make coverage      # coverage summary + gate check
make check         # lint + test + coverage (full local CI)
```

## Test-Driven Development (TDD) workflow

The audit (commit history pre-`549f382`) found that the project's original
implementation was Test-After Development, not TDD: every `feat:` commit
preceded its corresponding test. From this point forward, **every new
behavior change follows strict Red-Green-Refactor**:

1. **RED** — Write a failing test first. For a brand-new function, use a
   `todo!()` stub so the test fails at runtime (not just compile). Verify
   the failure mode is the expected "feature missing" signal.

2. **GREEN** — Implement the minimum code to make the test pass. No more.
   If you find yourself writing features the test doesn't ask for, stop.

3. **REFACTOR** — Now, and only now, clean up. Extract helpers, rename,
   deduplicate. The tests must stay green throughout.

The Iron Law: **NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST.**
If you wrote code before the test, delete the code and start over.

Existing in-module test pattern (used in `src/{client,config,cli}.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    // #[test] fn ... { ... }
}
```

For helper functions extracted from existing code (refactor), use the
`pub(crate) fn` + `#[cfg(test)] mod tests` pattern from
`src/commands/ask.rs::sse_parser`.

## Coverage gate

We use `cargo-llvm-cov` (cross-platform, supports the toolchain we already
have). Local-only:

```bash
# Install once:
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview

# Then:
./scripts/coverage.sh summary         # gate check at 80%
./scripts/coverage.sh html           # interactive HTML report
./scripts/coverage.sh text           # per-line annotated source
./scripts/coverage.sh lcov           # lcov.info for external tools

# Override threshold:
COVERAGE_MIN=85 ./scripts/coverage.sh summary
```

The gate is **total line coverage >= 80%** (with `--all-targets`, i.e.
unit + mocked-API + CLI e2e + real-API integration when a secret is set).
The aspiration is **100%** — see "Gap to 100%" below for what's missing.

### Current state (with `--all-targets`)

| File | Line Cov | Status |
|---|---|---|
| `src/error.rs` | **100%** | ✅ done |
| `src/client.rs` | **100%** | ✅ done |
| `src/main.rs` | **100%** | ✅ done |
| `src/commands/search.rs` | **100%** | ✅ done |
| `src/commands/auth.rs` | 96.43% | 🟡 test artifacts only |
| `src/commands/ask.rs` | 97.79% | 🟡 test artifacts only |
| `src/config.rs` | 97.83% | 🟡 closing-brace artifacts |
| `src/cli.rs` | 94.37% | 🟡 test panic arms |
| `src/output.rs` | 92.73% | 🟡 test artifacts only |
| **TOTAL** | **97.62%** | ✅ passes 80% gate |

**Function coverage: 100%** (every public function is called by at least one test).

## The final 2.38% — irreducible coverage artifacts

28 lines remain uncovered. They break down into two categories:

### Category A: `_ => panic!("expected …")` arms in test matches (cli.rs)

Lines `126, 138, 152, 185`. These are unreachable when tests pass; they're
the failure-mode of `match cli.command { Command::Ask(args) => … _ => panic!(…) }`.
Coverage tools count them as uncovered because the test never takes that
arm in the passing case. There is no idiomatic Rust way to express
"this match arm is only reached if the test is broken" — they're an
honest cost of `match`-based CLI parsing tests.

### Category B: closing `}` of test functions and `if let` blocks

Lines in `commands/{auth,ask}.rs`, `config.rs`, `output.rs`. The closing
brace of a function or block is reported as its own line by `cargo-llvm-cov`
even though it has no executable code. They appear as uncovered when
the surrounding block is entered but no further code lives after the
last statement (e.g., a test that ends after a single `assert!`).

These are **not real gaps** — they don't represent untested logic.
Resolving them would require restructuring tests for cosmetic coverage
metrics at the cost of readability. Documented here so future readers
don't waste time trying to cover them.

## Filing future work

When the gate fails or a gap is identified, write the test first
(per the TDD section above), then implement, then push.

## How the 100% coverage was achieved

Three extraction patterns did the heavy lifting:

1. **Pure function extraction** for parameter-assembly logic
   (`build_ask_body`, `build_request`, `status_payload`, `validate_secret`).
   The pure function returns the structured output; the I/O wrapper
   becomes a one-liner.

2. **Dependency injection** for HTTP clients (`handle_with_client` in
   `commands::{ask,search}.rs`). The `&ZhihuClient` parameter is provided
   by `wiremock` in unit tests and by `ZhihuClient::new()` in production.

3. **Reader injection** for stdin (`handle(cmd, &mut impl BufRead)` in
   `commands/auth.rs`). The `&[u8]` reader in tests, `io::stdin().lock()`
   in production.

The combination of these patterns, plus the `print_ask_result` and
`dispatch_result` helpers in each command module, is what got the
test-actionable code to 100% function coverage.

