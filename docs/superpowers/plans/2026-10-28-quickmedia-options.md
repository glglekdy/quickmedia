# QuickMedia yt-dlp 기능 확장 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 형식·자막·후처리·재생목록·로그인·이름 규칙·동시 다운로드를 패널과 ⚙ 설정 창에서 고르게 하고, 카드마다 사이트 아이콘과 이름을 보여 준다.

**Architecture:** 고른 값은 Qt 없는 `DownloadOptions`/`AppSettings`에 모이고, `build_args()`가 이를 yt-dlp 명령줄 인자로 바꿔 `yt_dlp.parse_options()`로 해석한다(고급 옵션은 맨 뒤에 붙인다). `DownloadJob`은 완성된 옵션 dict 만 받고, 재생목록이면 항목을 펼쳐 알린다. 동시 실행은 Qt 없는 `JobQueue`가, 사이트 표시는 `site.py`가 맡는다.

**Tech Stack:** Python 3, PySide6-Essentials, yt-dlp (로컬 2026.10.26), pytest.

**Spec:** `docs/superpowers/specs/2026-10-28-quickmedia-options-design.md` — 필드·값·인자·문구 표는 스펙이 기준이다. 이 계획은 스펙 표를 반복하지 않고 "스펙 표 X"로 가리킨다.

## Global Constraints

- `ytdl/options.py`, `ytdl/job_queue.py`는 PySide6 를 import 하지 않는다.
- 화면 문구는 스펙에 적힌 그대로 쓴다(오류 문구 표, 검사 문구 표, 안내 문구).
- 사이트 아이콘은 그 사이트의 `https://<host>/favicon.ico`에서만 받는다. 제3자 favicon 서비스 금지.
- yt-dlp 해석은 항상 `--ignore-config`로 시작한다.
- 테스트는 실제 레지스트리·네트워크를 쓰지 않는다. QSettings 는 `tmp_path`의 INI, Qt 는 `QT_QPA_PLATFORM=offscreen`.
- 기존 테스트 17개는 계속 통과해야 한다. 실행: `python -m pytest -q`
- 커밋 메시지는 한국어 한 줄 + 빈 줄 + 아래 두 줄:
  ```
  Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_01SwAsdfsDyXiog4PqzowpkY
  ```

## Review Focus

- "끝난 항목 지우기"는 완료·실패 카드뿐 아니라 재생목록 요약 카드, 건너뜀 카드, 취소 카드도 지워야 하고, 받는 중인 카드는 남겨야 한다 → Task 8 테스트.
- 재생목록 제목에 `%`, `/`, `:`, `?` 같은 글자가 있어도 파일 이름 접두어가 깨지지 않아야 한다 → Task 2 테스트.
- favicon 주소가 200 으로 HTML 페이지를 돌려줘도 대체 아이콘이 그대로 남아야 한다 → Task 5 테스트.
- 받는 중에 동시 다운로드 수를 줄여도 돌던 작업은 끝까지 가고, 늘리면 바로 다음 작업이 시작돼야 한다 → Task 4 테스트.
- 작업 여러 개가 도는 중에 창을 닫으면 모두 멈춰야 한다 → Task 8 테스트.

---

## 파일 구조

| 파일 | 상태 | 책임 |
|---|---|---|
| `ytdl/options.py` | 신규 | 데이터 클래스, 프리셋, dict 변환, 인자 만들기, 검사, 요약 문구 |
| `ytdl/job_queue.py` | 신규 | 동시 실행 한도 대기열 |
| `ytdl/site.py` | 신규 | 도메인·표시 이름 순수 함수, `SiteIcons` |
| `ytdl/downloader.py` | 수정 | `DownloadJob(url, ydl_opts)`, 재생목록 펼치기, 오류 문구 |
| `ytdl/options_panel.py` | 신규 | 펼치는 패널 |
| `ytdl/settings_dialog.py` | 신규 | ⚙ 설정 창 |
| `ytdl/main_window.py` | 수정 | 카드 사이트 줄, 패널·설정 연결, 대기열, 펼치기 |
| `ytdl/theme.py` | 수정 | 탭·콤보·체크박스·스핀박스·오류 문구·사이트 줄 스타일 |
| `tests/conftest.py` | 수정 | offscreen 환경, `qapp` fixture |
| `README.md` | 수정 | 기능 목록 |

---

### Task 1: 옵션 데이터 클래스, 프리셋, 저장 변환, 요약 문구

**Files:**
- Create: `ytdl/options.py`
- Test: `tests/test_options_model.py`

**Interfaces:**
- Produces:
  - `@dataclass class DownloadOptions` — 필드·기본값은 스펙 표 "패널 값". `sub_langs: str = "ko"`, `section_start: str = ""`, `section_end: str = ""`, `playlist_items: str = ""`, 나머지 켜기/끄기는 `bool = False`.
  - `@dataclass class AppSettings` — 스펙 표 "설정 창 값". `retries: int = 10`, `concurrency: int = 2`, `custom_template: str = "%(title)s.%(ext)s"`, `cookie_browser: str = "firefox"`, 나머지 문자열 `""`, `overwrite: bool = False`.
  - 선택지 상수: `QUALITIES`, `CONTAINERS`, `CODECS`, `AUDIO_FORMATS`, `AUDIO_QUALITIES`, `SUB_MODES`, `SUB_FORMATS`, `PLAYLIST_MODES`, `COOKIE_MODES`, `BROWSERS` (튜플, 스펙 표 순서), `NAME_TEMPLATES: dict[str, str]` (custom 제외 4개, 스펙 "파일 이름 템플릿" 표).
  - `PRESETS: dict[str, dict[str, str]]` 키 `"best"`, `"720p"`, `"mp3"` (스펙 "프리셋" 표). `PRESET_LABELS = {"best": "최고 화질", "720p": "720p", "mp3": "MP3 음원"}`.
  - `apply_preset(opts: DownloadOptions, key: str) -> DownloadOptions` (새 객체).
  - `matching_preset(opts: DownloadOptions) -> str | None`
  - `to_dict(obj) -> dict[str, object]`
  - `options_from_dict(d: dict, legacy_format: str | None = None) -> DownloadOptions`
  - `settings_from_dict(d: dict) -> AppSettings`
  - `describe(opts: DownloadOptions) -> str`
  - `changed_summary(opts: DownloadOptions) -> list[str]`

- [ ] **Step 1: 실패하는 테스트 작성** — `tests/test_options_model.py`

```python
from ytdl.options import (AppSettings, DownloadOptions, apply_preset, changed_summary, describe,
                          matching_preset, options_from_dict, settings_from_dict, to_dict)


def test_defaults():
    o = DownloadOptions()
    assert (o.kind, o.quality, o.container, o.codec) == ("video", "best", "mp4", "auto")
    assert (o.audio_format, o.audio_quality, o.sub_langs, o.playlist) == ("mp3", "192", "ko", "single")
    s = AppSettings()
    assert (s.cookies, s.cookie_browser, s.retries, s.concurrency, s.name_template) == ("none", "firefox", 10, 2, "title")


def test_presets_round_trip():
    assert matching_preset(DownloadOptions()) == "best"
    assert matching_preset(apply_preset(DownloadOptions(), "720p")) == "720p"
    mp3 = apply_preset(DownloadOptions(subs=True), "mp3")
    assert (mp3.kind, mp3.audio_format, mp3.audio_quality, mp3.subs) == ("audio", "mp3", "192", True)
    assert matching_preset(mp3) == "mp3"
    assert matching_preset(DownloadOptions(quality="1080")) is None
    assert matching_preset(DownloadOptions(kind="audio", audio_format="flac")) is None


def test_from_dict_coerces_registry_strings():
    o = options_from_dict({"subs": "true", "embed_chapters": "false", "quality": "1080", "nope": "x"})
    assert o.subs is True and o.embed_chapters is False and o.quality == "1080"
    s = settings_from_dict({"retries": "3", "concurrency": "4", "overwrite": "1"})
    assert (s.retries, s.concurrency, s.overwrite) == (3, 4, True)


def test_from_dict_rejects_bad_values():
    o = options_from_dict({"quality": "999", "container": "avi", "subs": "maybe"})
    assert (o.quality, o.container, o.subs) == ("best", "mp4", False)
    s = settings_from_dict({"concurrency": "9", "retries": "-1", "cookie_browser": "netscape"})
    assert (s.concurrency, s.retries, s.cookie_browser) == (2, 10, "firefox")


def test_legacy_format_only_when_no_saved_options():
    assert options_from_dict({}, legacy_format="mp3").kind == "audio"
    assert options_from_dict({}, legacy_format="720p").quality == "720"
    assert options_from_dict({"kind": "video"}, legacy_format="mp3").kind == "video"
    assert options_from_dict({}, legacy_format="weird") == DownloadOptions()


def test_to_dict_round_trip():
    o = DownloadOptions(kind="audio", subs=True, section_start="1:30")
    assert options_from_dict(to_dict(o)) == o


def test_describe():
    assert describe(DownloadOptions()) == "최고 화질 · MP4"
    assert describe(DownloadOptions(quality="1080", container="mkv")) == "1080p · MKV"
    assert describe(DownloadOptions(kind="audio")) == "MP3 · 192k"
    assert describe(DownloadOptions(kind="audio", audio_format="flac")) == "FLAC"
    assert describe(DownloadOptions(kind="audio", audio_format="opus", audio_quality="best")) == "OPUS · 최고 음질"
    assert describe(DownloadOptions(subs=True)) == "최고 화질 · MP4 · 자막"


def test_changed_summary():
    assert changed_summary(DownloadOptions()) == []
    assert changed_summary(apply_preset(DownloadOptions(), "mp3")) == []
    o = DownloadOptions(quality="1080", subs=True, save_thumbnail=True, embed_metadata=True,
                        split_chapters=True, section_end="3:00", playlist="all")
    assert changed_summary(o) == ["형식", "자막", "썸네일", "메타데이터", "챕터", "구간", "재생목록"]
```

- [ ] **Step 2: 실패 확인** — Run: `python -m pytest tests/test_options_model.py -q` / Expected: FAIL (`ModuleNotFoundError: ytdl.options`)

- [ ] **Step 3: 구현** — `ytdl/options.py`
  - `matching_preset`: 영상 프리셋은 `kind, quality, container, codec`만, mp3 는 `kind, audio_format, audio_quality`만 비교한다.
  - `_from_dict(cls, d)`: 필드 형식으로 되돌린다. bool 은 `True/False` 또는 문자열 `"true"/"false"/"1"/"0"`(대소문자 무시), 그 밖은 기본값. int 는 `int()` 후 범위(retries 0~50, concurrency 1~5) 밖이면 기본값. 선택지가 있는 str 필드(필드 이름 → 선택지 튜플 dict `_CHOICES`)는 목록 밖이면 기본값. 모르는 키는 버린다.
  - `options_from_dict`: `d`에 `"kind"`가 없고 `legacy_format in PRESETS`이면 프리셋을 적용한다.
  - `describe`: 영상 = `("최고 화질" if best else f"{q}p") + " · " + container.upper()`. 음원 = `fmt.upper()`에, flac·wav 가 아니면 `" · 최고 음질"` 또는 `f" · {q}k"`. `subs`면 `" · 자막"`.
  - `changed_summary` 순서와 조건: 형식(`matching_preset is None`), 자막(`subs`), 썸네일(`embed_thumbnail or save_thumbnail`), 메타데이터(`embed_metadata`), 챕터(`embed_chapters or split_chapters`), 구간(`section_start or section_end`), 재생목록(`playlist == "all" or playlist_items or number_prefix or playlist_folder or skip_archived`).

- [ ] **Step 4: 통과 확인** — Run: `python -m pytest tests/test_options_model.py -q` / Expected: 8 passed

- [ ] **Step 5: 커밋** — `git add ytdl/options.py tests/test_options_model.py` → "옵션 데이터 클래스와 프리셋, 저장 변환"

---

### Task 2: 옵션 → yt-dlp 인자·옵션 dict

**Files:**
- Modify: `ytdl/options.py`
- Test: `tests/test_options_args.py`

**Interfaces:**
- Consumes: Task 1 의 `DownloadOptions`, `AppSettings`, `NAME_TEMPLATES`.
- Produces:
  - `@dataclass(frozen=True) class PlaylistContext: title: str; index: int; count: int`
  - `build_args(opts, settings, *, folder: str, archive: str, playlist: PlaylistContext | None = None) -> list[str]`
  - `split_extra(text: str) -> list[str]`
  - `to_ydl_opts(opts, settings, *, folder: str, archive: str, ffmpeg_dir: str | None = None, playlist: PlaylistContext | None = None) -> dict`
    — `progress_hooks`는 넣지 않는다(작업이 넣는다). 넣는 고정값: `quiet=True`, `no_warnings=True`, `noprogress=True`, `extract_flat="in_playlist"`, `ffmpeg_dir`가 있으면 `ffmpeg_location`.

- [ ] **Step 1: 실패하는 테스트 작성** — `tests/test_options_args.py`

```python
import pytest

from ytdl.options import AppSettings, DownloadOptions, PlaylistContext, build_args, split_extra, to_ydl_opts

S = AppSettings()


def args(o=None, s=None, **kw):
    return build_args(o or DownloadOptions(), s or S, folder="D:/v", archive="A:/archive.txt", **kw)


def test_default_args_exact():
    assert args() == ["-P", "D:/v", "-o", "%(title)s.%(ext)s", "-R", "10", "--no-playlist",
                      "-f", "bv*+ba/b", "-S", "ext:mp4:m4a", "--merge-output-format", "mp4"]


@pytest.mark.parametrize("o, sort, merge", [
    (DownloadOptions(quality="720"), "res:720,ext:mp4:m4a", "mp4"),
    (DownloadOptions(quality="1080", codec="h264"), "res:1080,vcodec:h264,ext:mp4:m4a", "mp4"),
    (DownloadOptions(container="webm", codec="vp9"), "vcodec:vp9,ext:webm:webm", "webm"),
    (DownloadOptions(container="mkv", codec="av1"), "vcodec:av01", "mkv"),
])
def test_video_sort(o, sort, merge):
    a = args(o)
    assert a[a.index("-S") + 1] == sort and a[a.index("--merge-output-format") + 1] == merge


def test_mkv_auto_has_no_sort():
    assert "-S" not in args(DownloadOptions(container="mkv"))


def test_audio():
    a = args(DownloadOptions(kind="audio", audio_format="opus", audio_quality="best"))
    assert "-f" not in a and "--merge-output-format" not in a
    assert a[a.index("-x"):a.index("-x") + 5] == ["-x", "--audio-format", "opus", "--audio-quality", "0"]
    a = args(DownloadOptions(kind="audio", audio_quality="320"))
    assert a[a.index("--audio-quality") + 1] == "320K"


def test_subtitles():
    a = args(DownloadOptions(subs=True, sub_langs="all", auto_subs=True))
    assert a[a.index("--sub-langs") + 1] == "all,-live_chat"
    assert {"--write-subs", "--write-auto-subs", "--embed-subs"} <= set(a)
    a = args(DownloadOptions(subs=True, sub_mode="file", sub_format="ass"))
    assert a[a.index("--convert-subs") + 1] == "ass" and "--embed-subs" not in a
    a = args(DownloadOptions(kind="audio", subs=True))  # 음원에는 넣을 수 없어 파일로
    assert "--embed-subs" not in a and a[a.index("--convert-subs") + 1] == "srt"
    assert "--write-subs" not in args(DownloadOptions(sub_langs="en"))


def test_postprocessing_flags():
    a = args(DownloadOptions(embed_thumbnail=True, save_thumbnail=True, embed_metadata=True,
                             embed_chapters=True, split_chapters=True))
    for f in ("--embed-thumbnail", "--write-thumbnail", "--embed-metadata", "--embed-chapters", "--split-chapters"):
        assert f in a
    assert a[a.index("--convert-thumbnails") + 1] == "jpg"


@pytest.mark.parametrize("start, end, spec", [("1:30", "3:00", "*1:30-3:00"), ("", "3:00", "*0-3:00"), ("1:30", "", "*1:30-inf")])
def test_sections(start, end, spec):
    a = args(DownloadOptions(section_start=start, section_end=end))
    assert a[a.index("--download-sections") + 1] == spec and "--force-keyframes-at-cuts" in a


def test_playlist_and_archive():
    a = args(DownloadOptions(playlist="all", playlist_items="1-10,15", skip_archived=True))
    assert "--yes-playlist" in a and "--no-playlist" not in a
    assert a[a.index("-I") + 1] == "1-10,15"
    assert a[a.index("--download-archive") + 1] == "A:/archive.txt"
    assert "-I" not in args(DownloadOptions(playlist="single", playlist_items="1-3"))


def test_network_and_files():
    s = AppSettings(cookies="browser", cookie_browser="edge", proxy="http://p:1", rate_limit="5M", retries=3,
                    name_template="date_title", overwrite=True)
    a = args(s=s)
    assert a[a.index("--cookies-from-browser") + 1] == "edge"
    assert a[a.index("--proxy") + 1] == "http://p:1" and a[a.index("-r") + 1] == "5M"
    assert a[a.index("-R") + 1] == "3" and "--force-overwrites" in a
    assert a[a.index("-o") + 1] == "%(upload_date>%Y-%m-%d)s %(title)s.%(ext)s"
    a = args(s=AppSettings(cookies="file", cookie_file="C:/c.txt", name_template="custom", custom_template="%(id)s.%(ext)s"))
    assert a[a.index("--cookies") + 1] == "C:/c.txt" and a[a.index("-o") + 1] == "%(id)s.%(ext)s"


def test_playlist_prefix_sanitizes_title():
    o = DownloadOptions(playlist="all", number_prefix=True, playlist_folder=True)
    a = args(o, playlist=PlaylistContext(title="100% 노래/모음: 2?", index=7, count=120))
    tmpl = a[a.index("-o") + 1]
    folder, name = tmpl.split("/", 1)
    assert "%(" not in folder and "%%" in folder and "/" not in folder and ":" not in folder and "?" not in folder
    assert name == "007 - %(title)s.%(ext)s"
    assert "--no-playlist" in a and "--yes-playlist" not in a and "-I" not in a


def test_playlist_prefix_min_width_and_off():
    a = args(DownloadOptions(number_prefix=True), playlist=PlaylistContext("p", 3, 5))
    assert a[a.index("-o") + 1] == "03 - %(title)s.%(ext)s"
    a = args(DownloadOptions(), playlist=PlaylistContext("p", 3, 5))
    assert a[a.index("-o") + 1] == "%(title)s.%(ext)s"


def test_split_extra_keeps_windows_paths():
    assert split_extra('--limit-rate 1M  -o "C:\\내 폴더\\%(id)s.%(ext)s" --x \'a b\'') == [
        "--limit-rate", "1M", "-o", "C:\\내 폴더\\%(id)s.%(ext)s", "--x", "a b"]
    assert split_extra("   ") == []


def test_to_ydl_opts_fixed_values_and_extra_wins():
    s = AppSettings(rate_limit="5M", extra_args="--limit-rate 1M --flat-playlist --no-quiet")
    o = to_ydl_opts(DownloadOptions(), s, folder="D:/v", archive="A", ffmpeg_dir="F:/ff")
    assert o["ratelimit"] == 1024 * 1024
    assert o["paths"] == {"home": "D:/v"} and o["format_sort"] == ["ext:mp4:m4a"]
    assert (o["quiet"], o["noprogress"], o["extract_flat"], o["ffmpeg_location"]) == (True, True, "in_playlist", "F:/ff")
    assert "progress_hooks" not in o


def test_to_ydl_opts_audio_postprocessors():
    o = to_ydl_opts(DownloadOptions(kind="audio", embed_thumbnail=True), S, folder="D", archive="A")
    keys = [p["key"] for p in o["postprocessors"]]
    assert keys.index("FFmpegExtractAudio") < keys.index("EmbedThumbnail")
    assert "ffmpeg_location" not in o
```

- [ ] **Step 2: 실패 확인** — Run: `python -m pytest tests/test_options_args.py -q` / Expected: FAIL (`ImportError: PlaylistContext`)

- [ ] **Step 3: 구현** — `ytdl/options.py`
  - `build_args` 인자 순서: `-P folder`, `-o 템플릿`, (`--cookies-from-browser`/`--cookies`), (`--proxy`), (`-r`), `-R n`, (`--force-overwrites`), 재생목록(`--no-playlist` 또는 `--yes-playlist` + `-I` + `--download-archive`), 형식(영상: `-f bv*+ba/b`, `-S`, `--merge-output-format` / 음원: `-x --audio-format --audio-quality`), 자막, 썸네일·메타데이터·챕터, 구간. 테스트의 "정확한 목록"은 기본값일 때만 순서를 고정한다.
  - `-S` 조각: `res:<h>`(숫자일 때) → `vcodec:{h264|vp9|av01}`(auto 아니면) → `ext:mp4:m4a`(mp4) / `ext:webm:webm`(webm). 빈 목록이면 `-S`를 넣지 않는다.
  - `--download-archive`는 `skip_archived`일 때만. `-I`는 `playlist == "all"`이고 범위가 있고 `playlist`(컨텍스트)가 None 일 때만.
  - 재생목록 컨텍스트가 있으면 항상 `--no-playlist`. 접두어: 폴더 = `yt_dlp.utils.sanitize_filename(title).replace("%", "%%") + "/"` (빈 결과면 `"playlist/"`), 번호 = `str(index).zfill(max(2, len(str(count)))) + " - "`.
  - `split_extra`: 글자를 하나씩 읽는다. 공백은 바깥에서만 구분자, `"`·`'`는 같은 따옴표가 닫힐 때까지 한 덩어리(따옴표는 빼고), 백슬래시는 그냥 글자.
  - `to_ydl_opts`: `yt_dlp.parse_options(["--ignore-config", *build_args(...), *split_extra(settings.extra_args)]).ydl_opts`에 고정값을 덮어쓴다.

- [ ] **Step 4: 통과 확인** — Run: `python -m pytest tests/test_options_args.py -q` / Expected: all passed

- [ ] **Step 5: 커밋** — "옵션을 yt-dlp 인자와 옵션으로 바꾸기"

---

### Task 3: 입력값 검사

**Files:**
- Modify: `ytdl/options.py`
- Test: `tests/test_options_validate.py`

**Interfaces:**
- Consumes: Task 2 의 `split_extra`.
- Produces:
  - `validate_options(opts: DownloadOptions) -> dict[str, str]` — 키: `"section"`, `"playlist_items"` (범위는 `playlist == "all"`일 때만 검사).
  - `validate_settings(s: AppSettings) -> dict[str, str]` — 키: `"rate_limit"`, `"cookie_file"`(cookies=="file"일 때만), `"custom_template"`(custom 일 때만), `"extra_args"`.
  - 값은 스펙 "입력값 검사" 표의 문구 그대로. 오류가 없으면 빈 dict.

- [ ] **Step 1: 실패하는 테스트 작성** — `tests/test_options_validate.py`

```python
from ytdl.options import AppSettings, DownloadOptions, validate_options, validate_settings


def test_valid_defaults():
    assert validate_options(DownloadOptions()) == {} and validate_settings(AppSettings()) == {}


def test_sections():
    assert validate_options(DownloadOptions(section_start="1:30", section_end="3:00")) == {}
    assert validate_options(DownloadOptions(section_start="abc"))["section"] == "시간은 1:30 처럼 써 주세요"
    assert validate_options(DownloadOptions(section_start="3:00", section_end="1:00"))["section"] == "끝이 시작보다 뒤여야 해요"


def test_playlist_items_only_when_all():
    assert validate_options(DownloadOptions(playlist="single", playlist_items="x")) == {}
    assert validate_options(DownloadOptions(playlist="all", playlist_items="1-10, 15")) == {}
    assert validate_options(DownloadOptions(playlist="all", playlist_items="x"))["playlist_items"] == "범위는 1-10, 15 처럼 써 주세요"


def test_rate_limit():
    assert validate_settings(AppSettings(rate_limit="500K")) == {}
    assert validate_settings(AppSettings(rate_limit="fast"))["rate_limit"] == "속도 제한은 5M, 500K 처럼 써 주세요"


def test_cookie_file(tmp_path):
    assert validate_settings(AppSettings(cookie_file="Z:/none.txt")) == {}  # 쿠키 모드가 file 이 아닐 때는 보지 않음
    assert validate_settings(AppSettings(cookies="file", cookie_file=str(tmp_path / "no.txt")))["cookie_file"] == "쿠키 파일을 찾을 수 없어요"
    f = tmp_path / "c.txt"; f.write_text("x")
    assert validate_settings(AppSettings(cookies="file", cookie_file=str(f))) == {}


def test_custom_template():
    assert validate_settings(AppSettings(name_template="custom", custom_template=" "))["custom_template"] == "파일 이름 규칙을 입력해 주세요"


def test_extra_args():
    assert validate_settings(AppSettings(extra_args="--limit-rate 1M --no-mtime")) == {}
    msg = validate_settings(AppSettings(extra_args="--nope"))["extra_args"]
    assert msg.startswith("고급 옵션을 읽지 못했어요: ") and "--nope" in msg and "Usage" not in msg
    assert validate_settings(AppSettings(extra_args="https://x.com/v"))["extra_args"] == "고급 옵션에는 링크를 넣을 수 없어요"
```

- [ ] **Step 2: 실패 확인** — Run: `python -m pytest tests/test_options_validate.py -q` / Expected: FAIL (ImportError)

- [ ] **Step 3: 구현** — 스펙 표의 검사 함수를 쓴다: `parse_duration`, `parse_bytes`, `PlaylistEntries.parse_playlist_items`(제너레이터라 `list()`로 끝까지 돌려야 오류가 난다), `parse_options`. 고급 옵션은 `(optparse.OptParseError, ValueError, SystemExit)`를 잡고, 메시지 마지막 줄에서 `"error: "` 뒤만 남긴다. 해석 결과의 `urls`가 비어 있지 않으면 링크 오류.

- [ ] **Step 4: 통과 확인** — Run: `python -m pytest tests/test_options_validate.py -q` / Expected: 7 passed

- [ ] **Step 5: 커밋** — "패널과 설정 값 검사"

---

### Task 4: 동시 실행 대기열

**Files:**
- Create: `ytdl/job_queue.py`
- Test: `tests/test_job_queue.py`

**Interfaces:**
- Produces: `class JobQueue(Generic[T])`
  - `__init__(self, limit: int, start: Callable[[T], None])`
  - `add(job: T) -> None`, `done(job: T) -> None` (모르는 작업이면 무시), `set_limit(n: int) -> None`
  - 속성 `running: list[T]`, `pending: list[T]` (사본), `limit: int`

- [ ] **Step 1: 실패하는 테스트 작성** — `tests/test_job_queue.py`

```python
from ytdl.job_queue import JobQueue


def make(limit):
    started = []
    return JobQueue(limit, started.append), started


def test_starts_up_to_limit_in_order():
    q, started = make(2)
    for j in "abcd":
        q.add(j)
    assert started == ["a", "b"] and q.pending == ["c", "d"]


def test_done_starts_next():
    q, started = make(1)
    q.add("a"); q.add("b")
    q.done("a")
    assert started == ["a", "b"] and q.running == ["b"] and q.pending == []
    q.done("zzz")  # 모르는 작업
    assert q.running == ["b"]


def test_raise_limit_starts_immediately():
    q, started = make(1)
    for j in "abc":
        q.add(j)
    q.set_limit(3)
    assert started == ["a", "b", "c"]


def test_lower_limit_keeps_running_jobs():
    q, started = make(3)
    for j in "abcd":
        q.add(j)
    q.set_limit(1)
    assert q.running == ["a", "b", "c"]
    q.done("a"); q.done("b")
    assert started == ["a", "b", "c"]  # 아직 한도(1)를 넘는다
    q.done("c")
    assert started == ["a", "b", "c", "d"]
```

- [ ] **Step 2: 실패 확인** — Run: `python -m pytest tests/test_job_queue.py -q` / Expected: FAIL (ModuleNotFoundError)
- [ ] **Step 3: 구현** — `_fill()`: `while pending and len(running) < limit`: 앞에서 꺼내 running 에 넣고 `start(job)`. `add`·`done`·`set_limit`이 끝에 `_fill()`을 부른다. `set_limit`은 1 미만이면 1.
- [ ] **Step 4: 통과 확인** — Run: `python -m pytest tests/test_job_queue.py -q` / Expected: 4 passed
- [ ] **Step 5: 커밋** — "동시 다운로드 대기열"

---

### Task 5: 사이트 이름과 아이콘

**Files:**
- Create: `ytdl/site.py`
- Modify: `tests/conftest.py` (offscreen + `qapp` fixture)
- Test: `tests/test_site.py`

**Interfaces:**
- Consumes: `ytdl.theme.color("accent")` (대체 아이콘 색).
- Produces:
  - `domain_of(url: str) -> str`, `display_name(extractor_key: str, host: str) -> str`, `icon_url(host: str) -> str`, `letter_of(name: str) -> str`
  - `fetch_bytes(url: str) -> bytes | None` — `User-Agent: QuickMedia/<버전>`, 시간 제한 5초, 256KB 넘거나 어떤 예외든 None.
  - `class SiteIcons(QObject)`: `ready = Signal(str, QPixmap)`; `__init__(self, cache_dir: Path, fetch: Callable[[str], bytes | None] = fetch_bytes, max_age_days: int = 30)`; `icon(host: str) -> QPixmap`; `fallback(host: str) -> QPixmap`.
  - 아이콘 크기: 화면 16px. 캐시·픽스맵은 32px 로 만들고 `setDevicePixelRatio(2)`로 16px 에 그린다(고배율 화면에서 선명하게).
- `tests/conftest.py`에 추가: 맨 위 `os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")`, 그리고

```python
@pytest.fixture(scope="session")
def qapp():
    from PySide6.QtWidgets import QApplication
    return QApplication.instance() or QApplication([])
```

- [ ] **Step 1: 실패하는 테스트 작성** — `tests/test_site.py`

```python
import os
import time

from PySide6.QtCore import QBuffer, QByteArray, QIODevice, QThreadPool
from PySide6.QtGui import QColor, QImage
from PySide6.QtWidgets import QApplication

from ytdl.site import SiteIcons, display_name, domain_of, icon_url, letter_of


def test_domain_of():
    assert domain_of("https://www.youtube.com/watch?v=x") == "youtube.com"
    assert domain_of("https://m.youtube.com/watch?v=x") == "youtube.com"
    assert domain_of("https://tv.naver.com/v/1") == "tv.naver.com"
    assert domain_of("not a url") == ""


def test_display_name():
    assert display_name("Youtube", "youtube.com") == "YouTube"
    assert display_name("YoutubeTab", "youtube.com") == "YouTube"
    assert display_name("Twitch:vod", "twitch.tv") == "Twitch"
    assert display_name("Twitter", "x.com") == "X"
    assert display_name("SoopLive", "sooplive.co.kr") == "SOOP"
    assert display_name("Naver", "tv.naver.com") == "네이버 TV"
    assert display_name("Generic", "example.com") == "example.com"
    assert display_name("", "example.com") == "example.com"
    assert display_name("Rumble", "rumble.com") == "Rumble"


def test_icon_url_and_letter():
    assert icon_url("youtube.com") == "https://youtube.com/favicon.ico"
    assert letter_of("youtube.com") == "Y" and letter_of("네이버 TV") == "네" and letter_of("") == "?"


def _png():
    img = QImage(48, 48, QImage.Format.Format_ARGB32); img.fill(QColor("#00ff00"))
    ba = QByteArray(); buf = QBuffer(ba); buf.open(QIODevice.OpenModeFlag.WriteOnly); img.save(buf, "PNG")
    return bytes(ba)


def _settle():
    QThreadPool.globalInstance().waitForDone(3000)
    QApplication.processEvents()


def test_fetches_caches_and_signals(qapp, tmp_path):
    calls, got = [], []
    icons = SiteIcons(tmp_path, fetch=lambda u: calls.append(u) or _png())
    icons.ready.connect(lambda h, p: got.append(h))
    first = icons.icon("youtube.com")              # 처음엔 대체 아이콘
    assert first.toImage().pixelColor(16, 16) != QColor("#00ff00")
    _settle()
    assert got == ["youtube.com"] and calls == ["https://youtube.com/favicon.ico"]
    assert (tmp_path / "youtube.com.png").exists()
    assert icons.icon("youtube.com").toImage().pixelColor(16, 16) == QColor("#00ff00")
    again = SiteIcons(tmp_path, fetch=lambda u: calls.append(u))  # 디스크 캐시
    again.icon("youtube.com"); _settle()
    assert len(calls) == 1


def test_stale_cache_refetches(qapp, tmp_path):
    (tmp_path / "a.com.png").write_bytes(_png())
    old = time.time() - 31 * 86400
    os.utime(tmp_path / "a.com.png", (old, old))
    calls = []
    SiteIcons(tmp_path, fetch=lambda u: calls.append(u) or _png()).icon("a.com"); _settle()
    assert calls == ["https://a.com/favicon.ico"]


def test_html_response_keeps_fallback_and_is_not_retried(qapp, tmp_path):
    calls, got = [], []
    icons = SiteIcons(tmp_path, fetch=lambda u: calls.append(u) or b"<html>not found</html>")
    icons.ready.connect(lambda h, p: got.append(h))
    icons.icon("b.com"); _settle()
    icons.icon("b.com"); _settle()
    assert got == [] and len(calls) == 1 and not (tmp_path / "b.com.png").exists()
```

- [ ] **Step 2: 실패 확인** — Run: `python -m pytest tests/test_site.py -q` / Expected: FAIL (ModuleNotFoundError)

- [ ] **Step 3: 구현** — `ytdl/site.py`
  - 표시 이름 표는 스펙 "사이트 표시" 표. 키의 `:` 앞부분을 소문자로 바꿔 접두어(`startswith`)로 찾는다(`youtubetab` → youtube).
  - `domain_of`: `urllib.parse.urlsplit(url).hostname`, 없으면 `""`. 앞의 `www.`·`m.`을 뗀다.
  - `SiteIcons.icon`: 메모리 캐시 → 디스크(`<cache_dir>/<host>.png`, 수정 시각이 `max_age_days`보다 오래되면 무시) → 대체 아이콘을 돌려주고, 이번 실행에 받는 중이거나 실패한 host 가 아니면 받기 시작. 빈 host 는 받지 않는다.
  - 받기: `QRunnable` + 신호용 `QObject`(메인 스레드에서 `SiteIcons`가 만들어 둔 것 하나를 넘긴다. 그래야 신호가 메인 스레드로 큐잉된다). 작업 스레드에서 `fetch(icon_url(host))` → `QImage.fromData` → null 이면 실패. 성공하면 32px 로 줄인(`Qt.KeepAspectRatio, Qt.SmoothTransformation`) 이미지를 메인 스레드로 보낸다. 메인 스레드에서 `QPixmap.fromImage`, `setDevicePixelRatio(2)`, png 저장(`cache_dir.mkdir(parents=True, exist_ok=True)`), 메모리 캐시, `ready.emit`.
  - `fallback(host)`: 32px 투명 픽스맵에 강조색 원, 가운데 흰 굵은 `letter_of(host)`, `setDevicePixelRatio(2)`.

- [ ] **Step 4: 통과 확인** — Run: `python -m pytest -q` / Expected: 전체 통과

- [ ] **Step 5: 커밋** — "사이트 표시 이름과 아이콘 캐시"

---

### Task 6: 다운로드 작업 — 옵션 dict, 재생목록 펼치기, 오류 문구

**Files:**
- Modify: `ytdl/downloader.py`
- Test: `tests/test_downloader.py`

**Interfaces:**
- Consumes: 없음 (옵션 dict 는 Task 2 의 `to_ydl_opts` 결과를 Task 8 이 넘긴다).
- Produces:
  - `FFMPEG_DIR: str | None` — 묶인 `ffmpeg.exe`가 있으면 `str(BASE_DIR)`, 없으면 None. `FORMATS`는 지운다.
  - `@dataclass class PlaylistEntry: url: str; title: str; index: int`
  - `@dataclass class PlaylistResult: title: str; site_key: str; webpage_url: str; entries: list[PlaylistEntry]; skipped: int`
  - `playlist_entries(info: dict, is_archived: Callable[[dict], bool] = lambda e: False) -> tuple[list[PlaylistEntry], int]`
  - `class DownloadJob(QThread)`: `__init__(self, url: str, ydl_opts: dict)`. 기존 신호 `titled`, `progress`, `finished_ok`, `failed`는 그대로. 추가: `site_found = Signal(str, str)` (extractor_key, webpage_url), `expanded = Signal(object)` (PlaylistResult), `skipped = Signal()`.
  - `friendly_error(e: Exception) -> str` — 스펙 "오류 문구" 표.

- [ ] **Step 1: 실패하는 테스트 작성** — `tests/test_downloader.py`

```python
import pytest

from ytdl.downloader import PlaylistEntry, friendly_error, playlist_entries


@pytest.mark.parametrize("msg, expected", [
    ("ERROR: Unsupported URL: https://x", "지원하지 않는 링크예요"),
    ("ERROR: could not find chrome cookies database in ...", "브라우저 쿠키를 읽지 못했어요. 브라우저를 닫고 다시 시도하거나 cookies.txt 를 써 주세요"),
    ("ERROR: Could not copy Chrome cookie database. See ...", "브라우저 쿠키를 읽지 못했어요. 브라우저를 닫고 다시 시도하거나 cookies.txt 를 써 주세요"),
    ("ERROR: Failed to decrypt with DPAPI", "브라우저 쿠키를 읽지 못했어요. 브라우저를 닫고 다시 시도하거나 cookies.txt 를 써 주세요"),
    ("ERROR: Sign in to confirm your age", "로그인이 필요한 영상이에요. ⚙ 설정에서 쿠키를 켜 보세요"),
    ("ERROR: Join this channel to get access to members-only content", "로그인이 필요한 영상이에요. ⚙ 설정에서 쿠키를 켜 보세요"),
    ("ERROR: Video unavailable", "삭제되었거나 볼 수 없는 영상이에요"),
    ("ERROR: Requested format is not available", "고른 화질·형식이 없는 영상이에요. 세부 옵션을 바꿔 보세요"),
    ("ERROR: Postprocessing: ffprobe and ffmpeg not found", "변환 도구(ffmpeg)를 찾지 못했어요"),
    ("ERROR: Unable to connect to proxy", "인터넷 연결이나 프록시 설정을 확인해 주세요"),
    ("ERROR: [Errno 11001] getaddrinfo failed", "인터넷 연결이나 프록시 설정을 확인해 주세요"),
    ("ERROR: something odd", "something odd"),
])
def test_friendly_error(msg, expected):
    assert friendly_error(Exception(msg)) == expected


def test_friendly_error_never_mentions_youtube():
    assert "유튜브" not in friendly_error(Exception("ERROR: is not a valid URL"))


def test_playlist_entries():
    info = {"entries": [
        {"url": "https://y/1", "title": "one", "playlist_index": 4, "id": "1"},
        {"title": "no url"},
        None,
        {"webpage_url": "https://y/3", "id": "3"},
        {"url": "https://y/4", "title": "done", "id": "4"},
    ]}
    entries, skipped = playlist_entries(info, is_archived=lambda e: e.get("id") == "4")
    assert entries == [PlaylistEntry("https://y/1", "one", 4), PlaylistEntry("https://y/3", "https://y/3", 4)]
    assert skipped == 1


def test_playlist_entries_empty():
    assert playlist_entries({}) == ([], 0)
```

순번 규칙: 항목에 `playlist_index`가 없으면 "원래 `entries` 목록에서의 1부터 센 위치"를 쓴다(위 예에서 `https://y/3`은 네 번째 → 4).

- [ ] **Step 2: 실패 확인** — Run: `python -m pytest tests/test_downloader.py -q` / Expected: FAIL (ImportError)

- [ ] **Step 3: 구현**
  - `friendly_error`: 스펙 표를 위에서부터 대소문자 무시로 맞춰 본다(쿠키 줄을 로그인 줄보다 먼저). ffmpeg 줄은 `ffmpeg` 또는 `ffprobe`가 있고 `not found`나 `not installed`도 있을 때.
  - `playlist_entries`: `info.get("entries") or []`를 돌며 dict 가 아니거나 `url`/`webpage_url`이 없으면 건너뛴다. `is_archived(entry)`가 True 면 `skipped += 1`.
  - `DownloadJob.run()`:
    1. `opts = {**self.ydl_opts, "progress_hooks": [self._hook]}`
    2. `info = ydl.extract_info(url, download=False)`. None 이면 `skipped.emit()`.
    3. `site_found.emit(info.get("extractor_key") or "", info.get("webpage_url") or url)`
    4. `info.get("_type") == "playlist"`이면 `playlist_entries(info, ydl.in_download_archive if self.ydl_opts.get("download_archive") else lambda e: False)` → `expanded.emit(PlaylistResult(title=info.get("title") or info.get("id") or url, site_key=..., webpage_url=..., entries=..., skipped=...))`. 끝.
    5. 아니면 기존 흐름(`titled`, `_parts`, `process_ie_result(download=True)`). 결과가 None 이거나 `requested_downloads`가 비면 `skipped.emit()`, 아니면 `finished_ok.emit(filepath or self._file)`.
  - `__init__`의 기존 진행률 필드는 그대로 둔다.

- [ ] **Step 4: 통과 확인** — Run: `python -m pytest -q` / Expected: 전체 통과

- [ ] **Step 5: 손으로 확인 (네트워크)** — 아래 스크립트를 `$CLAUDE_JOB_DIR/tmp/probe.py`로 저장해 `python -I`로 돌리고, 재생목록 항목 수와 첫 항목 index 가 찍히는지 본다.

```python
import sys; sys.path.insert(0, r"C:\Users\USER\orca\youtube-downloader")
from PySide6.QtCore import QCoreApplication
from ytdl.downloader import DownloadJob
from ytdl.options import AppSettings, DownloadOptions, to_ydl_opts
app = QCoreApplication([])
o = to_ydl_opts(DownloadOptions(playlist="all", playlist_items="1-3"), AppSettings(), folder=".", archive="a.txt")
job = DownloadJob("https://www.youtube.com/playlist?list=PLzMcBGfZo4-lp3jAExUCewBfMx3UZFkh5", o)
job.expanded.connect(lambda r: print(r.title, len(r.entries), r.entries[0].index))
job.site_found.connect(lambda k, u: print("site", k)); job.failed.connect(print)
job.finished.connect(app.quit); job.start(); app.exec()
```
Expected: `site YoutubeTab`, 그리고 `<재생목록 이름> 3 1`. 재생목록이 사라졌으면 아무 공개 재생목록 주소로 바꾼다.

- [ ] **Step 6: 커밋** — "다운로드 작업이 옵션을 받고 재생목록을 펼침"

---

### Task 7: 세부 옵션 패널과 ⚙ 설정 창

**Files:**
- Create: `ytdl/options_panel.py`, `ytdl/settings_dialog.py`
- Modify: `ytdl/theme.py`
- Test: `tests/test_option_widgets.py`

**Interfaces:**
- Consumes: Task 1 의 데이터 클래스·선택지 상수, Task 3 의 `validate_settings`.
- Produces:
  - `class OptionsPanel(QWidget)`: `changed = Signal()`; `set_options(o: DownloadOptions) -> None` (신호 내지 않음); `options() -> DownloadOptions`; 탭 위젯 `self.tabs` (탭 이름 "형식", "자막", "후처리", "재생목록"); 영상 칸 묶음 `self.video_box`, 음원 칸 묶음 `self.audio_box`.
  - `class SettingsDialog(QDialog)`: `__init__(self, settings: AppSettings, parent=None)`; `value() -> AppSettings`; 저장 버튼 `self.save_btn`; 필드별 오류 라벨 `self.errors: dict[str, QLabel]` (키는 `validate_settings` 키). 저장을 누르면 검사 → 오류가 있으면 라벨에 문구를 보이고 창을 닫지 않음 → 없으면 `accept()`.

- [ ] **Step 1: 실패하는 테스트 작성** — `tests/test_option_widgets.py`

```python
from ytdl.options import AppSettings, DownloadOptions
from ytdl.options_panel import OptionsPanel
from ytdl.settings_dialog import SettingsDialog

NON_DEFAULT = DownloadOptions(kind="audio", quality="1080", container="mkv", codec="av1", audio_format="flac",
                              audio_quality="320", subs=True, sub_langs="ja,en", auto_subs=True, sub_mode="file",
                              sub_format="vtt", embed_thumbnail=True, save_thumbnail=True, embed_metadata=True,
                              embed_chapters=True, split_chapters=True, section_start="0:10", section_end="0:20",
                              playlist="all", playlist_items="1-3", number_prefix=True, playlist_folder=True,
                              skip_archived=True)


def test_panel_round_trip(qapp):
    p = OptionsPanel()
    assert p.options() == DownloadOptions()
    p.set_options(NON_DEFAULT)
    assert p.options() == NON_DEFAULT
    assert [p.tabs.tabText(i) for i in range(p.tabs.count())] == ["형식", "자막", "후처리", "재생목록"]


def test_panel_shows_only_relevant_format_box(qapp):
    p = OptionsPanel()
    assert not p.video_box.isHidden() and p.audio_box.isHidden()
    p.set_options(DownloadOptions(kind="audio"))
    assert p.video_box.isHidden() and not p.audio_box.isHidden()


def test_panel_emits_changed_on_user_edit_only(qapp):
    p = OptionsPanel(); hits = []
    p.changed.connect(lambda: hits.append(1))
    p.set_options(NON_DEFAULT)
    assert hits == []
    p.set_options(DownloadOptions())
    from PySide6.QtWidgets import QCheckBox
    box = next(c for c in p.findChildren(QCheckBox) if c.text() == "자막 받기")
    box.click()  # 사용자가 누른 것처럼
    assert hits and p.options().subs is True


def test_settings_dialog_round_trip(qapp):
    s = AppSettings(cookies="browser", cookie_browser="edge", proxy="http://p:1", rate_limit="5M", retries=3,
                    name_template="custom", custom_template="%(id)s.%(ext)s", overwrite=True, concurrency=4,
                    extra_args="--no-mtime")
    d = SettingsDialog(s)
    assert d.value() == s


def test_settings_dialog_blocks_invalid(qapp):
    d = SettingsDialog(AppSettings(rate_limit="fast", extra_args="--nope"))
    d.save_btn.click()
    assert d.result() != d.DialogCode.Accepted
    assert d.errors["rate_limit"].text() == "속도 제한은 5M, 500K 처럼 써 주세요" and not d.errors["rate_limit"].isHidden()
    assert not d.errors["extra_args"].isHidden() and d.errors["cookie_file"].isHidden()


def test_settings_dialog_accepts_valid(qapp):
    d = SettingsDialog(AppSettings())
    d.save_btn.click()
    assert d.result() == d.DialogCode.Accepted
```

- [ ] **Step 2: 실패 확인** — Run: `python -m pytest tests/test_option_widgets.py -q` / Expected: FAIL (ModuleNotFoundError)

- [ ] **Step 3: 패널 구현** — `ytdl/options_panel.py`
  - 콤보박스 항목은 화면 글자와 값(`itemData`)을 따로 둔다. 화면 글자: 화질 `최고 / 2160p / 1440p / 1080p / 720p / 480p / 360p`, 코덱 `자동 / H.264 (호환성 좋음) / VP9 / AV1`, 음질 `최고 / 320k / 192k / 128k`, 컨테이너·음원 형식은 대문자, 자막 형식은 대문자.
  - 형식 탭: 맨 위 "종류" 라디오 2개(`영상`, `음원만`) → 아래 `video_box`(화질·파일 형식·코덱) / `audio_box`(형식·음질). flac·wav 면 음질 콤보 비활성.
  - 자막 탭: 체크박스 "자막 받기", 언어 콤보(`한국어`=ko, `영어`=en, `모든 언어`=all, `직접 입력`) + 직접 입력일 때만 보이는 QLineEdit(안내 `ko,en`), 체크 "자동 생성 자막 포함", 저장 방식 라디오(`영상 파일 안에 넣기`, `별도 파일`) + 자막 형식 콤보(별도 파일일 때만 활성). "자막 받기"가 꺼지면 나머지 비활성. `sub_langs` 값이 ko/en/all 이 아니면 직접 입력으로 채운다.
  - 후처리 탭 체크박스 문구: `썸네일을 파일에 넣기`, `썸네일을 이미지 파일로도 저장`, `제목·업로더 같은 메타데이터 넣기`, `챕터 정보 넣기`, `챕터별로 파일 나누기`. 구간: `구간만 받기` 라벨 + 시작·끝 QLineEdit(안내 `1:30`, `3:00`).
  - 재생목록 탭: 라디오 `이 영상만`, `재생목록 전체`; 범위 QLineEdit(안내 `1-10, 15`); 체크 `파일 이름 앞에 번호 붙이기`, `재생목록 이름으로 폴더 만들기`, `이미 받은 영상 건너뛰기`. "이 영상만"이면 범위·번호·폴더 비활성(건너뛰기는 항상 활성).
  - 모든 입력의 사용자 변경 시그널(`toggled`, `currentIndexChanged`, `textEdited`)을 하나로 모아 활성/숨김 갱신 후 `changed.emit()`. `set_options`는 `_loading` 플래그로 신호를 막는다.

- [ ] **Step 4: 설정 창 구현** — `ytdl/settings_dialog.py`
  - 제목 `설정`. 세로로 구역 제목 4개: `로그인·네트워크`, `파일`, `다운로드`, `고급`.
  - 쿠키 라디오 `사용 안 함` / `브라우저에서 가져오기` + 브라우저 콤보(BROWSERS 순서, 화면 글자 `Firefox, Edge, Chrome, Whale, Brave, Opera, Vivaldi, Chromium`) / `cookies.txt 파일` + 경로 칸 + `찾기` 버튼(QFileDialog). 안내 한 줄: `Chrome·Edge 는 켜져 있으면 쿠키를 못 읽을 수 있어요`.
  - 프록시 칸(안내 `http://127.0.0.1:8080`), 속도 제한 칸(안내 `예: 5M`), 재시도 QSpinBox(0~50).
  - 파일 이름 콤보(`제목`, `제목 [ID]`, `업로더 - 제목`, `날짜 제목`, `직접 입력`) + 직접 입력 칸(custom 일 때만 활성). 체크 `같은 이름의 파일이 있으면 덮어쓰기`.
  - 동시 다운로드 QSpinBox(1~5).
  - 고급 칸 QLineEdit(안내 `예: --limit-rate 1M --no-mtime`) + 안내 한 줄 `yt-dlp 명령줄 옵션을 그대로 씁니다. 위 설정보다 우선해요.`
  - 각 검사 대상 칸 아래 오류 라벨(objectName `fieldError`, 처음엔 숨김). 버튼 `취소`, `저장`(objectName `primary`).
- [ ] **Step 5: 테마** — `ytdl/theme.py` `_TEMPLATE`에 추가: `QTabWidget::pane`(배경 `$card_bg`, 테두리 `$border`, radius 8px), `QTabBar::tab`(투명, `$muted`, padding 6px 12px; `:selected`는 `$text`, 아래 2px `$accent`), `QComboBox`·`QSpinBox`(`QLineEdit`과 같은 배경·radius·padding, `:focus` 테두리 `$accent`), `QCheckBox, QRadioButton { spacing: 6px; }`, `#fieldError { color: $danger; font-size: 12px; }`, `#cardSite { color: $muted; font-size: 11px; }`, `#optionsPanel { background: transparent; }`.
- [ ] **Step 6: 통과 확인** — Run: `python -m pytest -q` / Expected: 전체 통과
- [ ] **Step 7: 눈으로 확인** — 패널과 설정 창을 offscreen 으로 `QWidget.grab().save(...)` 해 `$CLAUDE_JOB_DIR/tmp/panel.png`, `settings.png`로 저장하고 Read 로 본다. 글자 잘림·겹침이 없어야 한다.
- [ ] **Step 8: 커밋** — "세부 옵션 패널과 설정 창"

---

### Task 8: 메인 창 연결 — 패널·설정·대기열·사이트 줄·재생목록

**Files:**
- Modify: `ytdl/main_window.py`, `app.py`(필요 없으면 그대로), `README.md`
- Test: `tests/test_main_window.py`

**Interfaces:**
- Consumes: Task 1~7 전부. `DownloadJob(url, ydl_opts)`와 그 신호, `PlaylistResult`, `JobQueue`, `SiteIcons`, `OptionsPanel`, `SettingsDialog`, `to_ydl_opts`, `PlaylistContext`, `validate_options`, `describe`, `changed_summary`, `apply_preset`, `matching_preset`, `options_from_dict`, `settings_from_dict`, `to_dict`, `FFMPEG_DIR`.
- Produces:
  - `MainWindow(settings: QSettings | None = None, data_dir: Path | None = None)` — 기본값은 지금의 `QSettings(*SETTINGS)`와 `QStandardPaths.writableLocation(AppDataLocation)`. 아카이브는 `data_dir / "archive.txt"`, 아이콘 캐시는 `data_dir / "favicons"`.
  - 속성: `self.panel`, `self.jobs: JobQueue[DownloadJob]`, `self.opts: DownloadOptions`, `self.app_settings: AppSettings`, `self.form_error: QLabel`.
  - `@dataclass class Snapshot: opts: DownloadOptions; settings: AppSettings; folder: str` — 작업마다 `job.snapshot`에 붙인다.
  - `JobCard(url: str, fmt_label: str, icons: SiteIcons)` + `set_site(extractor_key: str, webpage_url: str)`, `set_skipped()`, `set_playlist(title: str, added: int, skipped: int)`, `set_cancelled()`. 끝난 상태(`status` 의 `state` 속성)는 `done`, `error`, `skipped`, `playlist`.
  - `MainWindow._on_expanded(card: JobCard, job: DownloadJob, result: PlaylistResult) -> None`, `MainWindow._confirm_many(n: int) -> bool` (테스트에서 바꿔 끼운다).

- [ ] **Step 1: 실패하는 테스트 작성** — `tests/test_main_window.py`

```python
import pytest
from PySide6.QtCore import QSettings

from ytdl import main_window as mw
from ytdl.downloader import DownloadJob, PlaylistEntry, PlaylistResult
from ytdl.options import DownloadOptions


@pytest.fixture
def win(qapp, tmp_path, monkeypatch):
    monkeypatch.setattr(DownloadJob, "start", lambda self: None)  # 네트워크 없이
    s = QSettings(str(tmp_path / "s.ini"), QSettings.Format.IniFormat)
    s.setValue("folder", str(tmp_path))
    w = mw.MainWindow(settings=s, data_dir=tmp_path / "data")
    yield w
    w.jobs = mw.JobQueue(1, lambda j: None)  # closeEvent 가 가짜 작업을 건드리지 않게


def cards(w):
    return [w.list.itemAt(i).widget() for i in range(w.list.count()) if isinstance(w.list.itemAt(i).widget(), mw.JobCard)]


def test_legacy_format_is_migrated(qapp, tmp_path):
    s = QSettings(str(tmp_path / "old.ini"), QSettings.Format.IniFormat)
    s.setValue("format", "mp3")
    w = mw.MainWindow(settings=s, data_dir=tmp_path)
    assert w.opts.kind == "audio" and w.fmt_group.checkedButton().property("fmt") == "mp3"


def test_add_job_snapshots_options(win, tmp_path):
    win.url.setText("https://www.youtube.com/watch?v=abc")
    win.add_job()
    (card,) = cards(win)
    (job,) = win.jobs.running
    assert job.ydl_opts["paths"] == {"home": str(tmp_path)} and job.snapshot.opts == win.opts
    assert card.site_name.text() == "youtube.com"
    assert win.settings.value("options/kind") == "video"


def test_invalid_section_blocks_add(win):
    win.panel.set_options(DownloadOptions(section_start="abc"))
    win._panel_changed()
    win.url.setText("https://x.com/v")
    win.add_job()
    assert cards(win) == [] and win.form_error.text() == "시간은 1:30 처럼 써 주세요"


def test_concurrency_from_settings(win):
    win.app_settings.concurrency = 2
    win.jobs.set_limit(2)
    for i in range(3):
        win.url.setText(f"https://x.com/{i}"); win.add_job()
    assert len(win.jobs.running) == 2 and len(win.jobs.pending) == 1


def test_expand_playlist(win):
    win.panel.set_options(DownloadOptions(playlist="all", number_prefix=True)); win._panel_changed()
    win.url.setText("https://www.youtube.com/playlist?list=P"); win.add_job()
    (card,), (job,) = cards(win), win.jobs.running
    result = PlaylistResult("내 목록", "YoutubeTab", "https://www.youtube.com/playlist?list=P",
                            [PlaylistEntry(f"https://y/{i}", f"t{i}", i) for i in (1, 2, 3)], skipped=2)
    win._on_expanded(card, job, result)
    cs = cards(win)
    assert cs[0] is card and len(cs) == 4
    assert card.title.text() == "재생목록 · 내 목록" and card.status.text() == "영상 3개 추가 · 2개는 이미 받아서 건너뜀"
    new_jobs = [j for j in win.jobs.running + win.jobs.pending if j is not job]
    assert [j.url for j in new_jobs] == ["https://y/1", "https://y/2", "https://y/3"]
    assert new_jobs[0].ydl_opts["outtmpl"]["default"] == "01 - %(title)s.%(ext)s"
    assert new_jobs[0].ydl_opts["noplaylist"] is True


def test_expand_many_asks_and_can_cancel(win, monkeypatch):
    asked = []
    monkeypatch.setattr(win, "_confirm_many", lambda n: asked.append(n) or False)
    win.url.setText("https://x.com/list"); win.add_job()
    (card,), (job,) = cards(win), win.jobs.running
    win._on_expanded(card, job, PlaylistResult("big", "", "", [PlaylistEntry(f"u{i}", "", i) for i in range(201)], 0))
    assert asked == [201] and len(cards(win)) == 1 and card.status.text() == "취소했어요"


def test_clear_finished_keeps_running(win):
    for i in range(4):
        win.url.setText(f"https://x.com/{i}"); win.add_job()
    c = cards(win)
    c[0].set_done(""); c[1].set_skipped(); c[2].set_playlist("p", 0, 0)
    win.clear_finished()
    assert cards(win) == [c[3]]


def test_close_stops_every_running_job(win, monkeypatch):
    stopped = []
    monkeypatch.setattr(DownloadJob, "isRunning", lambda self: True)
    monkeypatch.setattr(DownloadJob, "terminate", lambda self: stopped.append(self))
    monkeypatch.setattr(DownloadJob, "wait", lambda self, ms=0: True)
    win.jobs.set_limit(3)
    for i in range(3):
        win.url.setText(f"https://x.com/{i}"); win.add_job()
    running = win.jobs.running
    win.close()
    assert stopped == running
```

- [ ] **Step 2: 실패 확인** — Run: `python -m pytest tests/test_main_window.py -q` / Expected: FAIL

- [ ] **Step 3: 카드** — `JobCard`
  - 맨 위에 사이트 줄: 16×16 `QLabel`(`site_icon`) + `QLabel`(`site_name`, objectName `cardSite`). 처음엔 `domain_of(url)`과 `icons.icon(host)`. `icons.ready`에 연결해 자기 host 면 아이콘을 바꾼다.
  - `set_site(key, webpage_url)`: host = `domain_of(webpage_url)` 또는 처음 host, 이름 = `display_name(key, host)`, 아이콘 갱신.
  - `set_skipped()`: 막대 숨김, 퍼센트 비움, 상태 `skipped`, 문구 `이미 받은 영상이라 건너뛰었어요`.
  - `set_playlist(title, added, skipped)`: 제목 `재생목록 · <title>`, 막대·퍼센트 숨김, 상태 `playlist`, 문구 `영상 <added>개 추가` + (skipped > 0 이면 ` · <skipped>개는 이미 받아서 건너뜀`).
  - `set_cancelled()`: 막대 숨김, 상태 `skipped`, 문구 `취소했어요`.
  - `fmt_label`은 `describe(opts)`.
- [ ] **Step 4: 창 구성**
  - 헤더 오른쪽에 `⚙` 버튼(objectName `ghost`, 툴팁 `설정`) → `SettingsDialog(self.app_settings, self)`. `exec()`가 Accepted 면 `self.app_settings = d.value()`, 저장(`settings/<필드>`), `self.jobs.set_limit(concurrency)`.
  - 형식 줄: 프리셋 버튼 3개(`PRESET_LABELS`, 버튼 `fmt` 속성 = 프리셋 키) + `세부 옵션 ▾` 토글 버튼(objectName `ghost`) + 늘어남 + 다운로드. 프리셋 버튼 그룹은 배타 선택(`fmt_group`, 지금 그대로)으로 두고, `matching_preset`이 None 이면 잠깐 `setExclusive(False)` → 모두 해제 → `setExclusive(True)`. (비배타 그룹의 `checkedButton()`은 믿을 수 없다.)
  - 토글 버튼을 누르면 패널을 보이거나 숨기고 `ui/panel_open`에 저장한다.
  - 그 아래 `self.form_error`(objectName `fieldError`, 숨김), 그 아래 `self.panel`(`ui/panel_open` 값에 따라 보이기).
  - 토글 버튼 글자: `세부 옵션 ▾`/`세부 옵션 ▴`, `changed_summary`가 있으면 뒤에 ` · ` + `", ".join(...)`.
  - 링크 입력칸 안내: `영상·재생목록 링크를 붙여넣으세요 (유튜브 외 사이트도 돼요)`, 빈칸 안내: `먼저 링크를 넣어 주세요`. 헤더 안내 문구는 `링크를 붙여넣고 형식을 고른 뒤 다운로드를 누르세요.` 그대로.
- [ ] **Step 5: 상태와 저장**
  - 불러오기: `options_from_dict({k: s.value(f"options/{k}") ...}, legacy_format=s.value("format"))`(저장된 키만 모은다), `settings_from_dict` 같은 방식.
  - `_panel_changed()`: `self.opts = self.panel.options()`, 저장, 프리셋 체크 갱신, 토글 글자 갱신, `form_error` 숨김.
  - 프리셋 클릭: `self.opts = apply_preset(self.opts, key)` → `panel.set_options` → 저장·갱신.
- [ ] **Step 6: 작업 흐름**
  - `add_job`: 빈 링크 처리(기존과 같되 새 안내 문구) → `validate_options(self.opts)` 오류면 첫 문구를 `form_error`에 보이고 끝 → 지금 옵션을 `options/<필드>`로 저장(기존의 `format` 저장을 대신한다) → `_enqueue(url, Snapshot(replace(self.opts), replace(self.app_settings), self.folder), insert_at=1, playlist=None)`.
  - `_enqueue(...) -> JobCard`: `to_ydl_opts(snap.opts, snap.settings, folder=snap.folder, archive=str(self.archive), ffmpeg_dir=FFMPEG_DIR, playlist=playlist)`로 `DownloadJob`을 만들고 `job.snapshot = snap`. 신호 연결: `titled`→`set_title`, `progress`, `finished_ok`→`set_done`, `failed`→`set_failed`, `site_found`→`set_site`, `skipped`→`set_skipped`, `expanded`→`lambda r: self._on_expanded(card, job, r)`, `finished`→`lambda: self.jobs.done(job)`. 카드를 `insert_at`에 넣고 `self.jobs.add(job)`.
  - `JobQueue`의 start 콜백: `card.set_progress(0, "영상 정보 불러오는 중…")` 후 `job.start()` (작업 → 카드 dict `self._cards`로 찾는다).
  - `_on_expanded`: 항목이 200개를 넘으면 `_confirm_many(n)`(QMessageBox.question, 제목 `재생목록`, 본문 `영상 <n>개를 모두 받을까요?`) → False 면 `card.set_cancelled()`. 아니면 `card.set_playlist(...)`, 카드의 사이트 갱신, 각 항목을 `PlaylistContext(result.title, e.index, len(entries))`로 `_enqueue` 하되 위치는 요약 카드 바로 아래부터 차례대로.
  - `clear_finished`: 상태가 `done`, `error`, `skipped`, `playlist`인 카드를 지운다.
  - `closeEvent`: `self.jobs.running` 전부 `isRunning()`이면 `terminate()` + `wait(2000)`.
- [ ] **Step 7: README** — 기능 목록을 바꾼다: `화질·파일 형식·코덱 고르기, 음원(MP3·M4A·OPUS·FLAC·WAV)`, `재생목록·채널 한 번에 받기(범위, 번호, 폴더, 이미 받은 영상 건너뛰기)`, `자막 받기·넣기`, `썸네일·메타데이터·챕터 넣기, 챕터별 나누기, 구간만 받기`, `브라우저 쿠키·프록시·속도 제한`, `파일 이름 규칙, 동시 다운로드`, `고급: yt-dlp 명령줄 옵션 직접 입력`, `유튜브 외 yt-dlp 지원 사이트, 카드에 사이트 아이콘과 이름 표시`, 그리고 기존 `새 버전이 나오면 앱이 알려 주고 스스로 업데이트`. 설명 줄 `링크만 붙여넣으면 영상(MP4)이나 음원(MP3)을 받는` → `링크만 붙여넣으면 영상이나 음원을 받는`.
- [ ] **Step 8: 통과 확인** — Run: `python -m pytest -q` / Expected: 전체 통과
- [ ] **Step 9: 커밋** — "메인 창에 패널·설정·동시 다운로드·사이트 표시 연결"

---

### Task 9: 실제로 받아 보기와 빌드

**Files:** 없음 (문제가 나오면 해당 Task 파일을 고치고 테스트를 더한다)

- [ ] **Step 1: 실행** — `python app.py`로 앱을 띄우거나, 클릭이 어려우면 Task 6 Step 5 처럼 `DownloadJob`을 스크립트로 돌린다. 저장 위치는 `$CLAUDE_JOB_DIR/tmp/dl`.
- [ ] **Step 2: 확인 목록** — 각 항목의 결과 파일을 `ffprobe`로 본다.
  1. 유튜브 영상 1개, 최고 화질 → mp4, 영상+음성 스트림.
  2. 짧은 재생목록, 전체 + 범위 `1-3` + 번호 + 폴더 → `<목록 이름>/01 - …` 3개.
  3. 같은 재생목록을 "이미 받은 영상 건너뛰기"로 두 번 → 두 번째는 `3개는 이미 받아서 건너뜀`.
  4. 유튜브 외 사이트 1곳(예: Vimeo 공개 영상) → 받아지고 사이트 이름이 표시됨.
  5. 자막 넣기(ko 또는 en) → mp4 안에 자막 스트림.
  6. MP3 + 썸네일 넣기 → mp3 안에 그림(attached pic).
  7. 챕터 넣기 → `ffprobe -show_chapters`에 챕터.
  8. 구간 `0:10`~`0:20` → 길이 약 10초.
  9. 동시 2개 → 두 카드가 함께 진행.
  10. 고급 `--limit-rate 1M` → 속도 1MiB/s 근처.
  11. 사이트 아이콘이 카드에 보이고 `%APPDATA%\QuickMedia\favicons`에 png.
- [ ] **Step 3: 빌드** — `./build.ps1` → `dist/QuickMedia-Setup-<버전>.exe` 생성. 설치본을 실행해 Step 2 의 1·6 을 한 번 더 본다(묶인 ffmpeg 확인). 끝나면 설치본을 지운다.
- [ ] **Step 4: 전체 테스트** — Run: `python -m pytest -q` / Expected: 전체 통과
- [ ] **Step 5: 커밋** — 고친 것이 있으면 "실제 다운로드 확인 중 발견한 문제 수정"
