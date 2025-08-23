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
