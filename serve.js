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
