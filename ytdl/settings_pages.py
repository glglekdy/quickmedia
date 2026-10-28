"""설정 화면 4개(로그인 / 네트워크 / 파일 / 고급). 바꾸면 바로 저장한다.

`SettingsModel` 하나를 네 화면이 같이 본다. 칸을 바꾸면 그 칸(과 묶인 칸)만 `model.update`에 넘기고,
틀리면 저장하지 않고 칸 아래에 오류를 띄운다. 다른 화면으로 갈 때 `leave()`가 틀린 입력을 버린다.
"""

from __future__ import annotations

from PySide6.QtCore import QObject, Qt, QTimer, Signal
from PySide6.QtWidgets import (
    QButtonGroup, QCheckBox, QComboBox, QFileDialog, QFrame, QGridLayout, QHBoxLayout, QLabel, QLineEdit,
    QPushButton, QRadioButton, QScrollArea, QSizePolicy, QSpinBox, QVBoxLayout, QWidget,
)

from .options import BROWSERS, ERROR_CAUSES, AppSettings, update_settings

_BROWSER_LABELS = {"firefox": "Firefox", "edge": "Edge", "chrome": "Chrome", "whale": "Whale",
                   "brave": "Brave", "opera": "Opera", "vivaldi": "Vivaldi", "chromium": "Chromium"}
_TEMPLATE_ITEMS = [("제목", "title"), ("제목 [ID]", "title_id"), ("업로더 - 제목", "uploader_title"),
                   ("날짜 제목", "date_title"), ("직접 입력", "custom")]


class SettingsModel(QObject):
    """지금 저장된 설정. 값이 실제로 바뀔 때만 `changed`를 낸다."""

    changed = Signal(object)

    def __init__(self, value: AppSettings, parent: QObject | None = None):
        super().__init__(parent)
        self.value = value

    def update(self, **fields) -> dict[str, str]:
        new, errors = update_settings(self.value, **fields)
        if new != self.value:
            self.value = new
            self.changed.emit(new)
        return errors


class SettingsPage(QWidget):
    """제목 + 설명 + 카드(이름 | 입력 칸, 그 아래 안내·오류) 틀. 화면마다 `_build`, `_read`, `_fill`을 채운다."""

    title = ""
    description = ""
    label_width = 112
    # 글자 칸 → 그 칸을 저장할 때 같이 넘기는 필드 묶음
    text_groups: dict[str, tuple[str, ...]] = {}
    # 오류 키 → 빨간 테두리를 두를 칸
    error_fields: dict[str, str] = {}

    def __init__(self, model: SettingsModel, parent: QWidget | None = None):
        super().__init__(parent)
        self.model = model
        self.errors: dict[str, QLabel] = {}
        self._loading = False
        self._fields_of: dict[QObject, tuple[str, ...]] = {}

        outer = QVBoxLayout(self)
        outer.setContentsMargins(0, 0, 0, 0)
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setFrameShape(QFrame.Shape.NoFrame)
        outer.addWidget(scroll)
        holder = QWidget()
        holder.setObjectName("settingsHolder")
        scroll.setWidget(holder)
        col = QVBoxLayout(holder)
        col.setContentsMargins(28, 24, 28, 24)
        col.setSpacing(0)

        head = QHBoxLayout()
        head.setSpacing(10)
        title = QLabel(self.title)
        title.setObjectName("crumb")
        head.addWidget(title)
        self.saved_badge = QLabel("✓ 저장했어요")
        self.saved_badge.setObjectName("savedBadge")
        self.saved_badge.hide()
        head.addWidget(self.saved_badge, 0, Qt.AlignmentFlag.AlignVCenter)
        head.addStretch(1)
        col.addLayout(head)
        col.addSpacing(4)
        desc = QLabel(self.description)
        desc.setObjectName("viewHint")
        desc.setWordWrap(True)
        col.addWidget(desc)
        col.addSpacing(16)

        card = QFrame()
        card.setObjectName("settingsCard")
        self._grid = QGridLayout(card)
        self._grid.setContentsMargins(20, 18, 20, 18)
        self._grid.setHorizontalSpacing(12)
        self._grid.setVerticalSpacing(14)
        self._grid.setColumnMinimumWidth(0, self.label_width)
        self._grid.setColumnStretch(1, 1)
        col.addWidget(card)
        col.addStretch(1)

        self._badge_timer = QTimer(self)
        self._badge_timer.setSingleShot(True)
        self._badge_timer.setInterval(1500)
        self._badge_timer.timeout.connect(self.saved_badge.hide)

        self._build()
        for name, fields in self.text_groups.items():
            self._watch(getattr(self, name), "editingFinished", *fields)
        self.load(model.value)

    # 화면마다 채우는 곳
    def _build(self) -> None:
        raise NotImplementedError

    def _read(self) -> dict:
        """칸에 지금 들어 있는 값. 이 화면이 맡은 필드만."""
        raise NotImplementedError

    def _fill(self, s: AppSettings) -> None:
        raise NotImplementedError

    def _refresh(self, *_) -> None:
        """켜고 끄는 규칙."""

    # 틀
    def _watch(self, widget: QObject, signal: str, *fields: str) -> None:
        """widget 의 signal 이 오면 fields 를 저장한다.

        lambda 로 self 를 붙잡으면 화면이 지워지지 않고 앱이 끝날 때 죽으므로 bound method 로 잇는다.
        """
        self._fields_of[widget] = fields
        getattr(widget, signal).connect(self._on_edit)

    def _on_edit(self, *_) -> None:
        self._refresh()
        self._commit(*self._fields_of[self.sender()])

    def _row(self, label: QWidget | str, control: QWidget | None = None, hint: str = "",
             error: str | None = None) -> None:
        r = self._grid.rowCount()
        if isinstance(label, str):
            label = QLabel(label)
        cell = QVBoxLayout()
        cell.setSpacing(4)
        if control is not None:
            control.setMaximumWidth(400)
            # 글자 칸처럼 늘어나는 칸은 400 까지 늘리고, 나머지는 제 크기로 왼쪽에
            grows = control.sizePolicy().horizontalPolicy() == QSizePolicy.Policy.Expanding
            cell.addWidget(control, 0, Qt.AlignmentFlag(0) if grows else Qt.AlignmentFlag.AlignLeft)
            # 이름을 입력 칸 글자 높이에 맞춘다
            label.setMinimumHeight(control.sizeHint().height())
        if hint:
            note = QLabel(hint)
            note.setObjectName("hint")
            note.setWordWrap(True)
            cell.addWidget(note)
        if error:
            msg = QLabel()
            msg.setObjectName("fieldError")
            msg.setWordWrap(True)
            msg.hide()
            self.errors[error] = msg
            cell.addWidget(msg)
        self._grid.addWidget(label, r, 0, Qt.AlignmentFlag.AlignTop)
        self._grid.addLayout(cell, r, 1)

    def load(self, s: AppSettings) -> None:
        """칸을 s 로 채운다. 채우는 동안에는 저장하지 않는다."""
        self._loading = True
        try:
            self._fill(s)
        finally:
            self._loading = False
        self._refresh()

    def _commit(self, *fields: str) -> None:
        if self._loading:
            return
        values = self._read()
        before = self.model.value
        errors = self.model.update(**{f: values[f] for f in fields})
        for key, label in self.errors.items():
            if ERROR_CAUSES[key] & set(fields):
                self._show_error(key, errors.get(key, ""))
        if self.model.value != before:
            self.saved_badge.show()
            self._badge_timer.start()

    def _show_error(self, key: str, text: str) -> None:
        self.errors[key].setText(text)
        self.errors[key].setVisible(bool(text))
        field = getattr(self, self.error_fields[key])
        field.setProperty("invalid", bool(text))
        field.style().unpolish(field)
        field.style().polish(field)

    def leave(self) -> None:
        """다른 화면으로 가거나 창을 닫을 때. 맞는 글자는 저장하고, 나머지는 저장된 값으로 되돌린다."""
        values = self._read()
        for fields in self.text_groups.values():
            if any(values[f] != getattr(self.model.value, f) for f in fields):
                self._commit(*fields)
        self.load(self.model.value)
        for key in self.errors:
            self._show_error(key, "")


class LoginPage(SettingsPage):
    title = "로그인"
    description = "로그인해야 볼 수 있는 영상은 브라우저의 로그인 정보(쿠키)를 써서 받아요."
    label_width = 160
    text_groups = {"cookie_path": ("cookies", "cookie_browser", "cookie_file")}
    error_fields = {"cookie_file": "cookie_path"}

    def _build(self) -> None:
        self.cookie_none = QRadioButton("사용 안 함")
        self.cookie_browser = QRadioButton("브라우저에서 가져오기")
        self.cookie_file = QRadioButton("cookies.txt 파일")
        group = QButtonGroup(self)
        for b in (self.cookie_none, self.cookie_browser, self.cookie_file):
            group.addButton(b)
        group.buttonToggled.connect(self._on_mode)

        self.browser = QComboBox()
        for key in BROWSERS:
            self.browser.addItem(_BROWSER_LABELS[key], key)
        self._watch(self.browser, "currentIndexChanged", "cookies", "cookie_browser", "cookie_file")

        path_row = QWidget()
        box = QHBoxLayout(path_row)
        box.setContentsMargins(0, 0, 0, 0)
        self.cookie_path = QLineEdit()
        self.cookie_path.setPlaceholderText("파일을 고르세요")
        box.addWidget(self.cookie_path, 1)
        self.browse_btn = QPushButton("찾기")
        self.browse_btn.clicked.connect(self._browse)
        box.addWidget(self.browse_btn)
        path_row.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Fixed)

        self._row(self.cookie_none)
        self._row(self.cookie_browser, self.browser, "Chrome·Edge 는 켜져 있으면 쿠키를 못 읽을 수 있어요")
        self._row(self.cookie_file, path_row, error="cookie_file")

    def _mode(self) -> str:
        return "browser" if self.cookie_browser.isChecked() else "file" if self.cookie_file.isChecked() else "none"

    def _read(self) -> dict:
        return {"cookies": self._mode(), "cookie_browser": self.browser.currentData(),
                "cookie_file": self.cookie_path.text().strip()}

    def _fill(self, s: AppSettings) -> None:
        {"browser": self.cookie_browser, "file": self.cookie_file}.get(s.cookies, self.cookie_none).setChecked(True)
        self.browser.setCurrentIndex(max(0, self.browser.findData(s.cookie_browser)))
        self.cookie_path.setText(s.cookie_file)

    def _refresh(self, *_) -> None:
        self.browser.setEnabled(self.cookie_browser.isChecked())
        file_mode = self.cookie_file.isChecked()
        self.cookie_path.setEnabled(file_mode)
        self.browse_btn.setEnabled(file_mode)

    def _on_mode(self, _button, checked: bool) -> None:
        self._refresh()
        if checked:
            self._commit("cookies", "cookie_browser", "cookie_file")

    def _browse(self) -> None:
        path, _ = QFileDialog.getOpenFileName(self, "cookies.txt 고르기", self.cookie_path.text(),
                                              "쿠키 파일 (*.txt);;모든 파일 (*)")
        if path:
            self.cookie_path.setText(path)
            self._commit("cookies", "cookie_browser", "cookie_file")


class NetworkPage(SettingsPage):
    title = "네트워크"
    description = "인터넷 연결과 받는 속도에 관한 설정이에요."
    text_groups = {"proxy": ("proxy",), "rate_limit": ("rate_limit",)}
    error_fields = {"rate_limit": "rate_limit"}

    def _build(self) -> None:
        self.proxy = QLineEdit()
        self.proxy.setPlaceholderText("http://127.0.0.1:8080")
        self.rate_limit = QLineEdit()
        self.rate_limit.setPlaceholderText("예: 5M")
        self.rate_limit.setFixedWidth(160)
        self.retries = QSpinBox()
        self.retries.setRange(0, 50)
        self.retries.setFixedWidth(90)
        self._watch(self.retries, "valueChanged", "retries")
        self.concurrency = QSpinBox()
        self.concurrency.setRange(1, 5)
        self.concurrency.setFixedWidth(90)
        self._watch(self.concurrency, "valueChanged", "concurrency")

        self._row("프록시", self.proxy)
        self._row("속도 제한", self.rate_limit, "비워 두면 제한 없이 받아요. 5M 은 초당 5MB", error="rate_limit")
        self._row("재시도 횟수", self.retries)
        self._row("동시 다운로드", self.concurrency, "한 번에 받는 영상 수 (1~5)")

    def _read(self) -> dict:
        return {"proxy": self.proxy.text().strip(), "rate_limit": self.rate_limit.text().strip(),
                "retries": self.retries.value(), "concurrency": self.concurrency.value()}

    def _fill(self, s: AppSettings) -> None:
        self.proxy.setText(s.proxy)
        self.rate_limit.setText(s.rate_limit)
        self.retries.setValue(s.retries)
        self.concurrency.setValue(s.concurrency)


class FilePage(SettingsPage):
    title = "파일"
    description = "받은 파일의 이름을 어떻게 지을지 정해요."
    text_groups = {"custom_template": ("name_template", "custom_template")}
    error_fields = {"custom_template": "custom_template"}

    def _build(self) -> None:
        self.template = QComboBox()
        for label, key in _TEMPLATE_ITEMS:
            self.template.addItem(label, key)
        self._watch(self.template, "currentIndexChanged", "name_template", "custom_template")
        self.custom_template = QLineEdit()
        self.overwrite = QCheckBox("같은 이름의 파일이 있으면 덮어쓰기")
        self._watch(self.overwrite, "toggled", "overwrite")

        self._row("파일 이름", self.template)
        self._row("직접 입력", self.custom_template, "yt-dlp 출력 템플릿 문법을 써요", error="custom_template")
        self._row("", self.overwrite, "끄면 '제목 (1).mp4' 처럼 번호를 붙여 새로 받아요")

    def _read(self) -> dict:
        return {"name_template": self.template.currentData(), "custom_template": self.custom_template.text(),
                "overwrite": self.overwrite.isChecked()}

    def _fill(self, s: AppSettings) -> None:
        self.template.setCurrentIndex(max(0, self.template.findData(s.name_template)))
        self.custom_template.setText(s.custom_template)
        self.overwrite.setChecked(s.overwrite)

    def _refresh(self, *_) -> None:
        self.custom_template.setEnabled(self.template.currentData() == "custom")


class AdvancedPage(SettingsPage):
    title = "고급"
    description = "yt-dlp 명령줄 옵션을 그대로 넣어요. 다른 설정보다 우선해요."
    text_groups = {"extra_args": ("extra_args",)}
    error_fields = {"extra_args": "extra_args"}

    def _build(self) -> None:
        self.extra_args = QLineEdit()
        self.extra_args.setPlaceholderText("예: --limit-rate 1M --no-mtime")
        self._row("yt-dlp 옵션", self.extra_args, error="extra_args")

    def _read(self) -> dict:
        return {"extra_args": self.extra_args.text().strip()}

    def _fill(self, s: AppSettings) -> None:
        self.extra_args.setText(s.extra_args)


PAGES: dict[str, type[SettingsPage]] = {
    "login": LoginPage, "network": NetworkPage, "file": FilePage, "advanced": AdvancedPage,
}
