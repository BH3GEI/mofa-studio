"""
Hello World Plugin - Example plugin for MoFA Studio
"""

import sys
import json
from pathlib import Path
from http.server import HTTPServer, SimpleHTTPRequestHandler

class PluginHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        self.directory = str(Path(__file__).parent / "static")
        super().__init__(*args, directory=self.directory, **kwargs)

    def do_GET(self):
        if self.path == "/api/info":
            self._json_response({
                "name": "Hello World Plugin",
                "version": "1.0.0",
                "message": "This is a simple example plugin!"
            })
        elif self.path == "/api/time":
            import datetime
            self._json_response({
                "time": datetime.datetime.now().isoformat()
            })
        else:
            super().do_GET()

    def do_POST(self):
        if self.path == "/api/greet":
            data = self._read_json()
            name = data.get("name", "World") if data else "World"
            self._json_response({
                "greeting": f"Hello, {name}! Welcome to MoFA Studio plugins."
            })
        else:
            self._json_response({"error": "Not found"}, 404)

    def _read_json(self):
        try:
            length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(length)
            return json.loads(body.decode("utf-8"))
        except:
            return None

    def _json_response(self, data, status=200):
        body = json.dumps(data, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, format, *args):
        print(f"[HelloWorld] {args[0]}")


def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8090
    server = HTTPServer(("127.0.0.1", port), PluginHandler)
    print(f"Hello World plugin running on http://127.0.0.1:{port}")
    server.serve_forever()


if __name__ == "__main__":
    main()
