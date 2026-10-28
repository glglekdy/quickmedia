from pathlib import Path

import pytest
import yt_dlp

from ytdl.downloader import NumberIfExistsPP, PlaylistEntry, friendly_error, playlist_entries
from ytdl.options import AppSettings, DownloadOptions, to_ydl_opts


@pytest.mark.parametrize("msg, expected", [
    ("ERROR: Unsupported URL: https://x", "지원하지 않는 링크예요"),
    ("ERROR: could not find chrome cookies database in ...", "브라우저 쿠키를 읽지 못했어요. 브라우저를 닫고 다시 시도하거나 cookies.txt 를 써 주세요"),
    ("ERROR: Could not copy Chrome cookie database. See ...", "브라우저 쿠키를 읽지 못했어요. 브라우저를 닫고 다시 시도하거나 cookies.txt 를 써 주세요"),
    ("ERROR: Failed to decrypt with DPAPI", "브라우저 쿠키를 읽지 못했어요. 브라우저를 닫고 다시 시도하거나 cookies.txt 를 써 주세요"),
    ("ERROR: Sign in to confirm your age", "로그인이 필요한 영상이에요. ⚙ 설정에서 쿠키를 켜 보세요"),
    ("ERROR: Join this channel to get access to members-only content", "로그인이 필요한 영상이에요. ⚙ 설정에서 쿠키를 켜 보세요"),
    ("ERROR: Video unavailable", "삭제되었거나 볼 수 없는 영상이에요"),
    ("ERROR: [youtube] aaaaaaaaaaa: This video is unavailable", "삭제되었거나 볼 수 없는 영상이에요"),
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


class _FakeYDL:
    """yt-dlp 처럼 펼칠 때 받은 기록에 있는 항목을 말없이 뺀다."""

    def __init__(self, opts):
        self.archive = {"youtube b"} if opts.get("download_archive") else set()

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        return False

    def add_post_processor(self, pp, when):
        pass

    def in_download_archive(self, entry):
        return f"{entry['ie_key'].lower()} {entry['id']}" in self.archive

    def extract_info(self, url, download=False):
        entries = [{"url": f"https://y/{i}", "id": i, "ie_key": "Youtube", "title": i} for i in "abc"]
        return {"_type": "playlist", "title": "p", "extractor_key": "YoutubeTab", "webpage_url": url,
                "entries": [e for e in entries if not self.in_download_archive(e)]}


def test_expand_counts_archived_entries(monkeypatch):
    from ytdl import downloader
    monkeypatch.setattr(downloader.yt_dlp, "YoutubeDL", _FakeYDL)
    job = downloader.DownloadJob("https://y/list", {"download_archive": "a.txt"})
    got = []
    job.expanded.connect(got.append)
    job.run()
    (result,) = got
    assert [e.url for e in result.entries] == ["https://y/a", "https://y/c"] and result.skipped == 1


def _renamed(tmp_path: Path, existing: list[str], info: dict, *,
             opts: DownloadOptions | None = None, settings: AppSettings | None = None) -> str:
    """이미 있는 파일을 만들어 두고, 형식이 정해진 뒤 단계를 거친 실제 저장 이름."""
    for name in existing:
        (tmp_path / name).write_bytes(b"x")
    ydl_opts = to_ydl_opts(opts or DownloadOptions(), settings or AppSettings(),
                           folder=str(tmp_path), archive=str(tmp_path / "a.txt"))
    with yt_dlp.YoutubeDL(ydl_opts) as ydl:
        _, info = NumberIfExistsPP(ydl).run({"id": "x", "title": "Me", **info})
        return Path(ydl.prepare_filename(info)).name


def test_number_if_exists_keeps_name_when_free(tmp_path):
    assert _renamed(tmp_path, [], {"ext": "mp4"}) == "Me.mp4"


def test_number_if_exists_adds_next_free_number(tmp_path):
    assert _renamed(tmp_path, ["Me.mp4"], {"ext": "mp4"}) == "Me (1).mp4"
    assert _renamed(tmp_path, ["Me (1).mp4"], {"ext": "mp4"}) == "Me (2).mp4"


def test_number_if_exists_checks_converted_audio(tmp_path):
    # 음성은 m4a 로 받아 mp3 로 바꾼다. 남아 있는 건 mp3 다.
    got = _renamed(tmp_path, ["Me.mp3"], {"ext": "m4a"}, opts=DownloadOptions(kind="audio", audio_format="mp3"))
    assert got == "Me (1).m4a"


def test_number_if_exists_leaves_overwrite_alone(tmp_path):
    assert _renamed(tmp_path, ["Me.mp4"], {"ext": "mp4"}, settings=AppSettings(overwrite=True)) == "Me.mp4"
