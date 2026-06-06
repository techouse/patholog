.DEFAULT_GOAL := help

CARGO ?= cargo
CARGO_MSRV ?= cargo +1.88.0
CARGO_NIGHTLY ?= cargo +nightly
RUSTDOCFLAGS_DOCS ?= -D warnings --cfg docsrs
PACKAGE_LIST ?= /tmp/patholog-package-list.txt
FUZZ_TARGETS ?= path_clean path_parse_doctor cli_read_only
FUZZ_SMOKE_SECONDS ?= 30

.PHONY: help build build-release clean fmt fmt-check clippy fuzz-clippy test test-all \
	test-doc coverage coverage-html msrv package-list package-check package-check-offline \
	package-metadata-check install-smoke docs docs-missing third-party-licenses third-party-licenses-check \
	publish-dry-run version-check release-check v1-contract-check pre-release ci fuzz-build fuzz-smoke fuzz-soak

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

fuzz-clippy: ## Run clippy for the cargo-fuzz support crate
	$(CARGO) clippy --manifest-path fuzz/Cargo.toml --all-targets --locked -- -D warnings

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

package-check-offline: ## Verify crate package creation using only local cache
	$(CARGO) package --locked --list --allow-dirty > $(PACKAGE_LIST)
	! grep -E '^(\.github/|\.history/|\.gitignore$$|AGENTS\.md$$|fuzz/|scripts/|src/.*/tests\.rs$$|src/.*/tests/|tests/)' $(PACKAGE_LIST)
	$(CARGO) package --locked --allow-dirty --offline

package-metadata-check: ## Check public package metadata and v1 package policy
	@grep -q '^description = "Diagnose, explain, clean, and safely repair PATH behavior\."' Cargo.toml
	@grep -q '^license = "BSD-3-Clause"' Cargo.toml
	@grep -q '^readme = "README.md"' Cargo.toml
	@grep -q '^documentation = "https://docs.rs/patholog"' Cargo.toml
	@grep -q '^repository = "https://github.com/techouse/patholog"' Cargo.toml
	@grep -q '^homepage = "https://techouse.github.io/patholog/"' Cargo.toml
	@grep -q '^\[package.metadata.deb\]' Cargo.toml
	@grep -q '^\[package.metadata.generate-rpm\]' Cargo.toml
	@grep -q '^\[profile.release\]' Cargo.toml
	@grep -q '^strip = "symbols"' Cargo.toml
	@for keyword in cli file-system path utilities; do grep -q "\"$$keyword\"" Cargo.toml || exit 1; done
	@grep -q '"SECURITY.md"' Cargo.toml
	@grep -q '"THIRD-PARTY-LICENSES.md"' Cargo.toml
	@grep -q 'cargo install patholog' README.md
	@grep -q 'github.com/techouse/patholog/releases' README.md
	@! grep -Ei 'private v1 release candidate|final private v1 release candidate|pre-v1' README.md
	$(CARGO) package --locked --list --allow-dirty > $(PACKAGE_LIST)
	! grep -E '^(SECURITY\.md|THIRD-PARTY-LICENSES\.md)$$' $(PACKAGE_LIST)

install-smoke: ## Install packaged crate into a temporary root and smoke-test it
	@set -e; \
	version="$$(sed -n 's/^version = "\([^"]*\)"$$/\1/p' Cargo.toml | head -n 1)"; \
	if [ -z "$$version" ]; then echo "Could not read version from Cargo.toml" >&2; exit 1; fi; \
	$(CARGO) package --locked --allow-dirty --offline; \
	package_dir="target/package/patholog-$$version"; \
	if [ ! -d "$$package_dir" ]; then echo "missing package directory $$package_dir" >&2; exit 1; fi; \
	tmp="$$(mktemp -d)"; \
	trap 'rm -rf "$$tmp"' EXIT; \
	mkdir -p "$$tmp/cargo-home"; \
	source_cargo_home="$${CARGO_HOME:-$$HOME/.cargo}"; \
	if [ -d "$$source_cargo_home/registry" ]; then cp -R "$$source_cargo_home/registry" "$$tmp/cargo-home/registry"; fi; \
	if [ -d "$$source_cargo_home/git" ]; then cp -R "$$source_cargo_home/git" "$$tmp/cargo-home/git"; fi; \
	CARGO_HOME="$$tmp/cargo-home" CARGO_TARGET_DIR="$$tmp/target" $(CARGO) install --locked --offline --path "$$package_dir" --root "$$tmp/root"; \
	bin="$$tmp/root/bin/patholog"; \
	if [ ! -x "$$bin" ] && [ -x "$$bin.exe" ]; then bin="$$bin.exe"; fi; \
	"$$bin" --version | grep -q "patholog $$version"; \
	"$$bin" health --json | grep -q '"score"'

version-check: ## Check release version references agree
	@version="$$(sed -n 's/^version = "\([^"]*\)"$$/\1/p' Cargo.toml | head -n 1)"; \
	if [ -z "$$version" ]; then echo "Could not read version from Cargo.toml" >&2; exit 1; fi; \
	for file in Cargo.lock fuzz/Cargo.lock; do \
		awk -v version="$$version" 'BEGIN { found = 0; in_package = 0 } /^\[\[package\]\]/ { in_package = 0 } /^name = "patholog"$$/ { in_package = 1 } in_package && $$0 == "version = \"" version "\"" { found = 1 } END { exit !found }' "$$file" || { echo "$$file does not contain patholog $$version" >&2; exit 1; }; \
	done; \
	! grep -Eq 'patholog [0-9]+\.[0-9]+\.[0-9]+' src/cli/tests.rs tests/binary_cli.rs || { echo "version tests must not hardcode patholog versions" >&2; exit 1; }; \
	grep -q 'CARGO_PKG_VERSION' src/cli/tests.rs || { echo "src/cli/tests.rs does not derive version from Cargo metadata" >&2; exit 1; }; \
	grep -q 'CARGO_PKG_VERSION' tests/binary_cli.rs || { echo "tests/binary_cli.rs does not derive version from Cargo metadata" >&2; exit 1; }; \
	grep -q "^## $$version " CHANGELOG.md || { echo "CHANGELOG.md is missing $$version" >&2; exit 1; }; \
	printf 'version-check: %s\n' "$$version"

release-check: ## Run public-v1 readiness audit checks
	$(MAKE) version-check
	$(MAKE) docs
	$(MAKE) docs-missing
	$(MAKE) package-list
	$(MAKE) package-check-offline

v1-contract-check: ## Run v1 CLI, JSON, docs, and package contract checks
	$(MAKE) version-check
	$(MAKE) docs
	$(MAKE) docs-missing
	$(MAKE) test
	$(MAKE) test-doc
	$(MAKE) package-metadata-check
	$(MAKE) package-list
	$(MAKE) package-check-offline
	$(MAKE) install-smoke

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
	$(MAKE) fuzz-clippy
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
	$(MAKE) fuzz-clippy
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
