from ytdl.options import DownloadOptions
from ytdl.options_panel import OptionsPanel

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

