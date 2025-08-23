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

# Run a lightweight build instead of the full build-web.sh to avoid timeout
echo -e "${BLUE}🔨 Building WASM package without heavy optimization...${NC}"

# Clean previous builds
rm -rf pkg/
rm -rf dist/

# Build with wasm-pack but skip wasm-opt to avoid timeout
wasm-pack build \
    --target web \
    --out-dir pkg \
    --release \
    --no-typescript \
    --no-opt

# Check if build was successful
if [ ! -f "pkg/space_looter.js" ]; then
    echo -e "${RED}❌ Build failed - WASM files not generated${NC}"
    exit 1
fi

# Create dist directory and copy files
echo -e "${BLUE}📁 Creating dist directory and copying files...${NC}"
mkdir -p dist
cp web/index.html dist/
cp pkg/space_looter.js dist/
cp pkg/space_looter_bg.wasm dist/
if [ -f "pkg/space_looter.d.ts" ]; then
    cp pkg/space_looter.d.ts dist/
fi

# Generate file sizes for reference
echo -e "${BLUE}📊 Build statistics:${NC}"
if command -v du &> /dev/null; then
    echo "   WASM file: $(du -h dist/space_looter_bg.wasm | cut -f1)"
    echo "   JS file: $(du -h dist/space_looter.js | cut -f1)"
    echo "   HTML file: $(du -h dist/index.html | cut -f1)"
fi

# Create server files (copying from build-web.sh)
cat > dist/serve.py << 'EOF'
#!/usr/bin/env python3
"""
Simple HTTP server for testing Space Looter web build locally.
Run with: python serve.py
"""
import http.server
import socketserver
import webbrowser
import os
import sys

PORT = 8080

class CORSRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        super().end_headers()

# Change to web directory
os.chdir(os.path.dirname(os.path.abspath(__file__)))

try:
    with socketserver.TCPServer(("", PORT), CORSRequestHandler) as httpd:
        print(f"🌐 Server starting at http://localhost:{PORT}")
        print(f"📁 Serving files from: {os.getcwd()}")
        print(f"🎮 Open http://localhost:{PORT} in your browser to play!")
        print(f"⏹️  Press Ctrl+C to stop the server")

        # Try to open browser automatically
        try:
            webbrowser.open(f'http://localhost:{PORT}')
        except:
            pass

        httpd.serve_forever()
except KeyboardInterrupt:
    print("\n🛑 Server stopped.")
    sys.exit(0)
except OSError as e:
    if "Address already in use" in str(e):
        print(f"❌ Port {PORT} is already in use. Try a different port or stop the existing server.")
        sys.exit(1)
    else:
        raise
EOF

chmod +x dist/serve.py

echo -e "${GREEN}🚀 Netlify build complete!${NC}"
