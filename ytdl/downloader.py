"""yt-dlp 로 한 건씩 받는 작업 스레드. 재생목록이면 항목만 펼쳐 알리고 끝난다."""

from __future__ import annotations

import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

import yt_dlp
from PySide6.QtCore import QThread, Signal
from yt_dlp.postprocessor import PostProcessor
from yt_dlp.utils import replace_extension

# exe 로 묶였을 때 함께 들어간 ffmpeg 위치
BASE_DIR = Path(getattr(sys, "_MEIPASS", Path(__file__).resolve().parent.parent))

FFMPEG_DIR = str(BASE_DIR) if (BASE_DIR / "ffmpeg.exe").exists() else None


@dataclass
class PlaylistEntry:
    url: str
    title: str
    index: int


@dataclass
class PlaylistResult:
    title: str
    site_key: str
    webpage_url: str
    entries: list[PlaylistEntry]
    skipped: int


def playlist_entries(info: dict, is_archived: Callable[[dict], bool] = lambda e: False
                     ) -> tuple[list[PlaylistEntry], int]:
    """재생목록 결과에서 받을 항목. 순번이 없으면 원래 목록에서의 위치를 쓴다."""
    entries: list[PlaylistEntry] = []
    skipped = 0
    for pos, entry in enumerate(info.get("entries") or [], start=1):
        if not isinstance(entry, dict):
            continue
        url = entry.get("url") or entry.get("webpage_url")
        if not url:
            continue
        if is_archived(entry):
            skipped += 1
            continue
        entries.append(PlaylistEntry(url, entry.get("title") or url, entry.get("playlist_index") or pos))
    return entries, skipped


class NumberIfExistsPP(PostProcessor):
    """같은 이름의 파일이 있으면 yt-dlp 는 받지 않고 넘어간다. 브라우저처럼 `이름 (1).mp4` 로 새로 받는다.
    최종 확장자는 형식을 고른 뒤에야 알 수 있어 `video` 단계에 붙인다. 덮어쓰기를 켰으면 손대지 않는다."""

    _EXT = ".%(ext)s"

    def run(self, info):
        ydl = self._downloader
        base = ydl.params["outtmpl"]["default"]
        if ydl.params.get("overwrites") or not base.endswith(self._EXT):
            return [], info
        stem, n = base[:-len(self._EXT)], 0
        while self._taken(ydl.prepare_filename(info), info):
            n += 1
            ydl.params["outtmpl"] = {**ydl.params["outtmpl"], "default": f"{stem} ({n}){self._EXT}"}
        return [], info

    def _taken(self, filename: str, info: dict) -> bool:
        # yt-dlp 가 "이미 받음" 으로 보는 경로와 같게: 받은 그대로와 변환(mp3 등)을 마친 이름
        ext = info.get("ext")
        final = self._downloader.params.get("final_ext") or ext
        return any(Path(p).exists() for p in (filename, replace_extension(filename, final, ext)))


# 위에서부터 맞춰 본다. 쿠키 실패 메시지에도 "Sign in" 이 섞일 수 있어 쿠키가 먼저다.
_ERRORS = [
    (("unsupported url", "is not a valid url"), "지원하지 않는 링크예요"),
    (("cookies database", "failed to decrypt", "cookie database"),
     "브라우저 쿠키를 읽지 못했어요. 브라우저를 닫고 다시 시도하거나 cookies.txt 를 써 주세요"),
    (("sign in", "login required", "private video", "members-only"),
     "로그인이 필요한 영상이에요. ⚙ 설정에서 쿠키를 켜 보세요"),
    (("video unavailable", "video is unavailable", "this video is not available"), "삭제되었거나 볼 수 없는 영상이에요"),
    (("requested format is not available",), "고른 화질·형식이 없는 영상이에요. 세부 옵션을 바꿔 보세요"),
    (("getaddrinfo", "timed out", "unable to connect", "proxyerror"), "인터넷 연결이나 프록시 설정을 확인해 주세요"),
]


def friendly_error(e: Exception) -> str:
    msg = str(e)
    low = msg.lower()
    if ("ffmpeg" in low or "ffprobe" in low) and ("not found" in low or "not installed" in low):
        return "변환 도구(ffmpeg)를 찾지 못했어요"
    for needles, text in _ERRORS:
        if any(n in low for n in needles):
            return text
    return msg.replace("ERROR: ", "")[:160]


class DownloadJob(QThread):
    titled = Signal(str)
    progress = Signal(float, str)   # 퍼센트, 상태 문구
    finished_ok = Signal(str)       # 저장된 파일 경로
    failed = Signal(str)
    site_found = Signal(str, str)   # extractor_key, webpage_url
    expanded = Signal(object)       # PlaylistResult
    skipped = Signal()              # 이미 받은 기록이 있어 받지 않음

    def __init__(self, url: str, ydl_opts: dict):
        super().__init__()
        self.url, self.ydl_opts = url, ydl_opts
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
        opts = {**self.ydl_opts, "progress_hooks": [self._hook]}
        try:
            with yt_dlp.YoutubeDL(opts) as ydl:
                ydl.add_post_processor(NumberIfExistsPP(ydl), when="video")
                # yt-dlp 는 재생목록을 펼칠 때 받은 기록에 있는 항목을 말없이 뺀다.
                # 몇 개를 건너뛰었는지 직접 세려고 펼치는 동안만 기록을 비워 둔다.
                archive, ydl.archive = ydl.archive, set()
                try:
                    info = ydl.extract_info(self.url, download=False)
                finally:
                    ydl.archive = archive
                if info is None:
                    self.skipped.emit()
                    return
                site_key = info.get("extractor_key") or ""
                page = info.get("webpage_url") or self.url
                self.site_found.emit(site_key, page)
                if info.get("_type") == "playlist":
                    archived = ydl.in_download_archive if self.ydl_opts.get("download_archive") else (lambda e: False)
                    entries, skipped = playlist_entries(info, archived)
                    self.expanded.emit(PlaylistResult(
                        title=info.get("title") or info.get("id") or self.url, site_key=site_key,
                        webpage_url=page, entries=entries, skipped=skipped))
                    return
                self.titled.emit(info.get("title") or self.url)
                self._parts = len(info.get("requested_formats") or []) or 1
                info = ydl.process_ie_result(info, download=True)
            downloads = (info or {}).get("requested_downloads") or []
            if not downloads:
                self.skipped.emit()
                return
            # 병합·변환을 거친 최종 파일 경로
            self.finished_ok.emit(downloads[0].get("filepath") or self._file)
        except Exception as e:  # noqa: BLE001 - 사용자에게 보여 줄 문구로 바꾼다
            self.failed.emit(friendly_error(e))
