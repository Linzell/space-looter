#!/bin/bash

# Netlify Build Script for Space Looter
# This script installs Rust dependencies and runs the existing build-web.sh script

set -e

echo "🚀 Starting Netlify build for Space Looter..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Environment setup
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export RUST_LOG=${RUST_LOG:-error}

echo -e "${BLUE}📋 Setting up build environment...${NC}"

# Install Rust (always fresh install to avoid partial installations)
echo -e "${YELLOW}Installing Rust...${NC}"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

# Source cargo environment
source ~/.cargo/env
export PATH="$HOME/.cargo/bin:$PATH"

# Verify Rust installation
echo -e "${GREEN}✅ Rust installed: $(rustc --version)${NC}"
echo -e "${GREEN}✅ Cargo installed: $(cargo --version)${NC}"

# Add WASM target
echo -e "${BLUE}Adding WASM target...${NC}"
rustup target add wasm32-unknown-unknown

# Install wasm-pack
echo -e "${YELLOW}Installing wasm-pack...${NC}"
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Ensure wasm-pack is in PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Verify wasm-pack installation
echo -e "${GREEN}✅ wasm-pack installed: $(wasm-pack --version)${NC}"

# Make build script executable
chmod +x build-web.sh

# Show environment info
echo -e "${BLUE}📊 Build Environment:${NC}"
echo "  Rust: $(rustc --version)"
echo "  Cargo: $(cargo --version)"
echo "  wasm-pack: $(wasm-pack --version)"
echo "  Working directory: $(pwd)"

echo -e "${GREEN}✅ Environment setup complete! Running build script...${NC}"

# Run the existing build script
./build-web.sh

echo -e "${GREEN}🚀 Netlify build complete!${NC}"
