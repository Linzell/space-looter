#!/bin/bash

# Space Looter Web Build Script
# This script builds the game for web deployment with cross-browser compatibility

set -e

echo "🚀 Building Space Looter for Web..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if required tools are installed
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo -e "${RED}❌ $1 is not installed. Please install it first.${NC}"
        echo "   Installation: $2"
        exit 1
    fi
}

echo -e "${BLUE}📋 Checking prerequisites...${NC}"
check_tool "rustc" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
check_tool "wasm-pack" "cargo install wasm-pack --locked"

# Show version information for debugging
echo -e "${BLUE}🔍 Build environment versions:${NC}"
echo "  Rust: $(rustc --version)"
echo "  Cargo: $(cargo --version)"
echo "  wasm-pack: $(wasm-pack --version)"
echo "  Target: wasm32-unknown-unknown"

# Add WASM target if not already added
echo -e "${BLUE}🎯 Setting up WASM target...${NC}"
if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    echo -e "${YELLOW}Adding wasm32-unknown-unknown target...${NC}"
    rustup target add wasm32-unknown-unknown
else
    echo -e "${GREEN}✅ WASM target already installed${NC}"
fi

# Clean previous builds
echo -e "${BLUE}🧹 Cleaning previous builds...${NC}"
rm -rf pkg/
rm -rf dist/
rm -rf web/space_looter.js
rm -rf web/space_looter_bg.wasm
rm -rf web/space_looter.d.ts

# Build with wasm-pack
echo -e "${BLUE}🔨 Building WASM package...${NC}"
wasm-pack build \
    --target web \
    --out-dir pkg \
    --release \
    --no-typescript

# Generate timestamp for cache busting
TIMESTAMP=$(date +%s)
echo -e "${BLUE}🕐 Using timestamp: ${TIMESTAMP} for cache busting${NC}"

# Check if build was successful
if [ ! -f "pkg/space_looter.js" ]; then
    echo -e "${RED}❌ Build failed - WASM files not generated${NC}"
    exit 1
fi

# Debug: Show wasm-bindgen version used in build
echo -e "${BLUE}🔍 Debugging build information...${NC}"
if command -v wasm-bindgen &> /dev/null; then
    BINDGEN_VERSION=$(wasm-bindgen --version)
    echo -e "${GREEN}✅ wasm-bindgen CLI version: ${BINDGEN_VERSION}${NC}"
else
    echo -e "${YELLOW}⚠️ wasm-bindgen CLI not found${NC}"
fi

# Show generated file info
echo -e "${BLUE}📊 Generated files info:${NC}"
echo "  JS file size: $(wc -c < pkg/space_looter.js) bytes"
echo "  WASM file size: $(wc -c < pkg/space_looter_bg.wasm) bytes"
echo "  JS file MD5: $(md5sum pkg/space_looter.js | cut -d' ' -f1)"
echo "  WASM file MD5: $(md5sum pkg/space_looter_bg.wasm | cut -d' ' -f1)"

# Check for wasm-bindgen generated functions
echo -e "${BLUE}🔍 Checking for closure wrappers in JS...${NC}"
if grep -q "__wbindgen_closure_wrapper" pkg/space_looter.js; then
    WRAPPER_COUNT=$(grep -c "__wbindgen_closure_wrapper" pkg/space_looter.js)
    echo -e "${GREEN}✅ Found ${WRAPPER_COUNT} closure wrappers in JS${NC}"
    echo "  First few wrappers:"
    grep "__wbindgen_closure_wrapper" pkg/space_looter.js | head -3 | sed 's/^/    /'
else
    echo -e "${RED}❌ No closure wrappers found in JS${NC}"
fi

# Create dist directory and copy files
echo -e "${BLUE}📁 Creating dist directory and copying files...${NC}"
mkdir -p dist

# Copy WASM and JS files
cp pkg/space_looter.js dist/
cp pkg/space_looter_bg.wasm dist/

# Update the index.html to use timestamp for cache busting
echo -e "${BLUE}🔄 Adding cache busting with timestamp ${TIMESTAMP}...${NC}"
sed -e "s/import('\.\/space_looter\.js')/import('.\/space_looter.js?v=${TIMESTAMP}')/g" \
    -e "s/wasmModule = await init();/wasmModule = await init('.\/space_looter_bg.wasm?v=${TIMESTAMP}');/g" \
    -e "s/href=\"\.\/space_looter\.js\"/href=\".\/space_looter.js?v=${TIMESTAMP}\"/g" \
    web/index.html > dist/index.html

if [ -f "pkg/space_looter.d.ts" ]; then
    cp pkg/space_looter.d.ts dist/
fi

echo -e "${GREEN}✅ Files copied with cache busting timestamp: ${TIMESTAMP}${NC}"

# Verify cache busting was applied
echo -e "${BLUE}🔍 Verifying cache busting in HTML...${NC}"
if grep -q "space_looter\.js?v=${TIMESTAMP}" dist/index.html; then
    echo -e "${GREEN}✅ JS cache busting applied successfully${NC}"
else
    echo -e "${RED}❌ JS cache busting failed${NC}"
fi

if grep -q "space_looter_bg\.wasm?v=${TIMESTAMP}" dist/index.html; then
    echo -e "${GREEN}✅ WASM cache busting applied successfully${NC}"
else
    echo -e "${RED}❌ WASM cache busting failed${NC}"
fi

echo -e "${BLUE}📋 Cache busting summary:${NC}"
echo "  - Timestamp: ${TIMESTAMP}"
echo "  - JS URL: space_looter.js?v=${TIMESTAMP}"
echo "  - WASM URL: space_looter_bg.wasm?v=${TIMESTAMP}"

# Create build info file for debugging
echo -e "${BLUE}📋 Creating build info file...${NC}"
cat > dist/build-info.json << EOF
{
  "buildTimestamp": "${TIMESTAMP}",
  "buildDate": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "wasmPackVersion": "$(wasm-pack --version 2>/dev/null || echo 'unknown')",
  "rustcVersion": "$(rustc --version 2>/dev/null || echo 'unknown')",
  "jsFileHash": "$(md5sum pkg/space_looter.js | cut -d' ' -f1)",
  "wasmFileHash": "$(md5sum pkg/space_looter_bg.wasm | cut -d' ' -f1)",
  "jsFileSize": $(wc -c < pkg/space_looter.js),
  "wasmFileSize": $(wc -c < pkg/space_looter_bg.wasm),
  "cacheBreaker": "v=${TIMESTAMP}"
}
EOF

# Copy assets directory if it exists
if [ -d "assets" ]; then
    echo -e "${BLUE}🎵 Copying game assets (audio, fonts, icons)...${NC}"
    cp -r assets dist/
    echo -e "${GREEN}✅ Assets copied successfully${NC}"

    # Verify audio assets were copied
    if [ -d "dist/assets/audio" ]; then
        echo -e "${GREEN}✅ Audio assets available for web build${NC}"
    else
        echo -e "${YELLOW}⚠️  No audio directory found in assets${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  No assets directory found - audio and graphics may not work${NC}"
fi

# Optimize WASM file (if wasm-opt is available)
if command -v wasm-opt &> /dev/null; then
    echo -e "${BLUE}⚡ Optimizing WASM file...${NC}"
    wasm-opt -Oz -o dist/space_looter_bg.wasm dist/space_looter_bg.wasm
else
    echo -e "${YELLOW}⚠️  wasm-opt not found - skipping WASM optimization${NC}"
    echo "   Install with: npm install -g wasm-opt"
fi

# Generate file sizes for reference
echo -e "${BLUE}📊 Build statistics:${NC}"
if command -v du &> /dev/null; then
    echo "   WASM file: $(du -h dist/space_looter_bg.wasm | cut -f1)"
    echo "   JS file: $(du -h dist/space_looter.js | cut -f1)"
    echo "   HTML file: $(du -h dist/index.html | cut -f1)"
fi

# Create a simple HTTP server script for testing
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

# Create Node.js server as alternative
cat > dist/serve.js << 'EOF'
/**
 * Simple Node.js HTTP server for testing Space Looter web build.
 * Run with: node serve.js
 */
const http = require('http');
const fs = require('fs');
const path = require('path');
const { exec } = require('child_process');

const PORT = 8080;
const HOST = 'localhost';

const mimeTypes = {
    '.html': 'text/html',
    '.js': 'application/javascript',
    '.wasm': 'application/wasm',
    '.css': 'text/css',
    '.png': 'image/png',
    '.jpg': 'image/jpeg',
    '.ico': 'image/x-icon'
};

const server = http.createServer((req, res) => {
    let filePath = path.join(__dirname, req.url === '/' ? 'index.html' : req.url);

    fs.readFile(filePath, (err, content) => {
        if (err) {
            if (err.code === 'ENOENT') {
                res.writeHead(404, { 'Content-Type': 'text/plain' });
                res.end('File not found');
            } else {
                res.writeHead(500);
                res.end('Server error');
            }
        } else {
            const ext = path.extname(filePath).toLowerCase();
            const contentType = mimeTypes[ext] || 'application/octet-stream';

            res.writeHead(200, {
                'Content-Type': contentType,
                'Cross-Origin-Embedder-Policy': 'require-corp',
                'Cross-Origin-Opener-Policy': 'same-origin'
            });
            res.end(content);
        }
    });
});

server.listen(PORT, HOST, () => {
    console.log(`🌐 Server running at http://${HOST}:${PORT}/`);
    console.log(`📁 Serving files from: ${__dirname}`);
    console.log(`🎮 Open http://${HOST}:${PORT} in your browser to play!`);
    console.log(`⏹️  Press Ctrl+C to stop the server`);

    // Try to open browser
    const start = process.platform === 'darwin' ? 'open' :
                  process.platform === 'win32' ? 'start' : 'xdg-open';
    exec(`${start} http://${HOST}:${PORT}`);
});

process.on('SIGINT', () => {
    console.log('\n🛑 Server stopped.');
    process.exit(0);
});
EOF

echo -e "${GREEN}✅ Build completed successfully!${NC}"
echo ""
echo -e "${BLUE}🎮 To test your game:${NC}"
echo "   1. Python server: cd dist && python serve.py"
echo "   2. Node.js server: cd dist && node serve.js"
echo "   3. Or use any static file server in the 'dist' directory"
echo ""
echo -e "${BLUE}📁 Files generated:${NC}"
echo "   - dist/index.html (Game HTML page)"
echo "   - dist/space_looter.js (Generated JavaScript)"
echo "   - dist/space_looter_bg.wasm (Game WASM binary)"
echo "   - dist/serve.py (Python test server)"
echo "   - dist/serve.js (Node.js test server)"
echo ""
echo -e "${BLUE}🌐 Deploy the 'dist' directory to any static hosting service:${NC}"
echo "   - GitHub Pages"
echo "   - Netlify"
echo "   - Vercel"
echo "   - Firebase Hosting"
echo "   - Or any web server"
echo ""
echo -e "${GREEN}🚀 Space Looter is ready for web deployment!${NC}"
