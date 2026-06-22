# Cathedral-LLM — Makefile
# Arquiteto ORCID 0009-0005-2697-4668

.PHONY: all build test lint fmt check clean install dev docs bench release docker

# Variáveis
RUSTFLAGS ?= "-C target-cpu=native"
CARGO := cargo
DOCKER := docker
PYTHON := python3

# Default target
all: fmt lint test build

# === BUILD ===

build:
	@echo "🏛️  Building Cathedral-LLM workspace..."
	$(CARGO) build --workspace

build-release:
	@echo "🏛️  Building Cathedral-LLM (release)..."
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) build --workspace --release

build-core:
	@echo "🏛️  Building cathedral-llm-core..."
	$(CARGO) build -p cathedral-llm-core --release

build-runtime:
	@echo "🏛️  Building cathedral-inference-runtime..."
	$(CARGO) build -p cathedral-inference-runtime --release

build-api:
	@echo "🏛️  Building cathedral-api..."
	$(CARGO) build -p cathedral-api --release

build-cli:
	@echo "🏛️  Building cathedral-cli..."
	$(CARGO) build -p cathedral-cli --release

# === TEST ===

test:
	@echo "🏛️  Running all tests..."
	$(CARGO) test --workspace

test-core:
	@echo "🏛️  Testing cathedral-llm-core..."
	$(CARGO) test -p cathedral-llm-core

test-runtime:
	@echo "🏛️  Testing cathedral-inference-runtime..."
	$(CARGO) test -p cathedral-inference-runtime

test-e2e:
	@echo "🏛️  Running end-to-end tests..."
	$(CARGO) test -p cathedral-tests -- --test-threads=1 --nocapture

test-zk:
	@echo "🏛️  Testing ZK proofs..."
	$(CARGO) test -p cathedral-zk

test-identity:
	@echo "🏛️  Testing identity verification..."
	$(CARGO) test -p cathedral-identity

# === LINT ===

lint:
	@echo "🏛️  Running clippy..."
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

fmt:
	@echo "🏛️  Formatting code..."
	$(CARGO) fmt --all

check:
	@echo "🏛️  Running cargo check..."
	$(CARGO) check --workspace

deny:
	@echo "🏛️  Running cargo deny..."
	cargo deny check

# === UTILITIES ===

clean:
	@echo "🏛️  Cleaning build artifacts..."
	$(CARGO) clean
	@rm -rf target/
	@find . -name "*.rs.bk" -delete

install:
	@echo "🏛️  Installing Cathedral-LLM CLI..."
	$(CARGO) install --path crates/cathedral-cli

update:
	@echo "🏛️  Updating dependencies..."
	$(CARGO) update

help:
	@echo "🏛️  Cathedral-LLM — Available targets:"
	@echo ""
	@echo "  Build:"
	@echo "    build, build-release, build-core, build-runtime, build-api, build-cli"
	@echo ""
	@echo "  Test:"
	@echo "    test, test-core, test-runtime, test-e2e, test-zk, test-identity"
	@echo ""
	@echo "  Lint:"
	@echo "    lint, fmt, check, deny"
	@echo ""
	@echo "  Utilities:"
	@echo "    clean, install, update, help"
