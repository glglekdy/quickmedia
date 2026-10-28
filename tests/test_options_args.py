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
    assert o.get("ffmpeg_location") is None  # yt-dlp 가 키를 None 으로 둔다
