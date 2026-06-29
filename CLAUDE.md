# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

`zhihu-cli` is a Rust command-line tool for the Zhihu Open Platform API. It exposes three command groups:

- `zhihu auth {login,set-secret,status}` — credential management
- `zhihu search {zhihu,global}` — Zhihu/site-wide and global web search
- `zhihu ask` — Zhida chat/completion API (fast/thinking/agent models)

The binary name is `zhihu`. The crate is `zhihu-cli`.

## Common development commands

Use the Makefile targets as the local development contract. There is no CI in this repo; `make check` is the full local gate.

```bash
make help              # list all targets
make test              # cargo test (unit + mocked API + CLI e2e + integration)
make lint              # cargo clippy --all-targets -- -D warnings
make coverage          # ./scripts/coverage.sh summary (>= 80% gate)
make check             # lint + test + coverage (run this before pushing)
make release-dry-run   # cargo build --release
make release-check     # build release binary and print size
```

Run a single test or subset:

```bash
cargo test <name_or_filter>
cargo test --test cli auth_set_secret_and_status
cargo test --test mocked_api handles_401_auth_failure
```

Coverage tooling must be installed once:

```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
```

Then use `./scripts/coverage.sh summary|html|text|lcov`.

## Code architecture

### Entry and dispatch

- `src/main.rs` is a thin `tokio::main` wrapper: parse `Cli`, dispatch to `commands::{ask,auth,search}::run`.
- `src/cli.rs` defines the clap-derived command tree, argument defaults, and value enums (`ModelTier`, `SearchDb`). It also contains CLI-parsing unit tests.

### Commands

Each command module lives under `src/commands/` and follows the same shape:

- A public `run(args)` async entry point that handles I/O side effects and prints errors.
- A testable inner `handle` (or `handle_with_client`) that returns `Result<T>`.
- Pure helper functions extracted for the parameter-assembly logic.

Examples:

- `commands/search.rs`: `build_request` returns `(path, query_params)`; `handle_with_client(&SearchCommand, &ZhihuClient)` is injected with a mock client in tests.
- `commands/ask.rs`: `build_ask_body` assembles the OpenAI-style request body; `stream_ask_with_client` handles SSE parsing via the nested `sse_parser` module.
- `commands/auth.rs`: `handle(cmd, &mut impl BufRead)` accepts a reader so tests can pass `&[u8]` while production passes `io::stdin().lock()`.

### HTTP and auth

- `src/client.rs` wraps `reqwest`. `ZhihuClient::new()` resolves the secret via `Config::resolve_secret()`. `with_secret_and_base_url` is the test constructor.
- Auth headers are injected centrally: `Authorization: Bearer <secret>` and `X-Request-Timestamp` (Unix seconds).
- Base URL defaults to `https://developer.zhihu.com`; override with `ZHIHU_OPENAPI_BASE_URL`.

### Config and errors

- `src/config.rs` stores the access secret in `~/.zhihu-cli/config.toml`. `resolve_secret` prefers `ZHIHU_ACCESS_SECRET` env var over the file.
- `src/error.rs` defines `ZhihuError` with `thiserror`. `ZhihuError::MissingSecret` renders with code `20001`; other variants omit the `code` field.
- `src/output.rs` pretty-prints success JSON and serializes errors to single-line JSON on stderr before exiting.

### Tests

- Unit tests are co-located in `src/*.rs` under `#[cfg(test)] mod tests`.
- `tests/cli.rs` runs the compiled binary with `assert_cmd`; it uses `tempfile::TempDir` and removes `ZHIHU_ACCESS_SECRET` to test unauthenticated paths.
- `tests/mocked_api.rs` uses `wiremock` to test HTTP error handling.
- `tests/integration.rs` calls the real Zhihu API when `ZHIHU_ACCESS_SECRET` is set; otherwise the tests short-circuit and pass. Use `serial_test::serial` on any test that mutates process env vars.

## Development workflow

This project follows strict Red-Green-Refactor TDD. From `docs/development.md`:

> **NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST.**

For a new behavior:

1. Write a failing test first (use `todo!()` for brand-new functions so the test fails at runtime).
2. Implement the minimum code to make it pass.
3. Refactor only after green, keeping tests green.

The project also enforces a coverage gate of **>= 80% total line coverage** with `--all-targets`. The aspiration is 100% function coverage. The remaining uncovered lines are documented in `docs/development.md` as irreducible artifacts (test-match panic arms and closing braces).

## Architectural patterns to preserve

When adding or changing commands, prefer these patterns that keep the code testable:

1. **Extract pure parameter-assembly functions** (e.g., `build_request`, `build_ask_body`, `status_payload`, `validate_secret`).
2. **Inject the HTTP client** for command handlers so `wiremock` can substitute it in tests.
3. **Inject readers for stdin** instead of calling `io::stdin()` directly inside business logic.
4. Keep `run()` as a thin I/O wrapper; put testable logic in `handle_*` helpers that return `Result`.

## Important details

- The config file path is `~/.zhihu-cli/config.toml`, not `~/.config/zhihu-cli/config.toml` (the README mentions the latter but the code uses the former).
- `ZHIHU_ACCESS_SECRET` always takes precedence over the config file.
- `zhihu search zhihu` clamps `--count` to `[1, 10]`; `zhihu search global` clamps to `[1, 20]`.
- `host=="zhihu.com"` is not supported in global search; use `zhihu search zhihu` for Zhihu-only content.
- Default ask model is `thinking` (`zhida-thinking-1p5`); alternatives are `fast` and `agent`.

## Release

Pushing a SemVer tag triggers the release workflow, which builds cross-platform binaries, creates a GitHub Release, publishes to npm, and updates the Homebrew tap.

```bash
git tag -a v0.1.3 -m "Release 0.1.3"
git push origin v0.1.3
```

## Claude Skill

This repo includes a Claude skill under `skills/zhihu-cli/`. Install it with:

```bash
npx skills add dawnswwwww/zhihu-cli
```

The skill is excluded from the Rust build and does not affect `cargo` commands.
