"""다운로드 카드에 보여 줄 사이트 이름과 아이콘.

아이콘은 그 사이트의 /favicon.ico 에서 직접 받는다. 어차피 영상을 받으려고 접속하는 곳이라
제3자 서비스에 주소를 보내지 않는다. 받은 아이콘은 앱 데이터 폴더에 저장해 두고 다시 쓴다.
"""

from __future__ import annotations

import time
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Callable

from PySide6.QtCore import QObject, QRunnable, Qt, QThreadPool, Signal
from PySide6.QtGui import QColor, QFont, QImage, QPainter, QPixmap

from . import __version__, theme
from .app_info import NAME

# yt-dlp extractor_key 접두어 → 보기 좋은 이름
_NAMES = [
    ("youtube", "YouTube"), ("twitter", "X"), ("instagram", "Instagram"), ("tiktok", "TikTok"),
    ("facebook", "Facebook"), ("twitch", "Twitch"), ("vimeo", "Vimeo"), ("soundcloud", "SoundCloud"),
    ("dailymotion", "Dailymotion"), ("reddit", "Reddit"), ("bilibili", "bilibili"),
    ("niconico", "니코니코"), ("naver", "네이버 TV"), ("chzzk", "치지직"), ("afreecatv", "SOOP"),
    ("soop", "SOOP"), ("kakao", "카카오TV"),
]

ICON_PX = 32      # 저장·그리기 크기. 화면에는 16px 로 (고배율 화면에서 선명하게)
_MAX_BYTES = 256 * 1024


def domain_of(url: str) -> str:
    try:
        host = urllib.parse.urlsplit(url.strip()).hostname or ""
    except ValueError:
        return ""
    for prefix in ("www.", "m."):
        if host.startswith(prefix):
            host = host[len(prefix):]
    return host


def display_name(extractor_key: str, host: str) -> str:
    base = (extractor_key or "").split(":")[0]
    if not base or base.lower() == "generic":
        return host
    low = base.lower()
    for prefix, name in _NAMES:
        if low.startswith(prefix):
            return name
    return base


def icon_url(host: str) -> str:
    return f"https://{host}/favicon.ico"


def letter_of(name: str) -> str:
    return name[:1].upper() or "?"


def fetch_bytes(url: str) -> bytes | None:
    try:
        req = urllib.request.Request(url, headers={"User-Agent": f"{NAME}/{__version__}"})
        with urllib.request.urlopen(req, timeout=5) as resp:
            data = resp.read(_MAX_BYTES + 1)
    except Exception:  # noqa: BLE001 - 아이콘은 없어도 그만이다
        return None
    return None if len(data) > _MAX_BYTES else data


class _Relay(QObject):
    """작업 스레드 → 메인 스레드. 메인 스레드에서 만들어 둔다."""
    fetched = Signal(str, object)  # host, QImage | None


class _Fetch(QRunnable):
    def __init__(self, host: str, fetch: Callable[[str], bytes | None], relay: _Relay):
        super().__init__()
        self.host, self.fetch, self.relay = host, fetch, relay

    def run(self) -> None:
        data = self.fetch(icon_url(self.host))
        image = QImage.fromData(data) if data else QImage()
        if not image.isNull():
            image = image.scaled(ICON_PX, ICON_PX, Qt.AspectRatioMode.KeepAspectRatio,
                                 Qt.TransformationMode.SmoothTransformation)
        try:
            self.relay.fetched.emit(self.host, None if image.isNull() else image)
        except RuntimeError:
            pass  # 받는 사이에 앱이 끝나 받을 곳이 사라졌다


class SiteIcons(QObject):
    ready = Signal(str, QPixmap)

    def __init__(self, cache_dir: Path, fetch: Callable[[str], bytes | None] = fetch_bytes,
                 max_age_days: int = 30):
        super().__init__()
        self.cache_dir = Path(cache_dir)
        self._fetch = fetch
        self._max_age = max_age_days * 86400
        self._memory: dict[str, QPixmap] = {}
        self._tried: set[str] = set()  # 이번 실행에 받는 중이거나 실패한 곳
        self._relay = _Relay()
        self._relay.fetched.connect(self._on_fetched)

    def icon(self, host: str) -> QPixmap:
        if host in self._memory:
            return self._memory[host]
        path = self.cache_dir / f"{host}.png"
        if host and path.exists() and time.time() - path.stat().st_mtime < self._max_age:
            pix = QPixmap(str(path))
            if not pix.isNull():
                pix.setDevicePixelRatio(2)
                self._memory[host] = pix
                return pix
        if host and host not in self._tried:
            self._tried.add(host)
            QThreadPool.globalInstance().start(_Fetch(host, self._fetch, self._relay))
        return self.fallback(host)

    def fallback(self, host: str) -> QPixmap:
        pix = QPixmap(ICON_PX, ICON_PX)
        pix.fill(Qt.GlobalColor.transparent)
        p = QPainter(pix)
        p.setRenderHint(QPainter.RenderHint.Antialiasing)
        p.setPen(Qt.PenStyle.NoPen)
        p.setBrush(QColor(theme.color("accent")))
        p.drawEllipse(0, 0, ICON_PX, ICON_PX)
        font = QFont()
        font.setBold(True)
        font.setPixelSize(ICON_PX * 9 // 16)
        p.setFont(font)
        p.setPen(QColor("white"))
        p.drawText(pix.rect(), Qt.AlignmentFlag.AlignCenter, letter_of(host))
        p.end()
        pix.setDevicePixelRatio(2)
        return pix

    def _on_fetched(self, host: str, image: QImage | None) -> None:
        if image is None:
            return
        pix = QPixmap.fromImage(image)
        try:
            self.cache_dir.mkdir(parents=True, exist_ok=True)
            pix.save(str(self.cache_dir / f"{host}.png"), "PNG")
        except OSError:
            pass
        pix.setDevicePixelRatio(2)
        self._memory[host] = pix
        self.ready.emit(host, pix)
