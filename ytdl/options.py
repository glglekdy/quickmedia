"""사용자가 고른 다운로드 옵션과 앱 설정. Qt 없이 동작한다.

화면은 `DownloadOptions`(세부 옵션 패널)와 `AppSettings`(⚙ 설정 창)를 채우기만 하고,
다운로드 작업은 `to_ydl_opts()`가 만든 yt-dlp 옵션 dict 를 받아 쓰기만 한다.
"""

from __future__ import annotations

import dataclasses
import optparse
import os
from dataclasses import dataclass

import yt_dlp
from yt_dlp.utils import PlaylistEntries, parse_bytes, parse_duration, sanitize_filename

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


# ---------- yt-dlp 옵션으로 바꾸기 ----------

@dataclass(frozen=True)
class PlaylistContext:
    """재생목록에서 펼친 항목 하나. 항목 작업은 영상 하나만 받으므로 재생목록 필드를 직접 넣는다."""
    title: str
    index: int
    count: int


_VCODECS = {"h264": "vcodec:h264", "vp9": "vcodec:vp9", "av1": "vcodec:av01"}
_EXT_SORT = {"mp4": "ext:mp4:m4a", "webm": "ext:webm:webm"}


def _template(settings: AppSettings) -> str:
    if settings.name_template == "custom":
        return settings.custom_template
    return NAME_TEMPLATES.get(settings.name_template, NAME_TEMPLATES["title"])


_RESERVED = {"CON", "PRN", "AUX", "NUL", *(f"COM{i}" for i in range(1, 10)), *(f"LPT{i}" for i in range(1, 10))}


def _safe_folder(title: str) -> str:
    """재생목록 제목으로 만든 폴더 이름. 제목은 남이 정한 값이라 저장 폴더 밖을 가리키면 안 된다."""
    # Windows 는 끝의 점·공백을 버리므로 ".." 같은 이름은 여기서 비게 된다
    folder = sanitize_filename(title).strip().rstrip(" .")
    if not folder:
        return "playlist"
    if folder.split(".")[0].upper() in _RESERVED:
        folder = "_" + folder
    return folder


def _playlist_prefix(opts: DownloadOptions, playlist: PlaylistContext) -> str:
    prefix = ""
    if opts.playlist_folder:
        prefix += _safe_folder(playlist.title).replace("%", "%%") + "/"
    if opts.number_prefix:
        prefix += str(playlist.index).zfill(max(2, len(str(playlist.count)))) + " - "
    return prefix


def _embed_subs(opts: DownloadOptions) -> bool:
    return opts.subs and opts.sub_mode == "embed" and opts.kind != "audio"


def build_args(opts: DownloadOptions, settings: AppSettings, *, folder: str, archive: str,
               playlist: PlaylistContext | None = None) -> list[str]:
    """고른 값을 yt-dlp 명령줄 인자로. 후처리 순서 같은 세부 사항을 CLI 와 똑같이 맞추려고 인자를 거친다."""
    template = _template(settings)
    if playlist is not None:
        template = _playlist_prefix(opts, playlist) + template
    args = ["-P", folder, "-o", template]

    if settings.cookies == "browser":
        args += ["--cookies-from-browser", settings.cookie_browser]
    elif settings.cookies == "file":
        args += ["--cookies", settings.cookie_file]
    if settings.proxy:
        args += ["--proxy", settings.proxy]
    if settings.rate_limit:
        args += ["-r", settings.rate_limit]
    args += ["-R", str(settings.retries)]
    if settings.overwrite:
        args.append("--force-overwrites")

    if playlist is not None or opts.playlist == "single":
        args.append("--no-playlist")
    else:
        args.append("--yes-playlist")
        if opts.playlist_items:
            args += ["-I", opts.playlist_items.replace(" ", "")]
    if opts.skip_archived:
        args += ["--download-archive", archive]

    if opts.kind == "audio":
        quality = "0" if opts.audio_quality == "best" else f"{opts.audio_quality}K"
        args += ["-x", "--audio-format", opts.audio_format, "--audio-quality", quality]
    else:
        sort = []
        if opts.quality != "best":
            sort.append(f"res:{opts.quality}")
        if opts.codec in _VCODECS:
            sort.append(_VCODECS[opts.codec])
        if opts.container in _EXT_SORT:
            sort.append(_EXT_SORT[opts.container])
        args += ["-f", "bv*+ba/b"]
        if sort:
            args += ["-S", ",".join(sort)]
        args += ["--merge-output-format", opts.container]

    if opts.subs:
        if opts.auto_subs:
            args.append("--write-auto-subs")
        langs = opts.sub_langs.replace(" ", "")
        args += ["--sub-langs", "all,-live_chat" if langs == "all" else langs]
        # 음원 파일에는 자막을 넣을 수 없어 파일로 남긴다
        if _embed_subs(opts):
            # --write-subs 를 함께 주면 yt-dlp 가 넣은 뒤에도 자막 파일을 남긴다.
            # 일반 자막 받기는 to_ydl_opts 가 켠다.
            args.append("--embed-subs")
        else:
            args += ["--write-subs", "--convert-subs", opts.sub_format]

    if opts.embed_thumbnail:
        args.append("--embed-thumbnail")
    if opts.save_thumbnail:
        args += ["--write-thumbnail", "--convert-thumbnails", "jpg"]
    if opts.embed_metadata:
        args.append("--embed-metadata")
    if opts.embed_chapters:
        args.append("--embed-chapters")
    if opts.split_chapters:
        args.append("--split-chapters")
    if opts.section_start or opts.section_end:
        args += ["--download-sections", f"*{opts.section_start or '0'}-{opts.section_end or 'inf'}",
                 "--force-keyframes-at-cuts"]
    return args


def split_extra(text: str) -> list[str]:
    """고급 옵션 칸을 인자로 나눈다. 따옴표로 묶을 수 있고, Windows 경로의 백슬래시는 그대로 둔다."""
    parts: list[str] = []
    current: list[str] = []
    quote = ""
    started = False
    for ch in text:
        if quote:
            if ch == quote:
                quote = ""
            else:
                current.append(ch)
        elif ch in "\"'":
            quote = ch
            started = True
        elif ch.isspace():
            if started:
                parts.append("".join(current))
                current, started = [], False
        else:
            current.append(ch)
            started = True
    if started:
        parts.append("".join(current))
    return parts


def to_ydl_opts(opts: DownloadOptions, settings: AppSettings, *, folder: str, archive: str,
                ffmpeg_dir: str | None = None, playlist: PlaylistContext | None = None) -> dict:
    """yt-dlp 옵션 dict. 고급 옵션은 맨 뒤에 붙여 같은 옵션이면 사용자 값이 이긴다.
    진행률 훅은 다운로드 작업이 붙인다."""
    argv = ["--ignore-config", *build_args(opts, settings, folder=folder, archive=archive, playlist=playlist),
            *split_extra(settings.extra_args)]
    ydl_opts = yt_dlp.parse_options(argv).ydl_opts
    # 앱이 진행률을 그리고 재생목록을 직접 펼치려면 이 값들은 앱 것이어야 한다
    # 오류는 예외로 받아 화면에 이유를 보인다. 묻히면 받은 게 없는 채로 끝나 '이미 받음'으로 보인다.
    ydl_opts.update(quiet=True, no_warnings=True, noprogress=True, extract_flat="in_playlist",
                    ignoreerrors=False)
    if _embed_subs(opts):
        # --embed-subs 에 --write-auto-subs 가 붙으면 yt-dlp 가 일반 자막을 끈다
        ydl_opts["writesubtitles"] = True
    if ffmpeg_dir:
        ydl_opts["ffmpeg_location"] = ffmpeg_dir
    return ydl_opts


# ---------- 입력값 검사 ----------

def validate_options(opts: DownloadOptions) -> dict[str, str]:
    """패널 값 오류. 키는 칸 이름, 값은 화면 문구."""
    errors: dict[str, str] = {}
    if opts.kind == "video" and opts.container == "webm" and opts.codec == "h264":
        errors["codec"] = "WEBM 에는 H.264 를 넣을 수 없어요. 코덱이나 파일 형식을 바꿔 주세요"
    if opts.section_start or opts.section_end:
        start = parse_duration(opts.section_start) if opts.section_start else 0
        end = parse_duration(opts.section_end) if opts.section_end else float("inf")
        if start is None or end is None:
            errors["section"] = "시간은 1:30 처럼 써 주세요"
        elif start >= end:
            errors["section"] = "끝이 시작보다 뒤여야 해요"
    if opts.playlist == "all" and opts.playlist_items:
        try:
            list(PlaylistEntries.parse_playlist_items(opts.playlist_items.replace(" ", "")))
        except ValueError:
            errors["playlist_items"] = "범위는 1-10, 15 처럼 써 주세요"
    return errors


def _extra_args_error(text: str) -> str | None:
    try:
        parsed = yt_dlp.parse_options(["--ignore-config", *split_extra(text)])
    except (optparse.OptParseError, ValueError, SystemExit) as exc:
        last = (str(exc).strip().splitlines() or [""])[-1]
        return "고급 옵션을 읽지 못했어요: " + last.split("error: ", 1)[-1]
    if parsed.urls:
        return "고급 옵션에는 링크를 넣을 수 없어요"
    return None


def validate_settings(s: AppSettings) -> dict[str, str]:
    """설정 창 값 오류. 키는 칸 이름, 값은 화면 문구."""
    errors: dict[str, str] = {}
    if s.rate_limit and parse_bytes(s.rate_limit) is None:
        errors["rate_limit"] = "속도 제한은 5M, 500K 처럼 써 주세요"
    if s.cookies == "file" and not os.path.isfile(s.cookie_file):
        errors["cookie_file"] = "쿠키 파일을 찾을 수 없어요"
    if s.name_template == "custom" and not s.custom_template.strip():
        errors["custom_template"] = "파일 이름 규칙을 입력해 주세요"
    if s.extra_args.strip() and (msg := _extra_args_error(s.extra_args)):
        errors["extra_args"] = msg
    return errors


# 오류 키 → 그 오류를 일으키는 칸
_ERROR_CAUSES = {
    "rate_limit": {"rate_limit"},
    "cookie_file": {"cookies", "cookie_file"},
    "custom_template": {"name_template", "custom_template"},
    "extra_args": {"extra_args"},
}


def update_settings(s: AppSettings, **fields) -> tuple[AppSettings, dict[str, str]]:
    """칸 몇 개를 바꾼 설정과, 바꾼 칸 때문에 생긴 오류. 오류가 있으면 s 를 그대로 돌려준다.

    이미 저장된 다른 값이 지금은 틀려도(지워진 쿠키 파일 등) 바꾼 칸은 저장된다.
    """
    cand = dataclasses.replace(s, **fields)
    errors = {k: msg for k, msg in validate_settings(cand).items() if _ERROR_CAUSES[k] & fields.keys()}
    return (s, errors) if errors else (cand, {})
