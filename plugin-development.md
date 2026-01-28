# MoFA Studio Plugin Development Guide

> Note: Plugins are loaded from `~/.mofa-studio/plugins/` directory.
> WebView plugins use a Python HTTP backend. If you want a Rust HTTP server, build a native app (compiled into the shell).

This guide explains how to create plugins for MoFA Studio. The system supports two types of plugins:

1. **Native plugins (apps)**: Written in Rust + Makepad, compiled into the application
2. **WebView plugins**: Written in Python + HTML, dynamically loaded at runtime

This guide focuses on WebView plugins, which are easier to develop and don't require recompiling the main application. If you need the same WebView pattern but with a Rust backend, see "Rust Backend (Embedded App)" below.

## Table of Contents

- [Quick Start](#quick-start)
- [Plugin Structure](#plugin-structure)
- [Manifest File](#manifest-file)
- [Python Backend](#python-backend)
- [Rust Backend (Embedded App)](#rust-backend-embedded-app)
- [Frontend HTML](#frontend-html)
- [Theme Support](#theme-support)
- [API Design](#api-design)
- [Examples](#examples)
- [Debugging](#debugging)

## Quick Start

1. Create a plugin directory:

```bash
mkdir -p ~/.mofa-studio/plugins/my-plugin/{python,static}
```

2. Create `manifest.json`:

```json
{
  "id": "my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "A short description of my plugin",
  "author": "Your Name",
  "type": "webview",
  "python_entry": "python/app.py",
  "static_dir": "static",
  "show_in_sidebar": true
}
```

3. Create `python/app.py` (Python HTTP server)

4. Create `static/index.html` (Web UI)

5. Restart MoFA Studio - your plugin will appear in the sidebar

If you want a Rust backend instead of Python, build a native app and embed the server (see "Rust Backend (Embedded App)").

## Plugin Structure

A WebView plugin follows this directory structure:

```
~/.mofa-studio/plugins/
  my-plugin/
    manifest.json      # Plugin metadata (required)
    python/
      app.py           # Python backend server (required for webview)
      requirements.txt # Python dependencies (optional)
    static/
      index.html       # Main UI page (required)
      style.css        # Additional styles (optional)
      app.js           # Additional scripts (optional)
```

## Manifest File

The `manifest.json` file defines your plugin's metadata:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique identifier (lowercase, hyphens allowed) |
| `name` | string | Yes | Display name shown in UI |
| `version` | string | Yes | Semantic version (e.g., "1.0.0") |
| `description` | string | Yes | Short description |
| `author` | string | No | Author name or organization |
| `type` | string | Yes | Plugin type: "webview" or "native" |
| `icon` | string | No | Icon name (for future use) |
| `python_entry` | string | Yes* | Path to Python entry point (* for webview) |
| `static_dir` | string | No | Path to static files directory (default: "static") |
| `show_in_sidebar` | boolean | No | Whether to show in sidebar (default: true) |

Example:

```json
{
  "id": "note-taker",
  "name": "Note Taker",
  "version": "1.0.0",
  "description": "A simple note-taking application",
  "author": "MoFA Team",
  "type": "webview",
  "icon": "notes",
  "python_entry": "python/app.py",
  "static_dir": "static",
  "show_in_sidebar": true
}
```

## Python Backend

The Python backend is an HTTP server that:
- Receives a port number as command line argument
- Serves static files from the `static_dir`
- Provides API endpoints for your plugin's functionality

### Minimal Example

```python
#!/usr/bin/env python3
import json
import sys
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path

class PluginHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        static_dir = Path(__file__).parent.parent / "static"
        super().__init__(*args, directory=str(static_dir), **kwargs)

    def do_GET(self):
        if self.path == "/api/info":
            self._send_json({"name": "My Plugin", "status": "running"})
        else:
            super().do_GET()

    def _send_json(self, data, status=200):
        body = json.dumps(data).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", len(body))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(body)

def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
    server = HTTPServer(("127.0.0.1", port), PluginHandler)
    print(f"Server running on http://127.0.0.1:{port}")
    server.serve_forever()

if __name__ == "__main__":
    main()
```

### Key Points

- The port is passed as the first command line argument
- Always bind to `127.0.0.1` for security
- Set `Access-Control-Allow-Origin: *` for API responses
- Inherit from `SimpleHTTPRequestHandler` to serve static files
- Override `do_GET`, `do_POST`, etc. for API endpoints

## Rust Backend (Embedded App)

If you want the same WebView + HTTP pattern but without Python, build a native app and run an in-process Rust HTTP server. This is compiled into the shell (not a dynamic plugin). The app then loads `http://127.0.0.1:{port}/` into a `WebViewContainer`.

Reference implementation:
- `apps/mofa-hello-world-rust/` (WebView UI + Rust HTTP server, no Python)

Typical structure:
- `static/index.html` for the UI
- Rust server thread that serves `/api/*` and static files
- Start/stop the server from the screen widget

Minimal sketch:

```rust
// In your screen module
let listener = TcpListener::bind("127.0.0.1:0")?;
let port = listener.local_addr()?.port();
// spawn thread to accept requests and serve /api + static
let url = format!("http://127.0.0.1:{}", port);
webview.load_url(&url)?;
```

### Using Flask (Alternative)

You can also use Flask for more complex backends:

```python
#!/usr/bin/env python3
import sys
from flask import Flask, jsonify, send_from_directory
from pathlib import Path

app = Flask(__name__, static_folder=str(Path(__file__).parent.parent / "static"))

@app.route("/")
def index():
    return send_from_directory(app.static_folder, "index.html")

@app.route("/api/info")
def info():
    return jsonify({"name": "My Plugin", "status": "running"})

if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
    app.run(host="127.0.0.1", port=port)
```

## Frontend HTML

The frontend is a standard HTML page with JavaScript. MoFA Studio's WebView provides:

- Full HTML5/CSS3/JavaScript support
- Access to your Python backend via HTTP
- Theme integration via `window.setTheme()` callback

### Minimal Example

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>My Plugin</title>
    <style>
        :root {
            --bg: #ffffff;
            --text: #1d1d1f;
        }
        .dark {
            --bg: #1c1c1e;
            --text: #f5f5f7;
        }
        body {
            font-family: system-ui, sans-serif;
            background: var(--bg);
            color: var(--text);
            margin: 0;
            padding: 20px;
        }
    </style>
</head>
<body>
    <h1>My Plugin</h1>
    <div id="content"></div>

    <script>
        // Theme support - called by MoFA Studio
        window.setTheme = function(darkMode) {
            if (darkMode >= 0.5) {
                document.body.classList.add('dark');
            } else {
                document.body.classList.remove('dark');
            }
        };

        // Load data from backend
        async function loadInfo() {
            const res = await fetch('/api/info');
            const data = await res.json();
            document.getElementById('content').textContent = JSON.stringify(data, null, 2);
        }

        loadInfo();
    </script>
</body>
</html>
```

## Theme Support

MoFA Studio calls `window.setTheme(darkMode)` when the theme changes:

- `darkMode` is a float from 0.0 (light) to 1.0 (dark)
- Called on page load and when user toggles theme
- Use CSS variables for easy theme switching

```javascript
window.setTheme = function(darkMode) {
    if (darkMode >= 0.5) {
        document.body.classList.add('dark');
    } else {
        document.body.classList.remove('dark');
    }
};
```

CSS variables pattern:

```css
:root {
    --bg-primary: #f5f5f7;
    --bg-secondary: #ffffff;
    --text-primary: #1d1d1f;
    --text-secondary: #6e6e73;
    --border-color: #d2d2d7;
    --accent-color: #007aff;
}

.dark {
    --bg-primary: #1c1c1e;
    --bg-secondary: #2c2c2e;
    --text-primary: #f5f5f7;
    --text-secondary: #98989d;
    --border-color: #48484a;
}
```

## API Design

### RESTful Patterns

Design your API using RESTful conventions:

```
GET    /api/items         # List all items
POST   /api/items         # Create new item
GET    /api/items/{id}    # Get single item
PUT    /api/items/{id}    # Update item
DELETE /api/items/{id}    # Delete item
```

### Response Format

Use consistent JSON response format:

```json
// Success
{
  "data": { ... },
  "message": "Item created successfully"
}

// Error
{
  "error": "Item not found",
  "code": 404
}
```

### CORS Headers

Always include CORS headers for API responses:

```python
def _send_json(self, data, status=200):
    body = json.dumps(data).encode("utf-8")
    self.send_response(status)
    self.send_header("Content-Type", "application/json; charset=utf-8")
    self.send_header("Access-Control-Allow-Origin", "*")
    self.send_header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
    self.send_header("Access-Control-Allow-Headers", "Content-Type")
    self.end_headers()
    self.wfile.write(body)
```

## Examples

### Hello World Plugin

See `~/.mofa-studio/plugins/hello-world/` for the simplest possible plugin.

### Hello World (Rust) App

See `apps/mofa-hello-world-rust/` for a Rust-powered WebView app that mirrors the Python plugin pattern.

### Note Taker Plugin

See `~/.mofa-studio/plugins/note-taker/` for a full-featured example with:
- CRUD operations
- File-based persistence
- Search functionality
- Dark mode support
- Responsive UI

## Debugging

### Check Plugin Loading

1. Look at MoFA Studio's console output for plugin loading messages:
   ```
   Loaded plugin: Note Taker v1.0.0
   ```

2. If your plugin doesn't appear, check:
   - `manifest.json` syntax (use a JSON validator)
   - Plugin directory is in `~/.mofa-studio/plugins/`
   - `show_in_sidebar` is `true`

### Debug Python Backend

1. Run the server manually to see errors:
   ```bash
   python3 ~/.mofa-studio/plugins/my-plugin/python/app.py 8080
   ```

2. Test API endpoints with curl:
   ```bash
   curl http://127.0.0.1:8080/api/info
   ```

### Debug Rust Backend (Embedded App)

- Run the shell and open your app page.
- Check console logs for server start/stop and request errors.

### Debug Frontend

1. Open browser developer tools (F12)
2. Check Console for JavaScript errors
3. Check Network tab for failed requests

### Common Issues

| Issue | Solution |
|-------|----------|
| Plugin not in sidebar | Check `show_in_sidebar: true` in manifest |
| Server won't start | Check Python path, port availability |
| API 404 errors | Check endpoint paths match frontend calls |
| Theme not updating | Ensure `window.setTheme` is defined |
| CORS errors | Add proper CORS headers to all API responses |

## Plugin Lifecycle

This section describes dynamic WebView plugins (Python backend). Native apps follow the standard app lifecycle.

1. **Discovery**: MoFA Studio scans `~/.mofa-studio/plugins/` at startup
2. **Loading**: Parses `manifest.json` for each plugin directory
3. **Display**: Shows plugins in sidebar (if `show_in_sidebar: true`)
4. **Activation**: When user clicks plugin, server starts on available port
5. **Running**: WebView loads `http://127.0.0.1:{port}/`
6. **Deactivation**: When user navigates away, server may stop

## Best Practices

1. **Keep it simple**: Start with minimal functionality, add features gradually
2. **Use relative paths**: Reference static files with relative URLs
3. **Handle errors gracefully**: Show user-friendly error messages
4. **Support themes**: Always implement `window.setTheme`
5. **Persist data locally**: Store data in user's Documents folder
6. **Log appropriately**: Print startup messages and errors to console
7. **Test standalone**: Ensure your server works before testing in MoFA Studio

## Next Steps

- Review the example plugins in `~/.mofa-studio/plugins/`
- Read the Makepad documentation for native plugin development
- Join the MoFA community for support and sharing plugins
