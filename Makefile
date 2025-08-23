# Space Looter Makefile
# Development commands for Rust/Bevy game with web deployment

# Variables
PROJECT_NAME := space-looter
WEB_DIR := dist
PKG_DIR := pkg
CARGO := cargo
WASM_PACK := wasm-pack

# Colors for output
BLUE := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[1;33m
RED := \033[0;31m
NC := \033[0m # No Color

.PHONY: help setup build run test clean web serve deploy check lint format install-deps

# Default target
help: ## Show this help message
	@echo "$(BLUE)🚀 Space Looter - Simple Commands$(NC)"
	@echo ""
	@echo "$(GREEN)make setup$(NC)   - Install all dependencies and tools"
	@echo "$(GREEN)make dev$(NC)     - Start development (choose native/web)"
	@echo "$(GREEN)make build$(NC)   - Build for production (native + web)"
	@echo "$(GREEN)make test$(NC)    - Run all tests"
	@echo "$(GREEN)make clean$(NC)   - Clean all build files"
	@echo ""
	@echo "$(YELLOW)First time? Run: make setup && make dev$(NC)"

setup: ## Install all dependencies and tools
	@echo "$(BLUE)Setting up development environment...$(NC)"
	@echo "$(YELLOW)Installing Rust WASM target...$(NC)"
	rustup target add wasm32-unknown-unknown
	@echo "$(YELLOW)Checking for wasm-pack...$(NC)"
	@if ! command -v wasm-pack &> /dev/null; then \
		echo "$(YELLOW)Installing latest wasm-pack...$(NC)"; \
		cargo install wasm-pack --locked; \
	else \
		echo "$(GREEN)✅ wasm-pack already installed$(NC)"; \
		echo "$(YELLOW)Updating to latest version...$(NC)"; \
		cargo install wasm-pack --locked --force; \
	fi
	@echo "$(YELLOW)Checking for cargo-watch...$(NC)"
	@if ! command -v cargo-watch &> /dev/null; then \
		echo "$(YELLOW)Installing cargo-watch...$(NC)"; \
		$(CARGO) install cargo-watch; \
	else \
		echo "$(GREEN)✅ cargo-watch already installed$(NC)"; \
	fi
	@echo "$(GREEN)✅ Setup complete!$(NC)"

status: ## Check development environment status
	@echo "$(BLUE)🔍 Space Looter Development Environment Status$(NC)"
	@echo ""
	@echo "$(YELLOW)Core Tools:$(NC)"
	@if command -v rustc &> /dev/null; then \
		echo "  $(GREEN)✅ Rust:$(NC) $$(rustc --version)"; \
	else \
		echo "  $(RED)❌ Rust not installed$(NC)"; \
	fi
	@if command -v cargo &> /dev/null; then \
		echo "  $(GREEN)✅ Cargo:$(NC) $$(cargo --version)"; \
	else \
		echo "  $(RED)❌ Cargo not installed$(NC)"; \
	fi
	@if rustup target list --installed | grep -q "wasm32-unknown-unknown"; then \
		echo "  $(GREEN)✅ WASM target installed$(NC)"; \
	else \
		echo "  $(RED)❌ WASM target not installed$(NC) - run 'make setup'"; \
	fi
	@if command -v wasm-pack &> /dev/null; then \
		echo "  $(GREEN)✅ wasm-pack:$(NC) $$(wasm-pack --version)"; \
	else \
		echo "  $(RED)❌ wasm-pack not installed$(NC) - run 'make setup'"; \
	fi
	@if command -v cargo-watch &> /dev/null; then \
		echo "  $(GREEN)✅ cargo-watch:$(NC) $$(cargo watch --version 2>/dev/null | head -1)"; \
	else \
		echo "  $(RED)❌ cargo-watch not installed$(NC) - run 'make setup'"; \
	fi
	@echo ""
	@echo "$(YELLOW)Optional Tools:$(NC)"
	@if command -v python3 &> /dev/null; then \
		echo "  $(GREEN)✅ Python 3:$(NC) $$(python3 --version)"; \
	elif command -v python &> /dev/null; then \
		echo "  $(GREEN)✅ Python:$(NC) $$(python --version)"; \
	else \
		echo "  $(YELLOW)⚠️  Python not found$(NC) - needed for local web server"; \
	fi
	@if command -v node &> /dev/null; then \
		echo "  $(GREEN)✅ Node.js:$(NC) $$(node --version)"; \
	else \
		echo "  $(YELLOW)⚠️  Node.js not found$(NC) - alternative for local web server"; \
	fi
	@if command -v wasm-opt &> /dev/null; then \
		echo "  $(GREEN)✅ wasm-opt:$(NC) available for optimization"; \
	else \
		echo "  $(YELLOW)⚠️  wasm-opt not found$(NC) - install with 'npm install -g wasm-opt'"; \
	fi
	@echo ""
	@echo "$(YELLOW)Project Status:$(NC)"
	@if [ -f "Cargo.toml" ]; then \
		echo "  $(GREEN)✅ Cargo project detected$(NC)"; \
	else \
		echo "  $(RED)❌ No Cargo.toml found$(NC)"; \
	fi
	@if [ -d "src" ]; then \
		echo "  $(GREEN)✅ Source directory exists$(NC)"; \
	else \
		echo "  $(RED)❌ No src directory found$(NC)"; \
	fi
	@if [ -f "web/index.html" ]; then \
		echo "  $(GREEN)✅ Web template found$(NC)"; \
	else \
		echo "  $(RED)❌ No web/index.html template$(NC)"; \
	fi

build: ## Build for production (native + web)
	@echo "$(BLUE)Building $(PROJECT_NAME) for production...$(NC)"
	@echo "$(YELLOW)Building native version...$(NC)"
	$(CARGO) build --release
	@echo "$(YELLOW)Building web version...$(NC)"
	./build-web.sh
	@if [ ! -d "$(WEB_DIR)" ] || [ ! -f "$(WEB_DIR)/index.html" ]; then \
		echo "$(RED)❌ Web build failed$(NC)"; \
		exit 1; \
	fi
	@echo "$(GREEN)✅ Production builds complete!$(NC)"
	@echo "$(BLUE)Files ready:$(NC)"
	@echo "  - Native: target/release/$(PROJECT_NAME)"
	@echo "  - Web: $(WEB_DIR)/ directory"

test: ## Run all tests
	@echo "$(BLUE)Running tests...$(NC)"
	$(CARGO) fmt
	$(CARGO) clippy -- -D warnings
	$(CARGO) test
	@echo "$(GREEN)✅ All tests passed!$(NC)"

clean: ## Clean all build files
	@echo "$(BLUE)Cleaning build files...$(NC)"
	$(CARGO) clean
	rm -rf $(PKG_DIR)/
	rm -rf $(WEB_DIR)/
	@echo "$(GREEN)✅ Clean complete!$(NC)"

dev: ## Start development (choose native/web)
	@echo "$(BLUE)🚀 Choose Development Mode:$(NC)"
	@echo ""
	@echo "$(YELLOW)1.$(NC) Native (hot reload)    - Fast rebuilds, native performance"
	@echo "$(YELLOW)2.$(NC) Web (auto-rebuild)     - Test in browser, slower rebuilds"
	@echo ""
	@read -p "Choose (1-2): " choice; \
	case $$choice in \
		1) echo "$(BLUE)Starting native development...$(NC)" && make dev-native ;; \
		2) echo "$(BLUE)Starting web development...$(NC)" && make dev-web ;; \
		*) echo "$(RED)Invalid choice$(NC)" && exit 1 ;; \
	esac

dev-native: ## Native development with hot reload (internal)
	@if ! command -v cargo-watch &> /dev/null; then \
		echo "$(YELLOW)Installing cargo-watch...$(NC)"; \
		$(CARGO) install cargo-watch; \
	fi
	@echo "$(YELLOW)Starting native development with hot reload...$(NC)"
	@echo "$(YELLOW)Press Ctrl+C to stop. Files will auto-rebuild on changes.$(NC)"
	$(CARGO) watch -x run

dev-web: ## Web development with auto-rebuild (internal)
	@if ! command -v cargo-watch &> /dev/null; then \
		echo "$(YELLOW)Installing cargo-watch...$(NC)"; \
		$(CARGO) install cargo-watch; \
	fi
	@echo "$(YELLOW)Building initial web version...$(NC)"
	@./build-web.sh
	@if [ ! -d "$(WEB_DIR)" ] || [ ! -f "$(WEB_DIR)/index.html" ]; then \
		echo "$(RED)❌ Web build failed - $(WEB_DIR) directory or files missing$(NC)"; \
		exit 1; \
	fi
	@echo "$(YELLOW)Starting web server with auto-rebuild...$(NC)"
	@echo "$(YELLOW)File watcher will rebuild on changes. Refresh browser to see updates.$(NC)"
	@echo "$(YELLOW)Starting file watcher in background...$(NC)"
	@trap 'echo "$(RED)Stopping...$(NC)"; pkill -f "cargo watch" 2>/dev/null || true; exit 0' INT TERM; \
	$(CARGO) watch -w src -w web -s "wasm-pack build --target web --out-dir pkg --release --no-typescript && cp pkg/space_looter.js $(WEB_DIR)/ && cp pkg/space_looter_bg.wasm $(WEB_DIR)/ && echo '$(GREEN)✅ Web build updated! Refresh browser.$(NC)'" & \
	sleep 2; \
	if command -v python3 &> /dev/null; then \
		cd $(WEB_DIR) && python3 serve.py; \
	elif command -v python &> /dev/null; then \
		cd $(WEB_DIR) && python serve.py; \
	elif command -v node &> /dev/null; then \
		cd $(WEB_DIR) && node serve.js; \
	else \
		echo "$(RED)❌ Need Python or Node.js for local server$(NC)"; \
		exit 1; \
	fi

# Internal targets (don't show in help)
.PHONY: help setup build test clean dev dev-native dev-web
