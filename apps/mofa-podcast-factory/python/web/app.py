"""
Podcast Factory Backend

Generate multi-episode podcast series from books using AI.
"""

import os
import json
import uuid
import subprocess
import threading
import base64
import re
from pathlib import Path
from typing import Optional, Dict, Any, List
from http.server import HTTPServer, SimpleHTTPRequestHandler
from urllib.parse import parse_qs, urlparse
import tempfile

# Optional imports for file parsing
try:
    import PyPDF2
    HAS_PYPDF2 = True
except ImportError:
    HAS_PYPDF2 = False

try:
    import ebooklib
    from ebooklib import epub
    from bs4 import BeautifulSoup
    HAS_EPUB = True
except ImportError:
    HAS_EPUB = False

# Global state
projects: Dict[str, Dict[str, Any]] = {}
OUTPUT_DIR = Path.home() / "Documents" / "MoFaPodcastFactory"
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

# macOS voices
VOICES = {
    "female_en": "Samantha",
    "male_en": "Daniel",
    "female_zh": "Ting-Ting",
    "male_zh": "Mei-Jia",
    "narrator": "Alex",
}


def parse_txt_file(file_path: Path) -> str:
    """Parse plain text file."""
    encodings = ['utf-8', 'gbk', 'gb2312', 'latin-1']
    for enc in encodings:
        try:
            return file_path.read_text(encoding=enc)
        except UnicodeDecodeError:
            continue
    return ""


def parse_pdf_file(file_path: Path) -> str:
    """Parse PDF file content."""
    if not HAS_PYPDF2:
        raise ImportError("PyPDF2 not installed. Run: pip install PyPDF2")

    text_parts = []
    with open(file_path, 'rb') as f:
        reader = PyPDF2.PdfReader(f)
        for page in reader.pages:
            text = page.extract_text()
            if text:
                text_parts.append(text)
    return '\n'.join(text_parts)


def parse_epub_file(file_path: Path) -> str:
    """Parse EPUB file content."""
    if not HAS_EPUB:
        raise ImportError("ebooklib not installed. Run: pip install ebooklib beautifulsoup4")

    book = epub.read_epub(str(file_path))
    text_parts = []

    for item in book.get_items():
        if item.get_type() == ebooklib.ITEM_DOCUMENT:
            soup = BeautifulSoup(item.get_content(), 'html.parser')
            text = soup.get_text(separator='\n')
            # Clean up extra whitespace
            text = re.sub(r'\n\s*\n', '\n\n', text)
            text_parts.append(text.strip())

    return '\n\n'.join(text_parts)


def parse_book_file(file_path: Path) -> str:
    """Parse book file based on extension."""
    ext = file_path.suffix.lower()

    if ext == '.txt':
        return parse_txt_file(file_path)
    elif ext == '.pdf':
        return parse_pdf_file(file_path)
    elif ext == '.epub':
        return parse_epub_file(file_path)
    else:
        raise ValueError(f"Unsupported file format: {ext}")


def call_openai(messages: List[Dict], api_key: str, max_tokens: int = 2000) -> Optional[str]:
    """Call OpenAI API."""
    try:
        import openai
        client = openai.OpenAI(api_key=api_key)
        response = client.chat.completions.create(
            model="gpt-4o-mini",
            messages=messages,
            max_tokens=max_tokens
        )
        return response.choices[0].message.content
    except Exception as e:
        print(f"OpenAI error: {e}")
        return None


def generate_outline(book_content: str, num_episodes: int, style: str, api_key: str) -> Optional[Dict]:
    """Generate episode outline from book content."""

    system_prompt = f"""You are a podcast series planner. Your task is to convert book content into a {num_episodes}-episode podcast series.

Style: {style}

For each episode, provide:
1. Episode number and title
2. Main topic/theme
3. Key points to cover (3-5 points)
4. Suggested duration in minutes

Output as JSON array:
[
  {{
    "episode": 1,
    "title": "Episode Title",
    "theme": "Main theme",
    "key_points": ["point1", "point2", "point3"],
    "duration_minutes": 15
  }},
  ...
]"""

    messages = [
        {"role": "system", "content": system_prompt},
        {"role": "user", "content": f"Create a {num_episodes}-episode podcast outline from this content:\n\n{book_content[:15000]}"}
    ]

    result = call_openai(messages, api_key, max_tokens=3000)
    if not result:
        return None

    # Parse JSON from response
    try:
        # Find JSON array in response
        start = result.find('[')
        end = result.rfind(']') + 1
        if start >= 0 and end > start:
            return {"episodes": json.loads(result[start:end])}
    except json.JSONDecodeError as e:
        print(f"JSON parse error: {e}")

    return None


def generate_episode_script(
    episode_info: Dict,
    book_content: str,
    personas: List[Dict],
    style: str,
    api_key: str
) -> Optional[Dict]:
    """Generate script for a single episode."""

    persona_desc = "\n".join([
        f"- {p['name']}: {p['personality']} (Voice: {p['voice']})"
        for p in personas
    ])

    system_prompt = f"""You are a podcast script writer. Write an engaging dialogue script for a podcast episode.

Episode: {episode_info['title']}
Theme: {episode_info['theme']}
Key Points: {', '.join(episode_info['key_points'])}
Target Duration: {episode_info.get('duration_minutes', 15)} minutes

Hosts/Characters:
{persona_desc}

Style: {style}

Rules:
1. Format each line as: "CharacterName: dialogue text"
2. Make conversations natural and engaging
3. Include reactions, questions, and smooth transitions
4. Cover all key points naturally
5. Start with intro, end with conclusion/teaser for next episode

Output the script directly, no markdown code blocks."""

    messages = [
        {"role": "system", "content": system_prompt},
        {"role": "user", "content": f"Write the script for Episode {episode_info['episode']}: {episode_info['title']}\n\nRelevant content:\n{book_content[:8000]}"}
    ]

    script_text = call_openai(messages, api_key, max_tokens=4000)
    if not script_text:
        return None

    # Parse into segments - be more lenient with role matching
    # Build mapping from various possible names to voice
    voice_map = {}
    for p in personas:
        voice_map[p['name'].lower()] = p['voice']
        voice_map[p['voice'].lower()] = p['voice']
        # Also add partial matches
        for word in p['name'].lower().split():
            voice_map[word] = p['voice']

    segments = []
    default_voices = [p['voice'] for p in personas]
    voice_index = 0

    for line in script_text.strip().split('\n'):
        line = line.strip()
        if ':' in line and not line.startswith('http'):
            parts = line.split(':', 1)
            if len(parts) == 2:
                role = parts[0].strip()
                text = parts[1].strip()
                if text and len(role) < 50:  # Reasonable role name length
                    # Find matching voice
                    role_lower = role.lower()
                    voice = voice_map.get(role_lower)
                    if not voice:
                        # Try partial match
                        for key, v in voice_map.items():
                            if key in role_lower or role_lower in key:
                                voice = v
                                break
                    if not voice:
                        # Fallback: alternate between available voices
                        voice = default_voices[voice_index % len(default_voices)]
                        voice_index += 1
                        # Remember this role for consistency
                        voice_map[role_lower] = voice

                    segments.append({"role": role, "text": text, "voice": voice})

    return {
        "script": script_text,
        "segments": segments
    }


def speak_macos(text: str, voice: str, output_path: Path, rate: int = 180) -> bool:
    """Generate audio using macOS say command."""
    try:
        # First generate AIFF
        aiff_path = output_path.with_suffix('.aiff')
        cmd = ['say', '-v', voice, '-r', str(rate), '-o', str(aiff_path), text]
        result = subprocess.run(cmd, capture_output=True, timeout=300)

        if result.returncode != 0:
            print(f"say error: {result.stderr.decode()}")
            return False

        # Convert to WAV using afconvert
        wav_path = output_path.with_suffix('.wav')
        convert_cmd = [
            'afconvert', '-f', 'WAVE', '-d', 'LEI16',
            str(aiff_path), str(wav_path)
        ]
        subprocess.run(convert_cmd, capture_output=True, timeout=60)

        # Remove temp AIFF
        aiff_path.unlink(missing_ok=True)

        return wav_path.exists()
    except Exception as e:
        print(f"TTS error: {e}")
        return False


def concatenate_audio(input_files: List[Path], output_path: Path) -> bool:
    """Concatenate WAV files."""
    if not input_files:
        return False

    if len(input_files) == 1:
        import shutil
        shutil.copy(input_files[0], output_path)
        return True

    # Try sox first
    try:
        cmd = ['sox'] + [str(f) for f in input_files] + [str(output_path)]
        result = subprocess.run(cmd, capture_output=True, timeout=300)
        if result.returncode == 0:
            return True
    except:
        pass

    # Fallback: manual concatenation with hound-like approach
    try:
        import wave

        with wave.open(str(output_path), 'wb') as output:
            for i, file_path in enumerate(input_files):
                with wave.open(str(file_path), 'rb') as inp:
                    if i == 0:
                        output.setparams(inp.getparams())
                    output.writeframes(inp.readframes(inp.getnframes()))
        return True
    except Exception as e:
        print(f"Concat error: {e}")
        return False


def generate_episode_audio(
    episode_num: int,
    segments: List[Dict],
    personas: List[Dict],
    project_dir: Path,
    rate: int = 180,
    progress_callback=None
) -> Optional[Path]:
    """Generate audio for an episode."""

    episode_dir = project_dir / f"episode_{episode_num:02d}"
    episode_dir.mkdir(exist_ok=True)

    # Build voice mapping as fallback
    voice_map = {p['name']: p['voice'] for p in personas}
    default_voice = personas[0]['voice'] if personas else 'Samantha'

    # Generate audio for each segment
    segment_files = []
    total = len(segments)

    for i, seg in enumerate(segments):
        if progress_callback:
            progress_callback(i + 1, total, f"Generating segment {i+1}/{total}")

        role = seg['role']
        text = seg['text']
        # Use voice from segment if available, otherwise look up
        voice = seg.get('voice') or voice_map.get(role, default_voice)

        seg_path = episode_dir / f"seg_{i:04d}.wav"
        if speak_macos(text, voice, seg_path, rate):
            segment_files.append(seg_path)

    if not segment_files:
        return None

    # Concatenate
    output_path = episode_dir / f"episode_{episode_num:02d}.wav"
    if concatenate_audio(segment_files, output_path):
        # Cleanup segment files
        for f in segment_files:
            f.unlink(missing_ok=True)
        return output_path

    return None


class PodcastFactoryHandler(SimpleHTTPRequestHandler):
    """HTTP handler for Podcast Factory."""

    def __init__(self, *args, **kwargs):
        self.directory = str(Path(__file__).parent / "static")
        super().__init__(*args, directory=self.directory, **kwargs)

    def do_GET(self):
        parsed = urlparse(self.path)

        if parsed.path == "/api/projects":
            self._json_response(200, {"projects": list(projects.keys())})
        elif parsed.path == "/api/project":
            params = parse_qs(parsed.query)
            pid = params.get("id", [None])[0]
            if pid and pid in projects:
                self._json_response(200, projects[pid])
            else:
                self._json_response(404, {"error": "Project not found"})
        elif parsed.path == "/api/voices":
            self._json_response(200, {"voices": VOICES})
        elif parsed.path == "/api/formats":
            # Report supported formats
            formats = [".txt"]
            if HAS_PYPDF2:
                formats.append(".pdf")
            if HAS_EPUB:
                formats.append(".epub")
            self._json_response(200, {"formats": formats, "has_pdf": HAS_PYPDF2, "has_epub": HAS_EPUB})
        else:
            super().do_GET()

    def do_POST(self):
        parsed = urlparse(self.path)

        if parsed.path == "/api/create-project":
            self._handle_create_project()
        elif parsed.path == "/api/upload-book":
            self._handle_upload_book()
        elif parsed.path == "/api/generate-outline":
            self._handle_generate_outline()
        elif parsed.path == "/api/generate-episode":
            self._handle_generate_episode()
        elif parsed.path == "/api/generate-all":
            self._handle_generate_all()
        elif parsed.path == "/api/preview-voice":
            self._handle_preview_voice()
        elif parsed.path == "/api/open-folder":
            self._handle_open_folder()
        else:
            self._json_response(404, {"error": "Not found"})

    def _read_json(self) -> Optional[Dict]:
        try:
            length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(length)
            return json.loads(body.decode("utf-8"))
        except:
            return None

    def _json_response(self, status: int, data: dict):
        body = json.dumps(data, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(body)

    def _handle_upload_book(self):
        """Handle book file upload (base64 encoded)."""
        data = self._read_json()
        if not data:
            self._json_response(400, {"error": "Invalid JSON"})
            return

        filename = data.get("filename", "book.txt")
        file_data = data.get("data")  # base64 encoded

        if not file_data:
            self._json_response(400, {"error": "No file data"})
            return

        # Check extension
        ext = Path(filename).suffix.lower()
        if ext not in ['.txt', '.pdf', '.epub']:
            self._json_response(400, {"error": f"Unsupported format: {ext}. Use .txt, .pdf, or .epub"})
            return

        if ext == '.pdf' and not HAS_PYPDF2:
            self._json_response(400, {"error": "PDF support not available. Install: pip install PyPDF2"})
            return

        if ext == '.epub' and not HAS_EPUB:
            self._json_response(400, {"error": "EPUB support not available. Install: pip install ebooklib beautifulsoup4"})
            return

        try:
            # Decode base64
            file_bytes = base64.b64decode(file_data)

            # Save to temp file
            with tempfile.NamedTemporaryFile(suffix=ext, delete=False) as tmp:
                tmp.write(file_bytes)
                tmp_path = Path(tmp.name)

            # Parse content
            content = parse_book_file(tmp_path)

            # Cleanup
            tmp_path.unlink(missing_ok=True)

            # Return stats
            char_count = len(content)
            word_count = len(content.split())

            self._json_response(200, {
                "content": content,
                "filename": filename,
                "char_count": char_count,
                "word_count": word_count,
                "preview": content[:2000] + ("..." if len(content) > 2000 else "")
            })

        except ImportError as e:
            self._json_response(400, {"error": str(e)})
        except Exception as e:
            self._json_response(500, {"error": f"Parse error: {str(e)}"})

    def _handle_create_project(self):
        data = self._read_json()
        if not data:
            self._json_response(400, {"error": "Invalid JSON"})
            return

        project_id = str(uuid.uuid4())[:8]
        project_dir = OUTPUT_DIR / project_id
        project_dir.mkdir(exist_ok=True)

        projects[project_id] = {
            "id": project_id,
            "name": data.get("name", "Untitled"),
            "book_content": data.get("book_content", ""),
            "book_filename": data.get("book_filename", ""),
            "num_episodes": data.get("num_episodes", 10),
            "style": data.get("style", "conversational"),
            "personas": data.get("personas", [
                {"name": "Host A", "personality": "Curious and engaging host", "voice": "Samantha"},
                {"name": "Host B", "personality": "Knowledgeable expert", "voice": "Daniel"}
            ]),
            "outline": None,
            "episodes": {},
            "status": "created",
            "dir": str(project_dir)
        }

        self._json_response(200, {"project_id": project_id})

    def _handle_generate_outline(self):
        data = self._read_json()
        if not data:
            self._json_response(400, {"error": "Invalid JSON"})
            return

        project_id = data.get("project_id")
        api_key = data.get("api_key") or os.environ.get("OPENAI_API_KEY")

        if not project_id or project_id not in projects:
            self._json_response(404, {"error": "Project not found"})
            return

        if not api_key:
            self._json_response(400, {"error": "API key required"})
            return

        project = projects[project_id]
        project["status"] = "generating_outline"

        outline = generate_outline(
            project["book_content"],
            project["num_episodes"],
            project["style"],
            api_key
        )

        if outline:
            project["outline"] = outline
            project["status"] = "outline_ready"
            self._json_response(200, {"outline": outline})
        else:
            project["status"] = "error"
            self._json_response(500, {"error": "Failed to generate outline"})

    def _handle_generate_episode(self):
        data = self._read_json()
        if not data:
            self._json_response(400, {"error": "Invalid JSON"})
            return

        project_id = data.get("project_id")
        episode_num = data.get("episode_num", 1)
        api_key = data.get("api_key") or os.environ.get("OPENAI_API_KEY")
        generate_audio = data.get("generate_audio", True)

        if not project_id or project_id not in projects:
            self._json_response(404, {"error": "Project not found"})
            return

        project = projects[project_id]

        if not project.get("outline"):
            self._json_response(400, {"error": "Generate outline first"})
            return

        # Find episode info
        episode_info = None
        for ep in project["outline"]["episodes"]:
            if ep["episode"] == episode_num:
                episode_info = ep
                break

        if not episode_info:
            self._json_response(404, {"error": "Episode not found in outline"})
            return

        # Generate script
        script_result = generate_episode_script(
            episode_info,
            project["book_content"],
            project["personas"],
            project["style"],
            api_key
        )

        if not script_result:
            self._json_response(500, {"error": "Failed to generate script"})
            return

        episode_data = {
            "episode": episode_num,
            "title": episode_info["title"],
            "script": script_result["script"],
            "segments": script_result["segments"],
            "audio_path": None
        }

        # Generate audio if requested
        if generate_audio and script_result["segments"]:
            project_dir = Path(project["dir"])
            audio_path = generate_episode_audio(
                episode_num,
                script_result["segments"],
                project["personas"],
                project_dir,
                rate=data.get("rate", 180)
            )
            if audio_path:
                episode_data["audio_path"] = str(audio_path)

        project["episodes"][str(episode_num)] = episode_data

        # Save script to file
        script_path = Path(project["dir"]) / f"episode_{episode_num:02d}" / "script.md"
        script_path.parent.mkdir(exist_ok=True)
        script_path.write_text(script_result["script"])

        self._json_response(200, {"episode": episode_data})

    def _handle_generate_all(self):
        """Generate all episodes in background."""
        data = self._read_json()
        if not data:
            self._json_response(400, {"error": "Invalid JSON"})
            return

        project_id = data.get("project_id")
        api_key = data.get("api_key") or os.environ.get("OPENAI_API_KEY")

        if not project_id or project_id not in projects:
            self._json_response(404, {"error": "Project not found"})
            return

        project = projects[project_id]

        if not project.get("outline"):
            self._json_response(400, {"error": "Generate outline first"})
            return

        def generate_all_episodes():
            project["status"] = "generating_episodes"
            total = len(project["outline"]["episodes"])

            for i, ep_info in enumerate(project["outline"]["episodes"]):
                project["current_episode"] = ep_info["episode"]
                project["progress"] = f"Episode {i+1}/{total}"

                script_result = generate_episode_script(
                    ep_info,
                    project["book_content"],
                    project["personas"],
                    project["style"],
                    api_key
                )

                if script_result:
                    episode_data = {
                        "episode": ep_info["episode"],
                        "title": ep_info["title"],
                        "script": script_result["script"],
                        "segments": script_result["segments"],
                        "audio_path": None
                    }

                    # Generate audio
                    project_dir = Path(project["dir"])
                    audio_path = generate_episode_audio(
                        ep_info["episode"],
                        script_result["segments"],
                        project["personas"],
                        project_dir
                    )
                    if audio_path:
                        episode_data["audio_path"] = str(audio_path)

                    project["episodes"][str(ep_info["episode"])] = episode_data

                    # Save script
                    script_path = project_dir / f"episode_{ep_info['episode']:02d}" / "script.md"
                    script_path.parent.mkdir(exist_ok=True)
                    script_path.write_text(script_result["script"])

            project["status"] = "completed"
            project["progress"] = "All episodes generated"

        thread = threading.Thread(target=generate_all_episodes)
        thread.start()

        self._json_response(200, {"status": "started", "total_episodes": len(project["outline"]["episodes"])})

    def _handle_preview_voice(self):
        """Preview a voice with sample text."""
        data = self._read_json()
        if not data:
            self._json_response(400, {"error": "Invalid JSON"})
            return

        voice = data.get("voice", "Samantha")
        text = data.get("text", "Hello, this is a voice preview.")

        try:
            subprocess.run(['say', '-v', voice, text], timeout=30)
            self._json_response(200, {"status": "ok"})
        except Exception as e:
            self._json_response(500, {"error": str(e)})

    def _handle_open_folder(self):
        """Open folder in Finder."""
        data = self._read_json()
        if not data:
            self._json_response(400, {"error": "Invalid JSON"})
            return

        path = data.get("path")
        if not path or not Path(path).exists():
            self._json_response(400, {"error": "Invalid path"})
            return

        try:
            subprocess.run(['open', path], timeout=10)
            self._json_response(200, {"status": "ok"})
        except Exception as e:
            self._json_response(500, {"error": str(e)})

    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()

    def log_message(self, format, *args):
        print(f"[PodcastFactory] {args[0]}")


def run_server(port: int = 8082):
    """Run the HTTP server."""
    server = HTTPServer(("127.0.0.1", port), PodcastFactoryHandler)
    print(f"Podcast Factory server running on http://127.0.0.1:{port}")
    server.serve_forever()


if __name__ == "__main__":
    import sys
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8082
    run_server(port)
