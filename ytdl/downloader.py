"""yt-dlp 로 한 건씩 받는 작업 스레드."""

from __future__ import annotations

import sys
import time
from pathlib import Path

import yt_dlp
from PySide6.QtCore import QThread, Signal

# exe 로 묶였을 때 함께 들어간 ffmpeg 위치
BASE_DIR = Path(getattr(sys, "_MEIPASS", Path(__file__).resolve().parent.parent))

FORMATS = {
    "best": ("최고 화질", {
        "format": "bv*[ext=mp4]+ba[ext=m4a]/bv*+ba/b",
        "merge_output_format": "mp4",
    }),
    "720p": ("720p", {
        "format": "bv*[height<=720][ext=mp4]+ba[ext=m4a]/bv*[height<=720]+ba/b[height<=720]",
        "merge_output_format": "mp4",
    }),
    "mp3": ("MP3 음원", {
        "format": "ba/b",
        "postprocessors": [{"key": "FFmpegExtractAudio", "preferredcodec": "mp3",
                            "preferredquality": "192"}],
    }),
}


def friendly_error(e: Exception) -> str:
    msg = str(e)
    if "is not a valid URL" in msg or "Unsupported URL" in msg:
        return "올바른 유튜브 링크가 아니에요"
    if "Private video" in msg or "Sign in" in msg:
        return "비공개이거나 로그인이 필요한 영상이에요"
    if "Video unavailable" in msg:
        return "삭제되었거나 볼 수 없는 영상이에요"
    if "getaddrinfo" in msg or "timed out" in msg:
        return "인터넷 연결을 확인해 주세요"
    return msg.replace("ERROR: ", "")[:160]


class DownloadJob(QThread):
    titled = Signal(str)
    progress = Signal(float, str)   # 퍼센트, 상태 문구
    finished_ok = Signal(str)       # 저장된 파일 경로
    failed = Signal(str)

    def __init__(self, url: str, fmt: str, folder: str):
        super().__init__()
        self.url, self.fmt, self.folder = url, fmt, folder
        self._file = ""
        # 영상·음성을 따로 받으면 스트림이 여러 개다. 전체를 하나의 진행률로 합친다.
        self._parts = 1
        self._part = 0
        self._last_pct = 0.0
        self._last_emit = 0.0

    def _hook(self, d: dict) -> None:
        if d["status"] == "downloading":
            total = d.get("total_bytes") or d.get("total_bytes_estimate")
            if not total:
                return
            frac = min(d["downloaded_bytes"] / total, 1.0)
            pct = (self._part + frac) / self._parts * 100
            # 크기 추정치가 바뀌어도 뒤로 가지 않게
            pct = max(pct, self._last_pct)
            now = time.monotonic()
            # 초당 8번까지만 알린다
            if now - self._last_emit < 0.12 and pct - self._last_pct < 5:
                return
            self._last_pct, self._last_emit = pct, now
            speed = (d.get("_speed_str") or "").strip()
            self.progress.emit(pct, f"받는 중 · {speed}")
        elif d["status"] == "finished":
            self._file = d.get("filename", "")
            self._part = min(self._part + 1, self._parts)
            self._last_pct = max(self._last_pct, self._part / self._parts * 100)
            if self._part >= self._parts:
                self.progress.emit(100, "파일 정리 중…")

    def run(self) -> None:
        opts = {
            "outtmpl": str(Path(self.folder) / "%(title)s.%(ext)s"),
            "progress_hooks": [self._hook],
            "noplaylist": True,
            "quiet": True,
            "no_warnings": True,
            "noprogress": True,
            **FORMATS[self.fmt][1],
        }
        if (BASE_DIR / "ffmpeg.exe").exists():
            opts["ffmpeg_location"] = str(BASE_DIR)
        try:
            with yt_dlp.YoutubeDL(opts) as ydl:
                info = ydl.extract_info(self.url, download=False)
                self.titled.emit(info.get("title") or self.url)
                self._parts = len(info.get("requested_formats") or []) or 1
                info = ydl.process_ie_result(info, download=True)
            # 병합·변환을 거친 최종 파일 경로
            done = (info.get("requested_downloads") or [{}])[0].get("filepath")
            self.finished_ok.emit(done or self._file)
        except Exception as e:  # noqa: BLE001 - 사용자에게 보여 줄 문구로 바꾼다
            self.failed.emit(friendly_error(e))
