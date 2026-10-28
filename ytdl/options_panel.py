"""펼치는 "세부 옵션" 패널: 형식 / 자막 / 후처리 / 재생목록."""

from __future__ import annotations

from PySide6.QtCore import Signal
from PySide6.QtWidgets import (
    QButtonGroup, QCheckBox, QComboBox, QFormLayout, QHBoxLayout, QLabel, QLineEdit, QRadioButton,
    QTabWidget, QVBoxLayout, QWidget,
)

from .options import (
    AUDIO_FORMATS, CONTAINERS, LOSSLESS, SUB_FORMATS, DownloadOptions,
)

_QUALITY_ITEMS = [("최고", "best"), ("2160p", "2160"), ("1440p", "1440"), ("1080p", "1080"),
                  ("720p", "720"), ("480p", "480"), ("360p", "360")]
_CODEC_ITEMS = [("자동", "auto"), ("H.264 (호환성 좋음)", "h264"), ("VP9", "vp9"), ("AV1", "av1")]
_AUDIO_QUALITY_ITEMS = [("최고", "best"), ("320k", "320"), ("192k", "192"), ("128k", "128")]
_LANG_ITEMS = [("한국어", "ko"), ("영어", "en"), ("모든 언어", "all"), ("직접 입력", "")]


def _combo(items: list[tuple[str, str]]) -> QComboBox:
    box = QComboBox()
    for label, value in items:
        box.addItem(label, value)
    return box


def _upper_items(values: tuple[str, ...]) -> list[tuple[str, str]]:
    return [(v.upper(), v) for v in values]


def _select(box: QComboBox, value: str) -> None:
    i = box.findData(value)
    box.setCurrentIndex(i if i >= 0 else 0)


def _radio_pair(a: str, b: str) -> tuple[QRadioButton, QRadioButton, QWidget]:
    holder = QWidget()
    row = QHBoxLayout(holder)
    row.setContentsMargins(0, 0, 0, 0)
    first, second = QRadioButton(a), QRadioButton(b)
    group = QButtonGroup(holder)
    group.addButton(first)
    group.addButton(second)
    row.addWidget(first)
    row.addWidget(second)
    row.addStretch(1)
    first.setChecked(True)
    return first, second, holder


class OptionsPanel(QWidget):
    changed = Signal()

    def __init__(self, parent: QWidget | None = None):
        super().__init__(parent)
        self.setObjectName("optionsPanel")
        self._loading = False
        root = QVBoxLayout(self)
        root.setContentsMargins(0, 0, 0, 0)
        self.tabs = QTabWidget()
        root.addWidget(self.tabs)
        self.tabs.addTab(self._format_tab(), "형식")
        self.tabs.addTab(self._subs_tab(), "자막")
        self.tabs.addTab(self._post_tab(), "후처리")
        self.tabs.addTab(self._playlist_tab(), "재생목록")

        for w in self.findChildren(QCheckBox) + self.findChildren(QRadioButton):
            w.toggled.connect(self._edited)
        for w in self.findChildren(QComboBox):
            w.currentIndexChanged.connect(self._edited)
        for w in self.findChildren(QLineEdit):
            w.textEdited.connect(self._edited)
        self.set_options(DownloadOptions())

    # ---------- 탭 ----------
    def _format_tab(self) -> QWidget:
        tab = QWidget()
        col = QVBoxLayout(tab)
        self.kind_video, self.kind_audio, kinds = _radio_pair("영상", "음원만")
        col.addWidget(kinds)

        self.video_box = QWidget()
        form = QFormLayout(self.video_box)
        form.setContentsMargins(0, 0, 0, 0)
        self.quality = _combo(_QUALITY_ITEMS)
        self.container = _combo(_upper_items(CONTAINERS))
        self.codec = _combo(_CODEC_ITEMS)
        form.addRow("화질", self.quality)
        form.addRow("파일 형식", self.container)
        form.addRow("코덱", self.codec)
        col.addWidget(self.video_box)

        self.audio_box = QWidget()
        form = QFormLayout(self.audio_box)
        form.setContentsMargins(0, 0, 0, 0)
        self.audio_format = _combo(_upper_items(AUDIO_FORMATS))
        self.audio_quality = _combo(_AUDIO_QUALITY_ITEMS)
        form.addRow("형식", self.audio_format)
        form.addRow("음질", self.audio_quality)
        col.addWidget(self.audio_box)
        col.addStretch(1)
        return tab

    def _subs_tab(self) -> QWidget:
        tab = QWidget()
        col = QVBoxLayout(tab)
        self.subs = QCheckBox("자막 받기")
        col.addWidget(self.subs)
        form = QFormLayout()
        lang_row = QHBoxLayout()
        self.sub_lang = _combo(_LANG_ITEMS)
        self.sub_lang_custom = QLineEdit()
        self.sub_lang_custom.setPlaceholderText("ko,en")
        lang_row.addWidget(self.sub_lang)
        lang_row.addWidget(self.sub_lang_custom, 1)
        form.addRow("언어", lang_row)
        self.auto_subs = QCheckBox("자동 생성 자막 포함")
        form.addRow("", self.auto_subs)
        self.sub_embed, self.sub_file, modes = _radio_pair("영상 파일 안에 넣기", "별도 파일")
        form.addRow("저장 방식", modes)
        self.sub_format = _combo(_upper_items(SUB_FORMATS))
        form.addRow("자막 형식", self.sub_format)
        col.addLayout(form)
        col.addStretch(1)
        return tab

    def _post_tab(self) -> QWidget:
        tab = QWidget()
        col = QVBoxLayout(tab)
        self.embed_thumbnail = QCheckBox("썸네일을 파일에 넣기")
        self.save_thumbnail = QCheckBox("썸네일을 이미지 파일로도 저장")
        self.embed_metadata = QCheckBox("제목·업로더 같은 메타데이터 넣기")
        self.embed_chapters = QCheckBox("챕터 정보 넣기")
        self.split_chapters = QCheckBox("챕터별로 파일 나누기")
        for w in (self.embed_thumbnail, self.save_thumbnail, self.embed_metadata,
                  self.embed_chapters, self.split_chapters):
            col.addWidget(w)
        row = QHBoxLayout()
        row.addWidget(QLabel("구간만 받기"))
        self.section_start = QLineEdit()
        self.section_start.setPlaceholderText("1:30")
        self.section_end = QLineEdit()
        self.section_end.setPlaceholderText("3:00")
        row.addWidget(self.section_start)
        row.addWidget(QLabel("~"))
        row.addWidget(self.section_end)
        row.addStretch(1)
        col.addLayout(row)
        col.addStretch(1)
        return tab

    def _playlist_tab(self) -> QWidget:
        tab = QWidget()
        col = QVBoxLayout(tab)
        self.playlist_single, self.playlist_all, modes = _radio_pair("이 영상만", "재생목록 전체")
        col.addWidget(modes)
        form = QFormLayout()
        self.playlist_items = QLineEdit()
        self.playlist_items.setPlaceholderText("1-10, 15")
        form.addRow("받을 범위", self.playlist_items)
        col.addLayout(form)
        self.number_prefix = QCheckBox("파일 이름 앞에 번호 붙이기")
        self.playlist_folder = QCheckBox("재생목록 이름으로 폴더 만들기")
        self.skip_archived = QCheckBox("이미 받은 영상 건너뛰기")
        for w in (self.number_prefix, self.playlist_folder, self.skip_archived):
            col.addWidget(w)
        col.addStretch(1)
        return tab

    # ---------- 값 ----------
    def set_options(self, o: DownloadOptions) -> None:
        self._loading = True
        try:
            (self.kind_audio if o.kind == "audio" else self.kind_video).setChecked(True)
            _select(self.quality, o.quality)
            _select(self.container, o.container)
            _select(self.codec, o.codec)
            _select(self.audio_format, o.audio_format)
            _select(self.audio_quality, o.audio_quality)
            self.subs.setChecked(o.subs)
            if o.sub_langs in ("ko", "en", "all"):
                _select(self.sub_lang, o.sub_langs)
                self.sub_lang_custom.clear()
            else:
                _select(self.sub_lang, "")
                self.sub_lang_custom.setText(o.sub_langs)
            self.auto_subs.setChecked(o.auto_subs)
            (self.sub_file if o.sub_mode == "file" else self.sub_embed).setChecked(True)
            _select(self.sub_format, o.sub_format)
            self.embed_thumbnail.setChecked(o.embed_thumbnail)
            self.save_thumbnail.setChecked(o.save_thumbnail)
            self.embed_metadata.setChecked(o.embed_metadata)
            self.embed_chapters.setChecked(o.embed_chapters)
            self.split_chapters.setChecked(o.split_chapters)
            self.section_start.setText(o.section_start)
            self.section_end.setText(o.section_end)
            (self.playlist_all if o.playlist == "all" else self.playlist_single).setChecked(True)
            self.playlist_items.setText(o.playlist_items)
            self.number_prefix.setChecked(o.number_prefix)
            self.playlist_folder.setChecked(o.playlist_folder)
            self.skip_archived.setChecked(o.skip_archived)
        finally:
            self._loading = False
        self._refresh()

    def options(self) -> DownloadOptions:
        lang = self.sub_lang.currentData() or self.sub_lang_custom.text().strip()
        return DownloadOptions(
            kind="audio" if self.kind_audio.isChecked() else "video",
            quality=self.quality.currentData(),
            container=self.container.currentData(),
            codec=self.codec.currentData(),
            audio_format=self.audio_format.currentData(),
            audio_quality=self.audio_quality.currentData(),
            subs=self.subs.isChecked(),
            sub_langs=lang or "ko",
            auto_subs=self.auto_subs.isChecked(),
            sub_mode="file" if self.sub_file.isChecked() else "embed",
            sub_format=self.sub_format.currentData(),
            embed_thumbnail=self.embed_thumbnail.isChecked(),
            save_thumbnail=self.save_thumbnail.isChecked(),
            embed_metadata=self.embed_metadata.isChecked(),
            embed_chapters=self.embed_chapters.isChecked(),
            split_chapters=self.split_chapters.isChecked(),
            section_start=self.section_start.text().strip(),
            section_end=self.section_end.text().strip(),
            playlist="all" if self.playlist_all.isChecked() else "single",
            playlist_items=self.playlist_items.text().strip(),
            number_prefix=self.number_prefix.isChecked(),
            playlist_folder=self.playlist_folder.isChecked(),
            skip_archived=self.skip_archived.isChecked(),
        )

    def _refresh(self) -> None:
        """고른 값에 따라 칸을 보이거나 끈다."""
        audio = self.kind_audio.isChecked()
        self.video_box.setVisible(not audio)
        self.audio_box.setVisible(audio)
        self.audio_quality.setEnabled(self.audio_format.currentData() not in LOSSLESS)
        subs = self.subs.isChecked()
        custom = self.sub_lang.currentData() == ""
        for w in (self.sub_lang, self.auto_subs, self.sub_embed, self.sub_file):
            w.setEnabled(subs)
        self.sub_lang_custom.setVisible(custom)
        self.sub_lang_custom.setEnabled(subs)
        self.sub_format.setEnabled(subs and self.sub_file.isChecked())
        whole = self.playlist_all.isChecked()
        for w in (self.playlist_items, self.number_prefix, self.playlist_folder):
            w.setEnabled(whole)

    def _edited(self, *_) -> None:
        if self._loading:
            return
        self._refresh()
        self.changed.emit()
