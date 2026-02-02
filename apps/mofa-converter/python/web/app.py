"""
Content Converter Backend

Convert between audio, video, and text formats.
"""

import os
import json
import uuid
import time
import tempfile
import subprocess
from pathlib import Path
from typing import Optional, Dict, Any, List
from http.server import HTTPServer, SimpleHTTPRequestHandler
from urllib.parse import parse_qs, urlparse
import threading

# Global state for conversion jobs
jobs: Dict[str, Dict[str, Any]] = {}

# Configuration
UPLOAD_DIR = Path(tempfile.gettempdir()) / "mofa-converter"
UPLOAD_DIR.mkdir(exist_ok=True)
OUTPUT_DIR = UPLOAD_DIR / "outputs"
OUTPUT_DIR.mkdir(exist_ok=True)

# Supported formats
AUDIO_FORMATS = {'.mp3', '.wav', '.m4a', '.flac', '.ogg', '.aac', '.wma'}
VIDEO_FORMATS = {'.mp4', '.mkv', '.avi', '.mov', '.webm', '.flv', '.wmv'}
TEXT_FORMATS = {'.txt', '.md', '.json', '.srt'}

ALL_FORMATS = AUDIO_FORMATS | VIDEO_FORMATS | TEXT_FORMATS


def get_file_type(ext: str) -> str:
    """Get file type from extension."""
    ext = ext.lower()
    if ext in AUDIO_FORMATS:
        return 'audio'
    if ext in VIDEO_FORMATS:
        return 'video'
    if ext in TEXT_FORMATS:
        return 'text'
    return 'unknown'


def update_job(job_id: str, **kwargs):
    """Update job fields."""
    if job_id in jobs:
        jobs[job_id].update(kwargs)


def extract_audio_from_video(video_path: str, output_path: str) -> bool:
    """Extract audio from video using ffmpeg."""
    try:
        cmd = [
            'ffmpeg', '-i', video_path,
            '-vn',
            '-acodec', 'pcm_s16le',
            '-ar', '16000',
            '-ac', '1',
            '-y',
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


def transcribe_with_whisper(audio_path: str, model_size: str = "base") -> Optional[Dict]:
    """Transcribe audio using faster-whisper."""
    try:
        from faster_whisper import WhisperModel

        model_dir = get_whisper_model_dir()
        if model_dir:
            model = WhisperModel(model_size, device="cpu", compute_type="int8", download_root=model_dir)
        else:
            model = WhisperModel(model_size, device="cpu", compute_type="int8")

        segments, info = model.transcribe(audio_path, beam_size=5)

        result_segments = []
        full_text = []

        for segment in segments:
            result_segments.append({
                "start": segment.start,
                "end": segment.end,
                "text": segment.text.strip()
            })
            full_text.append(segment.text.strip())

        return {
            "language": info.language,
            "language_probability": info.language_probability,
            "duration": info.duration,
            "text": " ".join(full_text),
            "segments": result_segments
        }
    except ImportError:
        return {"error": "faster-whisper not installed. Run: pip install faster-whisper"}
    except Exception as e:
        return {"error": str(e)}


def generate_summary(text: str, api_key: str, model: str = "gpt-4o-mini") -> str:
    """Generate summary using OpenAI API."""
    if not api_key:
        return "[Error: API key required]"

    try:
        import openai
        client = openai.OpenAI(api_key=api_key)

        response = client.chat.completions.create(
            model=model,
            messages=[
                {
                    "role": "system",
                    "content": "You are a helpful assistant that summarizes content. Provide a clear, structured summary with key points in the same language as the input."
                },
                {
                    "role": "user",
                    "content": f"Please summarize the following:\n\n{text[:8000]}"
                }
            ],
            max_tokens=1000
        )
        return response.choices[0].message.content
    except ImportError:
        return "[Error: openai not installed. Run: pip install openai]"
    except Exception as e:
        return f"[Error: {e}]"


def text_to_speech(text: str, output_path: str, voice: str = "zh-CN-XiaoxiaoNeural") -> bool:
    """Convert text to speech using edge-tts."""
    try:
        import edge_tts
        import asyncio

        async def _tts():
            communicate = edge_tts.Communicate(text, voice)
            await communicate.save(output_path)

        asyncio.run(_tts())
        return True
    except ImportError:
        # Fallback to macOS say command
        try:
            subprocess.run(['say', '-o', output_path, text], check=True, timeout=120)
            return True
        except:
            return False
    except Exception as e:
        print(f"TTS error: {e}")
        return False


def generate_slideshow_video(text: str, output_path: str, duration: int = 10) -> bool:
    """Generate a simple slideshow video with text."""
    try:
        from PIL import Image, ImageDraw, ImageFont
        import numpy as np

        # Create frames with text
        frames = []
        width, height = 1920, 1080

        # Split text into chunks
        chunks = [text[i:i+100] for i in range(0, min(len(text), 500), 100)]
        if not chunks:
            chunks = ["Video Generated"]

        for chunk in chunks:
            img = Image.new('RGB', (width, height), color=(30, 30, 30))
            draw = ImageDraw.Draw(img)

            # Try to load font, fallback to default
            try:
                font = ImageFont.truetype("/System/Library/Fonts/PingFang.ttc", 60)
            except:
                font = ImageFont.load_default()

            # Wrap text
            words = chunk
            bbox = draw.textbbox((0, 0), words, font=font)
            text_width = bbox[2] - bbox[0]
            text_height = bbox[3] - bbox[1]
            x = (width - text_width) // 2
            y = (height - text_height) // 2

            draw.text((x, y), words, font=font, fill=(255, 255, 255))
            frames.append(np.array(img))

        # Duplicate frames to reach desired duration (10fps)
        fps = 10
        total_frames = duration * fps
        frames_per_chunk = max(1, total_frames // len(frames))

        final_frames = []
        for frame in frames:
            for _ in range(frames_per_chunk):
                final_frames.append(frame)

        # Write video using imageio
        try:
            import imageio
            imageio.mimsave(output_path, final_frames, fps=fps)
            return True
        except ImportError:
            # Fallback: create a simple video with ffmpeg
            temp_dir = tempfile.mkdtemp()
            for i, frame in enumerate(final_frames):
                Image.fromarray(frame).save(f"{temp_dir}/frame_{i:04d}.png")

            cmd = [
                'ffmpeg', '-y', '-framerate', str(fps),
                '-i', f'{temp_dir}/frame_%04d.png',
                '-c:v', 'libx264', '-pix_fmt', 'yuv420p',
                output_path
            ]
            subprocess.run(cmd, capture_output=True, timeout=60)

            # Cleanup
            for f in Path(temp_dir).glob("*.png"):
                f.unlink()
            Path(temp_dir).rmdir()

            return True

    except Exception as e:
        print(f"Video generation error: {e}")
        return False


def process_conversion_job(job_id: str, source_type: str, target_type: str,
                           input_path: Optional[str], text_content: Optional[str],
                           options: Dict[str, Any]):
    """Process a conversion job."""
    job = jobs[job_id]
    api_key = options.get('api_key', '')
    model_size = options.get('model_size', 'base')

    try:
        update_job(job_id, status='processing', progress=10, stage='准备中...')
        time.sleep(0.5)

        # Audio/Video -> Text (Transcription)
        if source_type in ('audio', 'video') and target_type == 'text':
            update_job(job_id, progress=20, stage='提取音频...')

            audio_path = input_path
            if source_type == 'video':
                audio_path = str(UPLOAD_DIR / f"{job_id}_audio.wav")
                if not extract_audio_from_video(input_path, audio_path):
                    update_job(job_id, status='error', error='音频提取失败')
                    return

            update_job(job_id, progress=40, stage='语音转录中...')
            result = transcribe_with_whisper(audio_path, model_size)

            if result and 'error' not in result:
                update_job(job_id, progress=80, stage='生成文稿...')
                update_job(job_id,
                    result={
                        'type': 'text',
                        'content': result['text'],
                        'segments': result['segments'],
                        'language': result['language'],
                        'duration': result['duration']
                    },
                    progress=100,
                    status='completed',
                    stage='完成'
                )
            else:
                error_msg = result.get('error', '转录失败') if result else '转录失败'
                update_job(job_id, status='error', error=error_msg)

        # Text -> Text (Summary)
        elif source_type == 'text' and target_type == 'text':
            update_job(job_id, progress=30, stage='生成摘要...')

            summary = generate_summary(text_content or '', api_key)

            if summary.startswith('[Error'):
                update_job(job_id, status='error', error=summary)
            else:
                update_job(job_id,
                    result={
                        'type': 'text',
                        'content': summary,
                        'word_count': len(summary),
                        'original_length': len(text_content or '')
                    },
                    progress=100,
                    status='completed',
                    stage='完成'
                )

        # Text -> Audio (TTS)
        elif source_type == 'text' and target_type == 'audio':
            update_job(job_id, progress=30, stage='语音合成中...')

            output_path = str(OUTPUT_DIR / f"{job_id}_output.mp3")
            voice = options.get('voice', 'zh-CN-XiaoxiaoNeural')

            if text_to_speech(text_content or '', output_path, voice):
                update_job(job_id,
                    result={
                        'type': 'audio',
                        'url': f'/api/download/{job_id}_output.mp3',
                        'filename': f'{job_id}_output.mp3',
                        'text_length': len(text_content or '')
                    },
                    progress=100,
                    status='completed',
                    stage='完成'
                )
            else:
                update_job(job_id, status='error', error='语音合成失败')

        # Text -> Video (Slideshow)
        elif source_type == 'text' and target_type == 'video':
            update_job(job_id, progress=30, stage='生成视频中...')

            output_path = str(OUTPUT_DIR / f"{job_id}_output.mp4")
            duration = int(options.get('duration', 10))

            if generate_slideshow_video(text_content or '', output_path, duration):
                update_job(job_id,
                    result={
                        'type': 'video',
                        'url': f'/api/download/{job_id}_output.mp4',
                        'filename': f'{job_id}_output.mp4',
                        'duration': duration
                    },
                    progress=100,
                    status='completed',
                    stage='完成'
                )
            else:
                update_job(job_id, status='error', error='视频生成失败')

        # Audio -> Video (Waveform visualization)
        elif source_type == 'audio' and target_type == 'video':
            update_job(job_id, progress=30, stage='生成波形视频...')
            # For now, just copy audio and create a placeholder
            output_path = str(OUTPUT_DIR / f"{job_id}_output.mp4")

            # Simple: create video with audio
            cmd = [
                'ffmpeg', '-y',
                '-f', 'lavfi', '-i', 'color=c=black:s=1920x1080:d=10',
                '-i', input_path,
                '-shortest',
                '-c:v', 'libx264', '-c:a', 'aac',
                output_path
            ]
            try:
                subprocess.run(cmd, capture_output=True, timeout=60)
                update_job(job_id,
                    result={
                        'type': 'video',
                        'url': f'/api/download/{job_id}_output.mp4',
                        'filename': f'{job_id}_output.mp4'
                    },
                    progress=100,
                    status='completed',
                    stage='完成'
                )
            except:
                update_job(job_id, status='error', error='视频生成失败')

        else:
            update_job(job_id, status='error', error=f'不支持的转换: {source_type} -> {target_type}')

        # Cleanup upload file
        if input_path and Path(input_path).exists():
            try:
                Path(input_path).unlink()
            except:
                pass

    except Exception as e:
        update_job(job_id, status='error', error=str(e))


class ConverterHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        self.directory = str(Path(__file__).parent / 'static')
        super().__init__(*args, directory=self.directory, **kwargs)

    def do_GET(self):
        parsed = urlparse(self.path)

        if parsed.path == '/api/info':
            self._json_response({
                'name': 'Content Converter',
                'version': '1.0.0',
                'formats': {
                    'audio': list(AUDIO_FORMATS),
                    'video': list(VIDEO_FORMATS),
                    'text': list(TEXT_FORMATS)
                }
            })
        elif parsed.path == '/api/status':
            params = parse_qs(parsed.query)
            job_id = params.get('id', [None])[0]
            if job_id and job_id in jobs:
                self._json_response(jobs[job_id])
            else:
                self._json_response({'error': 'Job not found'}, 404)
        elif parsed.path.startswith('/api/download/'):
            filename = parsed.path.replace('/api/download/', '')
            file_path = OUTPUT_DIR / filename
            if file_path.exists() and file_path.is_file():
                self._serve_file(file_path)
            else:
                self._json_response({'error': 'File not found'}, 404)
        else:
            super().do_GET()

    def _serve_file(self, file_path: Path):
        """Serve a file for download."""
        import mimetypes
        mime_type, _ = mimetypes.guess_type(str(file_path))
        if not mime_type:
            mime_type = 'application/octet-stream'

        self.send_response(200)
        self.send_header('Content-Type', mime_type)
        self.send_header('Content-Disposition', f'attachment; filename="{file_path.name}"')
        self.send_header('Content-Length', str(file_path.stat().st_size))
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()

        with open(file_path, 'rb') as f:
            self.wfile.write(f.read())

    def do_POST(self):
        parsed = urlparse(self.path)

        if parsed.path == '/api/convert':
            self._handle_convert()
        elif parsed.path == '/api/upload':
            self._handle_upload()
        else:
            self._json_response({'error': 'Not found'}, 404)

    def _handle_convert(self):
        """Handle text-based conversion request."""
        data = self._read_json()

        if not data:
            self._json_response({'error': 'Invalid JSON'}, 400)
            return

        source_type = data.get('source_type')
        target_type = data.get('target_type')
        content = data.get('content', '')
        options = data.get('options', {})

        if not source_type or not target_type:
            self._json_response({'error': 'Missing source or target type'}, 400)
            return

        job_id = str(uuid.uuid4())[:8]
        jobs[job_id] = {
            'id': job_id,
            'source_type': source_type,
            'target_type': target_type,
            'status': 'queued',
            'progress': 0,
            'stage': '排队中',
            'result': None,
            'error': None
        }

        thread = threading.Thread(
            target=process_conversion_job,
            args=(job_id, source_type, target_type, None, content, options)
        )
        thread.start()

        self._json_response({'job_id': job_id})

    def _handle_upload(self):
        """Handle file upload and conversion."""
        content_type = self.headers.get('Content-Type', '')

        if 'multipart/form-data' not in content_type:
            self._json_response({'error': 'Expected multipart/form-data'}, 400)
            return

        boundary = None
        for part in content_type.split(';'):
            part = part.strip()
            if part.startswith('boundary='):
                boundary = part[9:].strip('"')
                break

        if not boundary:
            self._json_response({'error': 'No boundary found'}, 400)
            return

        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length)

        boundary_bytes = f'--{boundary}'.encode()
        parts = body.split(boundary_bytes)

        file_data = None
        filename = 'upload'
        target_type = 'text'
        options = {}

        for part in parts:
            if b'Content-Disposition' not in part:
                continue

            if b'\r\n\r\n' not in part:
                continue

            header_section, content = part.split(b'\r\n\r\n', 1)
            header_text = header_section.decode('utf-8', errors='ignore')

            if content.endswith(b'\r\n'):
                content = content[:-2]
            if content.endswith(b'--'):
                content = content[:-2]
            if content.endswith(b'\r\n'):
                content = content[:-2]

            if 'name="file"' in header_text:
                file_data = content
                for line in header_text.split('\r\n'):
                    if 'filename=' in line:
                        start = line.find('filename="') + 10
                        end = line.find('"', start)
                        if end > start:
                            filename = line[start:end]
            elif 'name="target"' in header_text:
                target_type = content.decode('utf-8').strip()
            elif 'name="options"' in header_text:
                try:
                    options = json.loads(content.decode('utf-8'))
                except:
                    pass

        if not file_data:
            self._json_response({'error': 'No file provided'}, 400)
            return

        ext = Path(filename).suffix.lower()
        if ext not in ALL_FORMATS:
            self._json_response({
                'error': f'Unsupported format: {ext}',
                'supported': list(ALL_FORMATS)
            }, 400)
            return

        source_type = get_file_type(ext)

        # Save file
        job_id = str(uuid.uuid4())[:8]
        file_path = UPLOAD_DIR / f"{job_id}{ext}"

        with open(file_path, 'wb') as f:
            f.write(file_data)

        jobs[job_id] = {
            'id': job_id,
            'filename': filename,
            'source_type': source_type,
            'target_type': target_type,
            'status': 'queued',
            'progress': 0,
            'stage': '排队中',
            'result': None,
            'error': None
        }

        thread = threading.Thread(
            target=process_conversion_job,
            args=(job_id, source_type, target_type, str(file_path), None, options)
        )
        thread.start()

        self._json_response({'job_id': job_id})

    def _read_json(self):
        try:
            length = int(self.headers.get('Content-Length', 0))
            body = self.rfile.read(length)
            return json.loads(body.decode('utf-8'))
        except:
            return None

    def _json_response(self, data, status=200):
        body = json.dumps(data, ensure_ascii=False).encode('utf-8')
        self.send_response(status)
        self.send_header('Content-Type', 'application/json; charset=utf-8')
        self.send_header('Content-Length', str(len(body)))
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        self.wfile.write(body)

    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        self.end_headers()

    def log_message(self, format, *args):
        print(f'[Converter] {args[0]}')


def run_server(port: int = 8080):
    server = HTTPServer(('127.0.0.1', port), ConverterHandler)
    print(f'Content Converter server running on http://127.0.0.1:{port}')
    server.serve_forever()


if __name__ == '__main__':
    import sys
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
    run_server(port)
