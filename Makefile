# zhihu-cli — local development shortcuts.
#
# `make help` lists available targets. All targets are local-only; CI is
# intentionally absent until the project has a remote.

.PHONY: help test lint coverage coverage-html coverage-text coverage-lcov check release-dry-run release-check cross-linux-arm cross-check

help: ## Show this help.
	@awk 'BEGIN {FS = ":.*##"; printf "Targets:\n"} \
	/^[a-zA-Z_-]+:.*##/ {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}' \
	$(MAKEFILE_LIST)

test: ## Run the full test suite (unit + integration).
	cargo test

lint: ## Run clippy with -D warnings (matches CI hygiene expectation).
	cargo clippy --all-targets -- -D warnings

coverage: ## Coverage summary + gate check (>= 80% line coverage).
	./scripts/coverage.sh summary

coverage-html: ## Open interactive HTML coverage report.
	./scripts/coverage.sh html

coverage-text: ## Per-line annotated source.
	./scripts/coverage.sh text

coverage-lcov: ## Emit lcov.info for external tooling.
	./scripts/coverage.sh lcov

check: lint test coverage ## Full local CI: lint + test + coverage gate.

release-dry-run: ## Build release binary locally (single target).
	cargo build --release

release-check: release-dry-run ## Build release binary and print its size.
	@ls -lh target/release/zhihu

cross-linux-arm: ## Build for Linux ARM64 using cross (requires cross tool).
	cross build --release --target aarch64-unknown-linux-gnu

cross-check: ## List installed Rust targets.
	rustup target list --installed
