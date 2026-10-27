"""시작할 때 새 버전을 확인하고, 있으면 업데이트 창을 띄운다."""

from __future__ import annotations

from pathlib import Path

from PySide6.QtCore import QThread, Qt, Signal
from PySide6.QtWidgets import (
    QApplication, QDialog, QHBoxLayout, QLabel, QProgressBar, QPushButton, QTextBrowser,
    QVBoxLayout,
)

from . import __version__, updater


class CheckJob(QThread):
    found = Signal(object)  # updater.Release

    def run(self) -> None:
        try:
            release = updater.fetch_latest()
        except updater.UpdateError:
            return  # 시작할 때 확인은 조용히 실패한다
        if release:
            self.found.emit(release)


class DownloadJob(QThread):
    progress = Signal(int, int)
    done = Signal(str)
    failed = Signal(str)

    def __init__(self, release):
        super().__init__()
        self.release = release

    def run(self) -> None:
        try:
            path = updater.download(self.release, lambda d, t: self.progress.emit(d, t))
        except updater.UpdateError as exc:
            self.failed.emit(str(exc))
            return
        self.done.emit(str(path))


class UpdateDialog(QDialog):
    def __init__(self, release, parent=None):
        super().__init__(parent)
        self.release = release
        self.job: DownloadJob | None = None
        self.setWindowTitle("업데이트")
        self.setMinimumWidth(420)

        col = QVBoxLayout(self)
        col.setContentsMargins(24, 20, 24, 20)
        col.setSpacing(10)
        title = QLabel(f"새 버전 {release.version} 이 나왔어요")
        title.setObjectName("crumb")
        title.setStyleSheet("font-size: 18px;")
        col.addWidget(title)
        hint = QLabel(f"지금 버전: {__version__}")
        hint.setObjectName("hint")
        col.addWidget(hint)

        if release.notes.strip():
            notes = QTextBrowser()
            notes.setMarkdown(release.notes)
            notes.setMaximumHeight(180)
            col.addWidget(notes)

        self.bar = QProgressBar()
        self.bar.setTextVisible(False)
        self.bar.hide()
        col.addWidget(self.bar)
        self.status = QLabel("")
        self.status.setObjectName("hint")
        self.status.setWordWrap(True)
        col.addWidget(self.status)

        row = QHBoxLayout()
        row.addStretch(1)
        self.later = QPushButton("나중에")
        self.later.clicked.connect(self.reject)
        row.addWidget(self.later)
        self.go = QPushButton("지금 업데이트")
        self.go.setObjectName("primary")
        self.go.clicked.connect(self.start)
        row.addWidget(self.go)
        col.addLayout(row)

        if not updater.can_self_update():
            self.go.setEnabled(False)
            self.status.setText("소스로 실행 중이라 자동 업데이트는 exe 에서만 돼요.")

    def start(self) -> None:
        self.go.setEnabled(False)
        self.later.setEnabled(False)
        self.bar.show()
        self.bar.setRange(0, 0)
        self.status.setText("받는 중…")
        self.job = DownloadJob(self.release)
        self.job.progress.connect(self._progress)
        self.job.done.connect(self._apply)
        self.job.failed.connect(self._failed)
        self.job.start()

    def _progress(self, done: int, total: int) -> None:
        if total:
            self.bar.setRange(0, total)
            self.bar.setValue(done)
            self.status.setText(f"받는 중… {done / total * 100:.0f}%")

    def _apply(self, path: str) -> None:
        self.status.setText("곧 다시 시작해요…")
        try:
            updater.launch_apply(Path(path))
        except updater.UpdateError as exc:
            self._failed(str(exc))
            return
        QApplication.quit()

    def _failed(self, msg: str) -> None:
        self.bar.hide()
        self.status.setText(msg)
        self.go.setEnabled(True)
        self.go.setText("다시 시도")
        self.later.setEnabled(True)


def check_on_startup(window) -> None:
    """창이 뜬 뒤 백그라운드로 확인한다. 새 버전이 있을 때만 창을 띄운다."""
    job = CheckJob(window)
    job.found.connect(lambda r: UpdateDialog(r, window).exec(), Qt.ConnectionType.QueuedConnection)
    window._update_check = job  # 스레드가 끝나기 전에 사라지지 않게
    job.start()
