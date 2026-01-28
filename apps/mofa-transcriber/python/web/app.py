"""
AI Transcriber Backend

Transcribe audio/video files using Whisper and summarize with LLM.
"""

import os
import json
import uuid
import asyncio
import tempfile
import subprocess
from pathlib import Path
from typing import Optional, Dict, Any
from http.server import HTTPServer, SimpleHTTPRequestHandler
from urllib.parse import parse_qs, urlparse
import threading

# Global state for transcription jobs
jobs: Dict[str, Dict[str, Any]] = {}

# Configuration
UPLOAD_DIR = Path(tempfile.gettempdir()) / "mofa-transcriber"
UPLOAD_DIR.mkdir(exist_ok=True)

# Supported formats
AUDIO_FORMATS = {'.mp3', '.wav', '.m4a', '.flac', '.ogg', '.wma', '.aac'}
VIDEO_FORMATS = {'.mp4', '.mkv', '.avi', '.mov', '.webm', '.flv', '.wmv'}


def extract_audio_from_video(video_path: str, output_path: str) -> bool:
    """Extract audio from video using ffmpeg."""
    try:
        cmd = [
            'ffmpeg', '-i', video_path,
            '-vn',  # No video
            '-acodec', 'pcm_s16le',  # PCM format
            '-ar', '16000',  # 16kHz sample rate (Whisper optimal)
            '-ac', '1',  # Mono
            '-y',  # Overwrite
            output_path
        ]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
        return result.returncode == 0
    except Exception as e:
        print(f"FFmpeg error: {e}")
        return False


def get_whisper_model_dir() -> Optional[str]:
    """Resolve Whisper model directory from env or bundled resources."""
    env_dir = os.environ.get("WHISPER_MODEL_DIR")
    if env_dir:
        return env_dir
    try:
        resources_dir = Path(__file__).resolve().parents[3]
        candidate = resources_dir / "models" / "whisper"
        if candidate.exists():
            return str(candidate)
    except Exception:
        return None
    return None


def transcribe_audio(audio_path: str, model_size: str = "tiny") -> Optional[Dict]:
    """Transcribe audio using faster-whisper."""
    try:
        from faster_whisper import WhisperModel

        # Use CPU by default, GPU if available
        model_dir = get_whisper_model_dir()
        if model_dir:
            model = WhisperModel(model_size, device="cpu", compute_type="int8", download_root=model_dir)
        else:
            model = WhisperModel(model_size, device="cpu", compute_type="int8")

        segments, info = model.transcribe(audio_path, beam_size=5)

        # Collect segments
        result_segments = []
        full_text = []

        for segment in segments:
            result_segments.append({
                "start": segment.start,
                "end": segment.end,
                "text": segment.text
            })
            full_text.append(segment.text)

        return {
            "language": info.language,
            "language_probability": info.language_probability,
            "duration": info.duration,
            "text": " ".join(full_text),
            "segments": result_segments
        }
    except ImportError:
        print("faster-whisper not installed, using mock transcription")
        return {
            "language": "en",
            "language_probability": 0.99,
            "duration": 60.0,
            "text": "[faster-whisper not installed. Please run: pip install faster-whisper]",
            "segments": []
        }
    except Exception as e:
        print(f"Transcription error: {e}")
        return None


def summarize_text(text: str, api_key: Optional[str] = None) -> Optional[str]:
    """Summarize text using OpenAI API."""
    if not api_key:
        api_key = os.environ.get("OPENAI_API_KEY")

    if not api_key:
        return "[No API key provided. Set OPENAI_API_KEY or provide in request.]"

    try:
        import openai
        client = openai.OpenAI(api_key=api_key)

        response = client.chat.completions.create(
            model="gpt-4o-mini",
            messages=[
                {
                    "role": "system",
                    "content": "You are a helpful assistant that summarizes transcripts. Provide a clear, structured summary with key points."
                },
                {
                    "role": "user",
                    "content": f"Please summarize the following transcript:\n\n{text[:8000]}"
                }
            ],
            max_tokens=1000
        )
        return response.choices[0].message.content
    except ImportError:
        return "[openai package not installed. Please run: pip install openai]"
    except Exception as e:
        return f"[Summary error: {e}]"


def generate_podcast_script(text: str, api_key: Optional[str] = None, num_hosts: int = 2) -> Optional[Dict]:
    """Generate a podcast script from text using OpenAI API."""
    if not api_key:
        api_key = os.environ.get("OPENAI_API_KEY")

    if not api_key:
        return {"error": "No API key provided. Set OPENAI_API_KEY or provide in request."}

    try:
        import openai
        client = openai.OpenAI(api_key=api_key)

        host_names = ["Host A", "Host B"] if num_hosts == 2 else ["Host A", "Host B", "Host C"]

        response = client.chat.completions.create(
            model="gpt-4o-mini",
            messages=[
                {
                    "role": "system",
                    "content": f"""You are a podcast script writer. Convert the given content into an engaging podcast conversation between {num_hosts} hosts.

Rules:
1. Use exactly these host names: {', '.join(host_names)}
2. Format each line as: "HostName: dialogue text"
3. Make the conversation natural, engaging, and informative
4. Include reactions, questions, and smooth transitions
5. Keep the total script around 500-800 words
6. Start with a brief introduction and end with a conclusion

Example format:
Host A: Welcome to today's episode! We have some fascinating content to discuss.
Host B: Absolutely! I'm really excited about this topic."""
                },
                {
                    "role": "user",
                    "content": f"Please convert this content into a podcast script:\n\n{text[:6000]}"
                }
            ],
            max_tokens=2000
        )

        script_text = response.choices[0].message.content

        # Parse the script into segments
        segments = []
        for line in script_text.strip().split('\n'):
            line = line.strip()
            if ':' in line and any(line.startswith(h) for h in host_names):
                parts = line.split(':', 1)
                if len(parts) == 2:
                    segments.append({
                        "role": parts[0].strip(),
                        "text": parts[1].strip()
                    })

        return {
            "script": script_text,
            "segments": segments,
            "hosts": host_names
        }
    except ImportError:
        return {"error": "openai package not installed. Please run: pip install openai"}
    except Exception as e:
        return {"error": f"Script generation error: {e}"}


# macOS voice mapping
MACOS_VOICES = {
    "Host A": "Samantha",  # English female
    "Host B": "Daniel",    # English male (British)
    "Host C": "Alex",      # English male
    "Ting-Ting": "Ting-Ting",  # Chinese female
    "Mei-Jia": "Mei-Jia",      # Chinese female (Taiwan)
}


def speak_text_macos(text: str, voice: str = "Samantha", rate: int = 180) -> bool:
    """Speak text using macOS say command."""
    try:
        cmd = ['say', '-v', voice, '-r', str(rate), text]
        subprocess.run(cmd, check=True, timeout=120)
        return True
    except Exception as e:
        print(f"TTS error: {e}")
        return False


def speak_script_async(segments: list, voice_mapping: Dict[str, str], rate: int = 180):
    """Speak podcast script segments sequentially."""
    for segment in segments:
        role = segment.get("role", "Host A")
        text = segment.get("text", "")
        voice = voice_mapping.get(role, "Samantha")

        if text:
            speak_text_macos(text, voice, rate)


# TTS state
tts_state = {"speaking": False, "stop_requested": False}


def process_job(job_id: str, file_path: str, model_size: str, api_key: Optional[str]):
    """Process a transcription job in background."""
    job = jobs[job_id]

    try:
        job["status"] = "processing"
        job["progress"] = 10

        # Check if video needs audio extraction
        ext = Path(file_path).suffix.lower()
        audio_path = file_path

        if ext in VIDEO_FORMATS:
            job["stage"] = "Extracting audio from video..."
            job["progress"] = 20
            audio_path = str(UPLOAD_DIR / f"{job_id}.wav")

            if not extract_audio_from_video(file_path, audio_path):
                job["status"] = "error"
                job["error"] = "Failed to extract audio from video"
                return

        # Transcribe
        job["stage"] = "Transcribing audio..."
        job["progress"] = 40

        result = transcribe_audio(audio_path, model_size)

        if not result:
            job["status"] = "error"
            job["error"] = "Transcription failed"
            return

        job["transcription"] = result
        job["progress"] = 70

        # Summarize
        job["stage"] = "Generating summary..."
        job["progress"] = 80

        summary = summarize_text(result["text"], api_key)
        job["summary"] = summary
        job["progress"] = 100
        job["status"] = "completed"
        job["stage"] = "Done"

        # Cleanup temp files
        try:
            if audio_path != file_path:
                os.remove(audio_path)
            os.remove(file_path)
        except:
            pass

    except Exception as e:
        job["status"] = "error"
        job["error"] = str(e)


class TranscriberHandler(SimpleHTTPRequestHandler):
    """HTTP request handler for transcriber API."""

    def __init__(self, *args, **kwargs):
        self.directory = str(Path(__file__).parent / "static")
        super().__init__(*args, directory=self.directory, **kwargs)

    def do_GET(self):
        """Handle GET requests."""
        parsed = urlparse(self.path)

        if parsed.path == "/api/status":
            # Get job status
            params = parse_qs(parsed.query)
            job_id = params.get("id", [None])[0]

            if job_id and job_id in jobs:
                self._json_response(200, jobs[job_id])
            else:
                self._json_response(404, {"error": "Job not found"})

        elif parsed.path == "/api/jobs":
            # List all jobs
            self._json_response(200, {"jobs": list(jobs.keys())})

        else:
            # Serve static files
            super().do_GET()

    def do_POST(self):
        """Handle POST requests."""
        parsed = urlparse(self.path)

        if parsed.path == "/api/transcribe":
            self._handle_transcribe()
        elif parsed.path == "/api/generate-podcast":
            self._handle_generate_podcast()
        elif parsed.path == "/api/speak":
            self._handle_speak()
        elif parsed.path == "/api/stop-speak":
            self._handle_stop_speak()
        else:
            self._json_response(404, {"error": "Not found"})

    def _handle_generate_podcast(self):
        """Handle podcast script generation."""
        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length)

        try:
            data = json.loads(body.decode("utf-8"))
            text = data.get("text", "")
            api_key = data.get("api_key") or os.environ.get("OPENAI_API_KEY")
            num_hosts = data.get("num_hosts", 2)

            if not text:
                self._json_response(400, {"error": "No text provided"})
                return

            result = generate_podcast_script(text, api_key, num_hosts)
            self._json_response(200, result)

        except json.JSONDecodeError:
            self._json_response(400, {"error": "Invalid JSON"})
        except Exception as e:
            self._json_response(500, {"error": str(e)})

    def _handle_speak(self):
        """Handle TTS request."""
        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length)

        try:
            data = json.loads(body.decode("utf-8"))
            segments = data.get("segments", [])
            voice_mapping = data.get("voice_mapping", {
                "Host A": "Samantha",
                "Host B": "Daniel",
                "Host C": "Alex"
            })
            rate = data.get("rate", 180)

            if not segments:
                # Single text mode
                text = data.get("text", "")
                voice = data.get("voice", "Samantha")
                if text:
                    tts_state["speaking"] = True
                    tts_state["stop_requested"] = False

                    # Run in background thread
                    def speak_single():
                        speak_text_macos(text, voice, rate)
                        tts_state["speaking"] = False

                    thread = threading.Thread(target=speak_single)
                    thread.start()
                    self._json_response(200, {"status": "speaking"})
                else:
                    self._json_response(400, {"error": "No text or segments provided"})
                return

            # Segment mode - speak podcast script
            tts_state["speaking"] = True
            tts_state["stop_requested"] = False

            def speak_all():
                for segment in segments:
                    if tts_state["stop_requested"]:
                        break
                    role = segment.get("role", "Host A")
                    text = segment.get("text", "")
                    voice = voice_mapping.get(role, "Samantha")
                    if text:
                        speak_text_macos(text, voice, rate)
                tts_state["speaking"] = False

            thread = threading.Thread(target=speak_all)
            thread.start()

            self._json_response(200, {"status": "speaking", "segment_count": len(segments)})

        except json.JSONDecodeError:
            self._json_response(400, {"error": "Invalid JSON"})
        except Exception as e:
            self._json_response(500, {"error": str(e)})

    def _handle_stop_speak(self):
        """Stop current TTS."""
        tts_state["stop_requested"] = True
        # Kill any running say process
        try:
            subprocess.run(['pkill', '-9', 'say'], capture_output=True)
        except:
            pass
        tts_state["speaking"] = False
        self._json_response(200, {"status": "stopped"})

    def _handle_transcribe(self):
        """Handle file upload and start transcription."""
        content_type = self.headers.get("Content-Type", "")

        if "multipart/form-data" in content_type:
            # Parse boundary from content type
            boundary = None
            for part in content_type.split(";"):
                part = part.strip()
                if part.startswith("boundary="):
                    boundary = part[9:].strip('"')
                    break

            if not boundary:
                self._json_response(400, {"error": "No boundary in multipart data"})
                return

            # Read all content
            content_length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_length)

            # Parse multipart data manually
            boundary_bytes = f"--{boundary}".encode()
            parts = body.split(boundary_bytes)

            file_data = None
            filename = "upload"
            model_size = "tiny"
            api_key = None

            for part in parts:
                if b"Content-Disposition" not in part:
                    continue

                # Split headers and content
                if b"\r\n\r\n" in part:
                    header_section, content = part.split(b"\r\n\r\n", 1)
                else:
                    continue

                header_text = header_section.decode("utf-8", errors="ignore")

                # Remove trailing boundary markers
                if content.endswith(b"\r\n"):
                    content = content[:-2]
                if content.endswith(b"--"):
                    content = content[:-2]
                if content.endswith(b"\r\n"):
                    content = content[:-2]

                # Parse field name
                if 'name="file"' in header_text:
                    file_data = content
                    # Extract filename
                    for line in header_text.split("\r\n"):
                        if "filename=" in line:
                            start = line.find('filename="') + 10
                            end = line.find('"', start)
                            if end > start:
                                filename = line[start:end]
                elif 'name="model"' in header_text:
                    model_size = content.decode("utf-8").strip()
                elif 'name="api_key"' in header_text:
                    api_key = content.decode("utf-8").strip() or None

            if not file_data:
                self._json_response(400, {"error": "No file provided"})
                return

            # Check file extension
            ext = Path(filename).suffix.lower()

            if ext not in AUDIO_FORMATS and ext not in VIDEO_FORMATS:
                self._json_response(400, {
                    "error": f"Unsupported format: {ext}",
                    "supported": list(AUDIO_FORMATS | VIDEO_FORMATS)
                })
                return

            # Save file
            job_id = str(uuid.uuid4())[:8]
            file_path = UPLOAD_DIR / f"{job_id}{ext}"

            with open(file_path, "wb") as f:
                f.write(file_data)

            # Create job
            jobs[job_id] = {
                "id": job_id,
                "filename": filename,
                "status": "queued",
                "progress": 0,
                "stage": "Queued",
                "transcription": None,
                "summary": None,
                "error": None
            }

            # Start processing in background
            thread = threading.Thread(
                target=process_job,
                args=(job_id, str(file_path), model_size, api_key)
            )
            thread.start()

            self._json_response(200, {"job_id": job_id})
        else:
            self._json_response(400, {"error": "Expected multipart/form-data"})

    def _json_response(self, status: int, data: dict):
        """Send JSON response."""
        body = json.dumps(data, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(body)

    def do_OPTIONS(self):
        """Handle CORS preflight."""
        self.send_response(200)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()

    def log_message(self, format, *args):
        """Custom logging."""
        print(f"[Transcriber] {args[0]}")


def run_server(port: int = 8080):
    """Run the HTTP server."""
    server = HTTPServer(("127.0.0.1", port), TranscriberHandler)
    print(f"Transcriber server running on http://127.0.0.1:{port}")
    server.serve_forever()


if __name__ == "__main__":
    import sys
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
    run_server(port)
