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
