"""⚙ 설정 창: 로그인·네트워크 / 파일 / 다운로드 / 고급."""

from __future__ import annotations

from PySide6.QtWidgets import (
    QButtonGroup, QCheckBox, QComboBox, QDialog, QFileDialog, QFormLayout, QHBoxLayout, QLabel,
    QLineEdit, QPushButton, QRadioButton, QSpinBox, QVBoxLayout, QWidget,
)

from .options import BROWSERS, AppSettings, validate_settings

_BROWSER_LABELS = {"firefox": "Firefox", "edge": "Edge", "chrome": "Chrome", "whale": "Whale",
                   "brave": "Brave", "opera": "Opera", "vivaldi": "Vivaldi", "chromium": "Chromium"}
_TEMPLATE_ITEMS = [("제목", "title"), ("제목 [ID]", "title_id"), ("업로더 - 제목", "uploader_title"),
                   ("날짜 제목", "date_title"), ("직접 입력", "custom")]


class SettingsDialog(QDialog):
    def __init__(self, settings: AppSettings, parent: QWidget | None = None):
        super().__init__(parent)
        self.setWindowTitle("설정")
        self.setMinimumWidth(460)
        self.errors: dict[str, QLabel] = {}
        col = QVBoxLayout(self)
        col.setContentsMargins(20, 18, 20, 16)
        col.setSpacing(8)

        # 로그인·네트워크
        col.addWidget(self._section("로그인·네트워크"))
        self.cookie_none = QRadioButton("사용 안 함")
        self.cookie_browser = QRadioButton("브라우저에서 가져오기")
        self.cookie_file = QRadioButton("cookies.txt 파일")
        group = QButtonGroup(self)
        for b in (self.cookie_none, self.cookie_browser, self.cookie_file):
            group.addButton(b)
            b.toggled.connect(self._refresh)
        col.addWidget(self.cookie_none)
        row = QHBoxLayout()
        row.addWidget(self.cookie_browser)
        self.browser = QComboBox()
        for key in BROWSERS:
            self.browser.addItem(_BROWSER_LABELS[key], key)
        row.addWidget(self.browser)
        row.addStretch(1)
        col.addLayout(row)
        note = QLabel("Chrome·Edge 는 켜져 있으면 쿠키를 못 읽을 수 있어요")
        note.setObjectName("hint")
        col.addWidget(note)
        row = QHBoxLayout()
        row.addWidget(self.cookie_file)
        self.cookie_path = QLineEdit()
        row.addWidget(self.cookie_path, 1)
        self.browse_btn = QPushButton("찾기")
        self.browse_btn.clicked.connect(self._browse)
        row.addWidget(self.browse_btn)
        col.addLayout(row)
        col.addWidget(self._error("cookie_file"))

        form = QFormLayout()
        self.proxy = QLineEdit()
        self.proxy.setPlaceholderText("http://127.0.0.1:8080")
        form.addRow("프록시", self.proxy)
        self.rate_limit = QLineEdit()
        self.rate_limit.setPlaceholderText("예: 5M")
        form.addRow("속도 제한", self.rate_limit)
        form.addRow("", self._error("rate_limit"))
        self.retries = QSpinBox()
        self.retries.setRange(0, 50)
        form.addRow("재시도 횟수", self.retries)
        col.addLayout(form)

        # 파일
        col.addWidget(self._section("파일"))
        form = QFormLayout()
        self.template = QComboBox()
        for label, key in _TEMPLATE_ITEMS:
            self.template.addItem(label, key)
        self.template.currentIndexChanged.connect(self._refresh)
        form.addRow("파일 이름", self.template)
        self.custom_template = QLineEdit()
        form.addRow("직접 입력", self.custom_template)
        form.addRow("", self._error("custom_template"))
        col.addLayout(form)
        self.overwrite = QCheckBox("같은 이름의 파일이 있으면 덮어쓰기")
        col.addWidget(self.overwrite)

        # 다운로드
        col.addWidget(self._section("다운로드"))
        form = QFormLayout()
        self.concurrency = QSpinBox()
        self.concurrency.setRange(1, 5)
        form.addRow("동시 다운로드", self.concurrency)
        col.addLayout(form)

        # 고급
        col.addWidget(self._section("고급"))
        self.extra_args = QLineEdit()
        self.extra_args.setPlaceholderText("예: --limit-rate 1M --no-mtime")
        col.addWidget(self.extra_args)
        note = QLabel("yt-dlp 명령줄 옵션을 그대로 씁니다. 위 설정보다 우선해요.")
        note.setObjectName("hint")
        col.addWidget(note)
        col.addWidget(self._error("extra_args"))

        buttons = QHBoxLayout()
        buttons.addStretch(1)
        cancel = QPushButton("취소")
        cancel.clicked.connect(self.reject)
        buttons.addWidget(cancel)
        self.save_btn = QPushButton("저장")
        self.save_btn.setObjectName("primary")
        self.save_btn.clicked.connect(self._save)
        buttons.addWidget(self.save_btn)
        col.addSpacing(6)
        col.addLayout(buttons)

        self._load(settings)

    def _section(self, text: str) -> QLabel:
        label = QLabel(text)
        label.setObjectName("sectionTitle")
        return label

    def _error(self, key: str) -> QLabel:
        label = QLabel()
        label.setObjectName("fieldError")
        label.setWordWrap(True)
        label.hide()
        self.errors[key] = label
        return label

    def _load(self, s: AppSettings) -> None:
        {"browser": self.cookie_browser, "file": self.cookie_file}.get(s.cookies, self.cookie_none).setChecked(True)
        self.browser.setCurrentIndex(max(0, self.browser.findData(s.cookie_browser)))
        self.cookie_path.setText(s.cookie_file)
        self.proxy.setText(s.proxy)
        self.rate_limit.setText(s.rate_limit)
        self.retries.setValue(s.retries)
        self.template.setCurrentIndex(max(0, self.template.findData(s.name_template)))
        self.custom_template.setText(s.custom_template)
        self.overwrite.setChecked(s.overwrite)
        self.concurrency.setValue(s.concurrency)
        self.extra_args.setText(s.extra_args)
        self._refresh()

    def value(self) -> AppSettings:
        cookies = "browser" if self.cookie_browser.isChecked() else "file" if self.cookie_file.isChecked() else "none"
        return AppSettings(
            cookies=cookies,
            cookie_browser=self.browser.currentData(),
            cookie_file=self.cookie_path.text().strip(),
            proxy=self.proxy.text().strip(),
            rate_limit=self.rate_limit.text().strip(),
            retries=self.retries.value(),
            name_template=self.template.currentData(),
            custom_template=self.custom_template.text(),
            overwrite=self.overwrite.isChecked(),
            concurrency=self.concurrency.value(),
            extra_args=self.extra_args.text().strip(),
        )

    def _refresh(self, *_) -> None:
        self.browser.setEnabled(self.cookie_browser.isChecked())
        file_mode = self.cookie_file.isChecked()
        self.cookie_path.setEnabled(file_mode)
        self.browse_btn.setEnabled(file_mode)
        self.custom_template.setEnabled(self.template.currentData() == "custom")

    def _browse(self) -> None:
        path, _ = QFileDialog.getOpenFileName(self, "cookies.txt 고르기", self.cookie_path.text(),
                                              "쿠키 파일 (*.txt);;모든 파일 (*)")
        if path:
            self.cookie_path.setText(path)

    def _save(self) -> None:
        problems = validate_settings(self.value())
        for key, label in self.errors.items():
            label.setText(problems.get(key, ""))
            label.setVisible(key in problems)
        if not problems:
            self.accept()
