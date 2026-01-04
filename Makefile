# Makefile for tarzi - Rust-native lite search for AI applications

# Variables
CARGO = cargo
MATURIN = uv run maturin
PYTHON = python3
PYTEST = pytest
RUST_TARGET = target/release/tarzi
PYTHON_PACKAGE = target/wheels/*.whl
PYTHON_TEST_DIR = tests/python
PYTHON_MODULES = examples python

# Colors for output
BLUE = \033[34m
GREEN = \033[32m
RED = \033[31m
RESET = \033[0m

# =============================================================================
# HELP
# =============================================================================

.PHONY: help
help: ## Show this help message
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# =============================================================================
# BUILD COMMANDS
# =============================================================================

.PHONY: build
build: build-rust build-python ## Build everything (Rust + Python)

.PHONY: build-rust
build-rust: ## Build Rust binary in release mode
	$(CARGO) build --release

.PHONY: build-python
build-python: ## Build Python wheel
	$(MATURIN) build --release

# =============================================================================
# INSTALL COMMANDS
# =============================================================================

.PHONY: install
install: build-rust build-python
	cp $(RUST_TARGET) /usr/local/bin/tarzi
	pip install $(PYTHON_PACKAGE)

# =============================================================================
# TEST COMMANDS
# =============================================================================

.PHONY: install-dev
install-dev:
	uv sync --extra dev

.PHONY: test
test: test-unit test-integration ## Run all tests (unit + integration)

.PHONY: test-unit
test-unit: test-unit-rust test-unit-python ## Run all unit tests (Rust + Python)

.PHONY: test-unit-rust
test-unit-rust: ## Run Rust unit tests only
	$(CARGO) test --lib --features test-helpers

.PHONY: test-unit-python
test-unit-python: install-dev ## Run Python unit tests only
	uv run -m pytest -m unit -v

.PHONY: test-integration
test-integration: test-integration-rust test-integration-python ## Run all integration tests (Rust + Python)

.PHONY: test-integration-rust
test-integration-rust: ## Run Rust integration tests
	$(CARGO) test --test '*' --features test-helpers

.PHONY: test-integration-python
test-integration-python: install-dev ## Run Python integration tests only
	uv run -m pytest -m integration -v

# =============================================================================
# CODE QUALITY COMMANDS
# =============================================================================

.PHONY: format
format: install-dev ## Format code (Rust + Python)
	$(CARGO) fmt
	@uv run autoflake --in-place --recursive --remove-all-unused-imports --remove-unused-variables $(PYTHON_MODULES) || echo "autoflake not found, skipping Python auto-cleanup"
	@uv run isort $(PYTHON_MODULES) || echo "isort not found, skipping Python import sorting"
	@uv run black $(PYTHON_MODULES) || echo "black not found, skipping Python formatting"

.PHONY: lint
lint: install-dev ## Lint code (Rust + Python)
	$(CARGO) clippy --all-targets --all-features -- -D warnings
	@uv run ruff check $(PYTHON_MODULES) || echo "ruff not found, skipping Python linting"

.PHONY: check
check: install-dev lint
	$(CARGO) fmt -- --check
	@uv run black --check $(PYTHON_MODULES) || echo "black not found, skipping Python format check"
	@uv run isort --check-only $(PYTHON_MODULES) || echo "isort not found, skipping Python import check"
	$(CARGO) check

.PHONY: autofix
autofix: install-dev format ## Fix linting issues (Rust + Python)
	$(CARGO) clippy --fix --allow-dirty --allow-staged --all-targets --all-features -- -D warnings
	@uv run ruff check --fix $(PYTHON_MODULES) || echo "ruff not found, skipping Python lint fixes"

# =============================================================================
# CLEAN COMMANDS
# =============================================================================

.PHONY: clean
clean:  ## Clean everything including dependencies
	rm -rf target/
	rm -rf .venv/
	rm -rf __pycache__/
	rm -rf *.egg-info/
	$(CARGO) clean
	rm -rf target/wheels/
	rm -rf $(PYTHON_TEST_DIR)/.pytest_cache
	rm -rf $(PYTHON_TEST_DIR)/htmlcov
	rm -rf $(PYTHON_TEST_DIR)/.coverage
	find $(PYTHON_TEST_DIR) -name "__pycache__" -type d -exec rm -rf {} +
	find $(PYTHON_TEST_DIR) -name "*.pyc" -delete

# =============================================================================
# DOCUMENTATION COMMANDS
# =============================================================================

.PHONY: doc-build
doc-build:
	$(CARGO) doc --no-deps
	@cd docs && make html
	@echo "$(GREEN)Documentation built: docs/_build/html/index.html$(RESET)"

.PHONY: docs-clean
docs-clean: ## Clean documentation build artifacts
	@rm -rf docs/_build/

# =============================================================================
# RELEASE COMMANDS
# =============================================================================

.PHONY: release
release: release-rust release-python ## Build all release artifacts (Rust + Python)

.PHONY: release-rust
release-rust: ## Build release artifacts (Rust binary)
	$(CARGO) build --release

.PHONY: release-python
release-python: ## Build Python release artifacts
	$(MATURIN) build --release

.PHONY: publish
publish: publish-rust ## Publish Rust crate to crates.io (use with caution!)
	$(CARGO) publish --registry crates-io

.PHONY: publish-rust
publish-rust: ## Publish Rust crate to crates.io (use with caution!)
	$(CARGO) publish --registry crates-io

.PHONY: publish-python
publish-python: ## Publish Python package to PyPI
	@if [ -z "$(shell ls -A target/wheels/ 2>/dev/null)" ]; then \
		echo "$(RED)❌ No wheels found. Run 'make build-python' first.$(RESET)"; \
		exit 1; \
	fi
	twine check $(PYTHON_PACKAGE)
	twine upload $(PYTHON_PACKAGE)
	@echo "$(GREEN)✅ Package published to PyPI$(RESET)"

# =============================================================================
# UTILITY COMMANDS
# =============================================================================

.PHONY: version
version: ## Show current version
	@echo "$(BLUE)Current version:$(RESET)"
	@echo "Workspace (Cargo.toml): $(shell grep '^version = ' Cargo.toml | cut -d'"' -f2)"
	@echo "Python (pyproject.toml): $(shell grep '^version = ' pyproject.toml | cut -d'"' -f2)"

.PHONY: version-update
version-update:
	@if [ -z "$(VERSION)" ]; then \
		echo "$(RED)❌ VERSION parameter is required. Usage: make version-update VERSION=1.2.3$(RESET)"; \
		exit 1; \
	fi
	@echo "$(BLUE)Updating version to $(VERSION)...$(RESET)"
	@# Update workspace Cargo.toml
	@sed -i.bak 's/^version = ".*"/version = "$(VERSION)"/' Cargo.toml
	@rm -f Cargo.toml.bak
	@echo "$(GREEN)✅ Updated workspace Cargo.toml$(RESET)"
	@# Update pyproject.toml
	@sed -i.bak 's/^version = ".*"/version = "$(VERSION)"/' pyproject.toml
	@rm -f pyproject.toml.bak
	@echo "$(GREEN)✅ Updated pyproject.toml$(RESET)"
	@# Update Cargo.lock
	@$(CARGO) update
	@echo "$(GREEN)✅ Updated Cargo.lock$(RESET)"
	@echo "$(GREEN)✅ Version updated to $(VERSION)$(RESET)"

.PHONY: setup
setup: ## Setup development environment
	rustup update
	$(CARGO) install cargo-outdated
	pip install -e .[dev]
	@echo "$(GREEN)✅ Development environment ready$(RESET)"

.PHONY: setup-docs
setup-docs: ## Setup documentation development environment
	pip install -r docs/requirements.txt
	@echo "$(GREEN)✅ Documentation environment ready$(RESET)"

.PHONY: setup-drivers
setup-drivers: ## Install WebDriver dependencies (requires brew on macOS/Linux)
	@echo "$(BLUE)Installing WebDriver dependencies...$(RESET)"
	@if command -v brew >/dev/null 2>&1; then \
		echo "$(GREEN)Installing ChromeDriver and GeckoDriver via brew...$(RESET)"; \
		brew install --cask chromedriver || echo "$(RED)Failed to install chromedriver$(RESET)"; \
		brew install geckodriver || echo "$(RED)Failed to install geckodriver$(RESET)"; \
		echo "$(GREEN)✅ WebDriver dependencies installed$(RESET)"; \
	else \
		echo "$(RED)❌ Homebrew not found. Please install drivers manually:$(RESET)"; \
		echo "  - ChromeDriver: https://chromedriver.chromium.org/"; \
		echo "  - GeckoDriver: https://github.com/mozilla/geckodriver/releases"; \
		echo "  - Or install Homebrew: https://brew.sh/"; \
		exit 1; \
	fi

.PHONY: setup-full
setup-full: setup setup-drivers setup-docs ## Complete development environment setup

# =============================================================================
# DEVELOPMENT COMMANDS
# =============================================================================

.PHONY: dev
dev: ## Run in development mode (debug build)
	$(CARGO) run -p tarzi

.PHONY: dev-release
dev-release: ## Run in development mode (release build)
	$(CARGO) run --release -p tarzi

.PHONY: dev-check
dev-check: check test-unit ## Quick development check (check + unit tests)

.PHONY: full-check
full-check: check test build ## Full development check (all check + all tests + build)

.PHONY: run-examples
run-examples: run-examples-rust run-examples-python ## Run all examples (Rust + Python)

.PHONY: run-examples-rust
run-examples-rust: ## Run all Rust examples
	@echo "$(BLUE)Running Rust examples...$(RESET)"
	@for example in basic_usage browser_driver_usage sogou_weixin_search simple_usage; do \
		echo "$(GREEN)Running example: $$example$(RESET)"; \
		$(CARGO) run --example $$example || echo "$(RED)Example $$example failed$(RESET)"; \
		echo ""; \
	done
	@echo "$(GREEN)✅ All Rust examples completed$(RESET)"

.PHONY: run-examples-python
run-examples-python: install-dev ## Run all Python examples
	@echo "$(BLUE)Running Python examples...$(RESET)"
	@for example in basic_usage.py sogou_weixin_search.py; do \
		if [ -f "examples/$$example" ]; then \
			echo "$(GREEN)Running example: $$example$(RESET)"; \
			uv run python "examples/$$example" || echo "$(RED)Example $$example failed$(RESET)"; \
			echo ""; \
		fi; \
	done
	@echo "$(GREEN)✅ All Python examples completed$(RESET)" 
