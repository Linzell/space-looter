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
check_tool "wasm-pack" "curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"

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

# Copy assets directory if it exists
if [ -d "assets" ]; then
    echo -e "${BLUE}🎵 Copying game assets (audio, fonts, icons)...${NC}"
    cp -r assets dist/
    echo -e "${GREEN}✅ Assets copied successfully${NC}"
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
