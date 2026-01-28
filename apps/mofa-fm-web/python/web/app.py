"""
MoFA.fm Web Backend

Serves a static page that embeds https://mofa.fm/.
"""

import sys
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path


class AppHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        static_dir = Path(__file__).parent / "static"
        super().__init__(*args, directory=str(static_dir), **kwargs)

    def log_message(self, format, *args):
        print(f"[MoFA.fm] {args[0]}")


def main() -> None:
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8095
    server = HTTPServer(("127.0.0.1", port), AppHandler)
    print(f"MoFA.fm web server running on http://127.0.0.1:{port}")
    server.serve_forever()


if __name__ == "__main__":
    main()
