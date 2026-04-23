.DEFAULT_GOAL := help

CARGO ?= cargo
CARGO_MSRV ?= cargo +1.88.0
CARGO_NIGHTLY ?= cargo +nightly
RUSTDOCFLAGS_DOCS ?= -D warnings --cfg docsrs
PACKAGE_LIST ?= /tmp/patholog-package-list.txt
FUZZ_TARGETS ?= path_clean path_parse_doctor cli_read_only
FUZZ_SMOKE_SECONDS ?= 30

.PHONY: help build build-release clean fmt fmt-check clippy test test-all \
	test-doc coverage coverage-html msrv package-list package-check docs \
	docs-missing third-party-licenses third-party-licenses-check \
	publish-dry-run pre-release ci fuzz-build fuzz-smoke fuzz-soak

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*## "}; /^[a-zA-Z0-9_.-]+:.*## / {printf "%-18s %s\n", $$1, $$2}' $(MAKEFILE_LIST) | sort

build: ## Build the crate
	$(CARGO) build --locked

build-release: ## Build the crate in release mode
	$(CARGO) build --release --locked

clean: ## Remove Cargo build artifacts
	$(CARGO) clean
	$(CARGO) clean --manifest-path fuzz/Cargo.toml

fmt: ## Format Rust sources, including fuzz targets
	$(CARGO) fmt
	$(CARGO) fmt --manifest-path fuzz/Cargo.toml

fmt-check: ## Check Rust formatting, including fuzz targets
	$(CARGO) fmt --check
	$(CARGO) fmt --manifest-path fuzz/Cargo.toml --check

clippy: ## Run clippy with CI warning policy
	$(CARGO) clippy --all-targets --all-features --locked -- -D warnings

test: ## Run default tests
	$(CARGO) test --locked

test-all: ## Run all-feature tests
	$(CARGO) test --all-features --locked

test-doc: ## Run documentation tests
	$(CARGO) test --doc --locked

coverage: ## Generate an LCOV coverage report (requires cargo-llvm-cov)
	$(CARGO) llvm-cov --all-features --locked --lib --tests --lcov --output-path lcov.info

coverage-html: ## Generate an HTML coverage report (requires cargo-llvm-cov)
	$(CARGO) llvm-cov --all-features --locked --lib --tests --html

msrv: ## Run tests on the crate MSRV (requires toolchain 1.88.0)
	$(CARGO_MSRV) test --locked

package-list: ## List files included in the published crate package
	$(CARGO) package --locked --list --allow-dirty > $(PACKAGE_LIST)
	@cat $(PACKAGE_LIST)

package-check: ## Verify crates.io package creation
	$(CARGO) package --locked --list --allow-dirty > $(PACKAGE_LIST)
	! grep -E '^(\.github/|\.history/|\.gitignore$$|AGENTS\.md$$|fuzz/|scripts/|src/.*/tests\.rs$$|src/.*/tests/|tests/)' $(PACKAGE_LIST)
	$(CARGO) package --locked --allow-dirty

docs: ## Build library docs with docs.rs warning settings
	RUSTDOCFLAGS='$(RUSTDOCFLAGS_DOCS)' $(CARGO) doc --locked --no-deps --lib

docs-missing: ## Check public library docs with missing_docs denied
	RUSTFLAGS='-D missing_docs' $(CARGO) check --lib --all-features --locked

third-party-licenses: ## Regenerate third-party dependency license notices
	$(CARGO) about generate --locked about.hbs > THIRD-PARTY-LICENSES.md

third-party-licenses-check: ## Check generated third-party license notices are current
	@tmp="$$(mktemp)"; \
	trap 'rm -f "$$tmp"' EXIT; \
	$(CARGO) about generate --locked about.hbs > "$$tmp"; \
	diff -u THIRD-PARTY-LICENSES.md "$$tmp"

publish-dry-run: ## Verify crates.io publishability without uploading
	$(CARGO) publish --dry-run --locked --allow-dirty

pre-release: ## Run the full local gate before tagging a release
	$(MAKE) fmt-check
	$(MAKE) clippy
	$(MAKE) test
	$(MAKE) test-all
	$(MAKE) test-doc
	$(MAKE) docs
	$(MAKE) docs-missing
	$(MAKE) msrv
	$(MAKE) third-party-licenses-check
	$(MAKE) package-check
	$(MAKE) publish-dry-run
	$(MAKE) build-release

ci: ## Run the main local CI checks
	$(MAKE) fmt-check
	$(MAKE) clippy
	$(MAKE) test
	$(MAKE) test-doc
	$(MAKE) msrv
	$(MAKE) package-check

fuzz-build: ## Build all cargo-fuzz targets
	@for target in $(FUZZ_TARGETS); do \
		$(CARGO_NIGHTLY) fuzz build "$$target"; \
	done

fuzz-smoke: ## Run a short fuzz soak against all targets
	PATHOLOG_FUZZ_SECONDS=$(FUZZ_SMOKE_SECONDS) PATHOLOG_FUZZ_TARGETS="$(FUZZ_TARGETS)" bash scripts/fuzz_soak.sh

fuzz-soak: ## Run the fuzz soak script from a normal interactive shell
	PATHOLOG_FUZZ_TARGETS="$(FUZZ_TARGETS)" bash scripts/fuzz_soak.sh
