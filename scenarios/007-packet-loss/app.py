import os
import urllib.request
import urllib.error
from http.server import HTTPServer, BaseHTTPRequestHandler

BACKEND = os.environ.get("BACKEND_URL", "http://backend:9000/ping")

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/health":
            try:
                urllib.request.urlopen(BACKEND, timeout=2)
                self.send_response(200)
                self.end_headers()
                self.wfile.write(b"ok")
            except Exception as e:
                self.send_response(503)
                self.end_headers()
                self.wfile.write(f"backend error: {e}".encode())
        else:
            self.send_response(404)
            self.end_headers()

    def log_message(self, format, *args):
        pass

if __name__ == "__main__":
    print("app listening on :8080")
    HTTPServer(("0.0.0.0", 8080), Handler).serve_forever()
