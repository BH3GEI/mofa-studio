#!/usr/bin/env python3
"""
Note Taker Plugin - Backend Server

A simple note-taking application demonstrating the MoFA Studio plugin system.
Features:
- Create, read, update, delete notes
- Persist notes to JSON file
- Search notes by title or content
- Dark mode theme support
"""

import json
import os
import sys
import uuid
from datetime import datetime
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from urllib.parse import urlparse, parse_qs


class NotesStorage:
    """Simple file-based notes storage."""

    def __init__(self, storage_path: Path):
        self.storage_path = storage_path
        self.notes_file = storage_path / "notes.json"
        self.notes = {}
        self._load()

    def _load(self):
        """Load notes from file."""
        if self.notes_file.exists():
            try:
                with open(self.notes_file, "r", encoding="utf-8") as f:
                    data = json.load(f)
                    self.notes = data.get("notes", {})
            except Exception as e:
                print(f"Error loading notes: {e}")
                self.notes = {}

    def _save(self):
        """Save notes to file."""
        self.storage_path.mkdir(parents=True, exist_ok=True)
        with open(self.notes_file, "w", encoding="utf-8") as f:
            json.dump({"notes": self.notes}, f, indent=2, ensure_ascii=False)

    def create(self, title: str, content: str) -> dict:
        """Create a new note."""
        note_id = str(uuid.uuid4())[:8]
        now = datetime.now().isoformat()
        note = {
            "id": note_id,
            "title": title,
            "content": content,
            "created_at": now,
            "updated_at": now,
        }
        self.notes[note_id] = note
        self._save()
        return note

    def get(self, note_id: str) -> dict | None:
        """Get a note by ID."""
        return self.notes.get(note_id)

    def get_all(self) -> list[dict]:
        """Get all notes sorted by updated_at descending."""
        notes = list(self.notes.values())
        notes.sort(key=lambda n: n.get("updated_at", ""), reverse=True)
        return notes

    def update(self, note_id: str, title: str = None, content: str = None) -> dict | None:
        """Update a note."""
        note = self.notes.get(note_id)
        if not note:
            return None
        if title is not None:
            note["title"] = title
        if content is not None:
            note["content"] = content
        note["updated_at"] = datetime.now().isoformat()
        self._save()
        return note

    def delete(self, note_id: str) -> bool:
        """Delete a note."""
        if note_id in self.notes:
            del self.notes[note_id]
            self._save()
            return True
        return False

    def search(self, query: str) -> list[dict]:
        """Search notes by title or content."""
        query = query.lower()
        results = []
        for note in self.notes.values():
            if query in note.get("title", "").lower() or query in note.get("content", "").lower():
                results.append(note)
        results.sort(key=lambda n: n.get("updated_at", ""), reverse=True)
        return results


# Global storage instance
storage: NotesStorage = None


class NoteHandler(SimpleHTTPRequestHandler):
    """HTTP request handler for the notes API."""

    def __init__(self, *args, **kwargs):
        # Set static directory
        self.static_dir = Path(__file__).parent / "static"
        super().__init__(*args, directory=str(self.static_dir), **kwargs)

    def do_GET(self):
        """Handle GET requests."""
        parsed = urlparse(self.path)
        path = parsed.path

        # API endpoints
        if path == "/api/notes":
            self._send_json({"notes": storage.get_all()})
        elif path.startswith("/api/notes/"):
            note_id = path.split("/")[-1]
            note = storage.get(note_id)
            if note:
                self._send_json(note)
            else:
                self._send_error(404, "Note not found")
        elif path == "/api/search":
            params = parse_qs(parsed.query)
            query = params.get("q", [""])[0]
            results = storage.search(query)
            self._send_json({"notes": results, "query": query})
        elif path == "/api/info":
            self._send_json({
                "name": "Note Taker",
                "version": "1.0.0",
                "note_count": len(storage.notes),
            })
        else:
            # Serve static files
            super().do_GET()

    def do_POST(self):
        """Handle POST requests."""
        parsed = urlparse(self.path)
        path = parsed.path

        if path == "/api/notes":
            data = self._get_json_body()
            if not data:
                self._send_error(400, "Invalid JSON body")
                return
            title = data.get("title", "Untitled")
            content = data.get("content", "")
            note = storage.create(title, content)
            self._send_json(note, 201)
        else:
            self._send_error(404, "Not found")

    def do_PUT(self):
        """Handle PUT requests."""
        parsed = urlparse(self.path)
        path = parsed.path

        if path.startswith("/api/notes/"):
            note_id = path.split("/")[-1]
            data = self._get_json_body()
            if not data:
                self._send_error(400, "Invalid JSON body")
                return
            note = storage.update(
                note_id,
                title=data.get("title"),
                content=data.get("content"),
            )
            if note:
                self._send_json(note)
            else:
                self._send_error(404, "Note not found")
        else:
            self._send_error(404, "Not found")

    def do_DELETE(self):
        """Handle DELETE requests."""
        parsed = urlparse(self.path)
        path = parsed.path

        if path.startswith("/api/notes/"):
            note_id = path.split("/")[-1]
            if storage.delete(note_id):
                self._send_json({"deleted": True})
            else:
                self._send_error(404, "Note not found")
        else:
            self._send_error(404, "Not found")

    def _get_json_body(self) -> dict | None:
        """Parse JSON body from request."""
        try:
            length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(length).decode("utf-8")
            return json.loads(body)
        except Exception:
            return None

    def _send_json(self, data: dict, status: int = 200):
        """Send JSON response."""
        body = json.dumps(data, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", len(body))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(body)

    def _send_error(self, status: int, message: str):
        """Send error response."""
        self._send_json({"error": message}, status)

    def do_OPTIONS(self):
        """Handle CORS preflight."""
        self.send_response(204)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()

    def log_message(self, format, *args):
        """Log HTTP requests."""
        print(f"[NoteTaker] {args[0]}")


def main():
    global storage

    # Get port from command line
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080

    # Initialize storage
    storage_path = Path.home() / "Documents" / "MoFANoteTaker"
    storage = NotesStorage(storage_path)
    print(f"Notes stored in: {storage_path}")

    # Start server
    server = HTTPServer(("127.0.0.1", port), NoteHandler)
    print(f"Note Taker server running on http://127.0.0.1:{port}")

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down...")
        server.shutdown()


if __name__ == "__main__":
    main()
