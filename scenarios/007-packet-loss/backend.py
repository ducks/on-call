from http.server import HTTPServer, BaseHTTPRequestHandler

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.end_headers()
        self.wfile.write(b"pong")

    def log_message(self, format, *args):
        pass

if __name__ == "__main__":
    print("backend listening on :9000")
    HTTPServer(("0.0.0.0", 9000), Handler).serve_forever()
