"""사용자가 고른 다운로드 옵션과 앱 설정. Qt 없이 동작한다.

화면은 `DownloadOptions`(세부 옵션 패널)와 `AppSettings`(⚙ 설정 창)를 채우기만 하고,
다운로드 작업은 `to_ydl_opts()`가 만든 yt-dlp 옵션 dict 를 받아 쓰기만 한다.
"""

from __future__ import annotations

import dataclasses
from dataclasses import dataclass

QUALITIES = ("best", "2160", "1440", "1080", "720", "480", "360")
CONTAINERS = ("mp4", "mkv", "webm")
CODECS = ("auto", "h264", "vp9", "av1")
AUDIO_FORMATS = ("mp3", "m4a", "opus", "flac", "wav")
AUDIO_QUALITIES = ("best", "320", "192", "128")
SUB_MODES = ("embed", "file")
SUB_FORMATS = ("srt", "vtt", "ass")
PLAYLIST_MODES = ("single", "all")
COOKIE_MODES = ("none", "browser", "file")
BROWSERS = ("firefox", "edge", "chrome", "whale", "brave", "opera", "vivaldi", "chromium")
NAME_TEMPLATES = {
    "title": "%(title)s.%(ext)s",
    "title_id": "%(title)s [%(id)s].%(ext)s",
    "uploader_title": "%(uploader)s - %(title)s.%(ext)s",
    "date_title": "%(upload_date>%Y-%m-%d)s %(title)s.%(ext)s",
}
LOSSLESS = ("flac", "wav")


@dataclass
class DownloadOptions:
    """세부 옵션 패널 값."""
    kind: str = "video"
    quality: str = "best"
    container: str = "mp4"
    codec: str = "auto"
    audio_format: str = "mp3"
    audio_quality: str = "192"
    subs: bool = False
    sub_langs: str = "ko"
    auto_subs: bool = False
    sub_mode: str = "embed"
    sub_format: str = "srt"
    embed_thumbnail: bool = False
    save_thumbnail: bool = False
    embed_metadata: bool = False
    embed_chapters: bool = False
    split_chapters: bool = False
    section_start: str = ""
    section_end: str = ""
    playlist: str = "single"
    playlist_items: str = ""
    number_prefix: bool = False
    playlist_folder: bool = False
    skip_archived: bool = False


@dataclass
class AppSettings:
    """⚙ 설정 창 값."""
    cookies: str = "none"
    cookie_browser: str = "firefox"
    cookie_file: str = ""
    proxy: str = ""
    rate_limit: str = ""
    retries: int = 10
    name_template: str = "title"
    custom_template: str = "%(title)s.%(ext)s"
    overwrite: bool = False
    concurrency: int = 2
    extra_args: str = ""


# 저장값을 되돌릴 때 이 목록 밖의 값은 기본값으로 둔다
_CHOICES = {
    "kind": ("video", "audio"),
    "quality": QUALITIES,
    "container": CONTAINERS,
    "codec": CODECS,
    "audio_format": AUDIO_FORMATS,
    "audio_quality": AUDIO_QUALITIES,
    "sub_mode": SUB_MODES,
    "sub_format": SUB_FORMATS,
    "playlist": PLAYLIST_MODES,
    "cookies": COOKIE_MODES,
    "cookie_browser": BROWSERS,
    "name_template": (*NAME_TEMPLATES, "custom"),
}
_RANGES = {"retries": (0, 50), "concurrency": (1, 5)}

PRESETS: dict[str, dict[str, str]] = {
    "best": {"kind": "video", "quality": "best", "container": "mp4", "codec": "auto"},
    "720p": {"kind": "video", "quality": "720", "container": "mp4", "codec": "auto"},
    "mp3": {"kind": "audio", "audio_format": "mp3", "audio_quality": "192"},
}
PRESET_LABELS = {"best": "최고 화질", "720p": "720p", "mp3": "MP3 음원"}


def apply_preset(opts: DownloadOptions, key: str) -> DownloadOptions:
    return dataclasses.replace(opts, **PRESETS[key])


def matching_preset(opts: DownloadOptions) -> str | None:
    """형식 값이 같은 프리셋. 없으면 None (= 사용자 지정)."""
    for key, values in PRESETS.items():
        if all(getattr(opts, k) == v for k, v in values.items()):
            return key
    return None


def to_dict(obj) -> dict[str, object]:
    return dataclasses.asdict(obj)


def _coerce(field: dataclasses.Field, value, default):
    if field.type in (bool, "bool"):
        if isinstance(value, bool):
            return value
        text = str(value).strip().lower()
        return {"true": True, "1": True, "false": False, "0": False}.get(text, default)
    if field.type in (int, "int"):
        try:
            number = int(value)
        except (TypeError, ValueError):
            return default
        lo, hi = _RANGES.get(field.name, (number, number))
        return number if lo <= number <= hi else default
    text = "" if value is None else str(value)
    choices = _CHOICES.get(field.name)
    return text if choices is None or text in choices else default


def _from_dict(cls, d: dict):
    defaults = cls()
    values = {}
    for f in dataclasses.fields(cls):
        if f.name in d:
            values[f.name] = _coerce(f, d[f.name], getattr(defaults, f.name))
    return cls(**values)


def options_from_dict(d: dict, legacy_format: str | None = None) -> DownloadOptions:
    """저장된 패널 값. 아직 저장한 적 없으면 옛 앱의 `format` 값을 프리셋으로 옮긴다."""
    opts = _from_dict(DownloadOptions, d)
    if "kind" not in d and legacy_format in PRESETS:
        opts = apply_preset(opts, legacy_format)
    return opts


def settings_from_dict(d: dict) -> AppSettings:
    return _from_dict(AppSettings, d)


def describe(opts: DownloadOptions) -> str:
    """카드에 쓸 짧은 요약. 예: `1080p · MKV`, `MP3 · 192k`."""
    if opts.kind == "audio":
        text = opts.audio_format.upper()
        if opts.audio_format not in LOSSLESS:
            text += " · 최고 음질" if opts.audio_quality == "best" else f" · {opts.audio_quality}k"
    else:
        quality = "최고 화질" if opts.quality == "best" else f"{opts.quality}p"
        text = f"{quality} · {opts.container.upper()}"
    if opts.subs:
        text += " · 자막"
    return text


def changed_summary(opts: DownloadOptions) -> list[str]:
    """세부 옵션 버튼에 붙일, 기본값과 달라진 항목."""
    checks = [
        ("형식", matching_preset(opts) is None),
        ("자막", opts.subs),
        ("썸네일", opts.embed_thumbnail or opts.save_thumbnail),
        ("메타데이터", opts.embed_metadata),
        ("챕터", opts.embed_chapters or opts.split_chapters),
        ("구간", bool(opts.section_start or opts.section_end)),
        ("재생목록", opts.playlist == "all" or bool(opts.playlist_items) or opts.number_prefix
         or opts.playlist_folder or opts.skip_archived),
    ]
    return [name for name, changed in checks if changed]
