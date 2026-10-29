"""업데이트 화면. 새 버전을 확인하고, 받아서 적용한다.

앱을 켤 때 `check()`로 뒤에서 확인하고, 새 버전이 있으면 `available_changed(True)`로
메뉴의 빨간 점만 켠다. 화면을 바꾸거나 창을 띄우지 않는다.
"""

from __future__ import annotations

from datetime import datetime
from pathlib import Path
from typing import Callable

from PySide6.QtCore import QThread, Signal
from PySide6.QtGui import QFont, QTextCharFormat, QTextCursor
from PySide6.QtWidgets import (
    QApplication, QFrame, QHBoxLayout, QLabel, QProgressBar, QPushButton, QScrollArea, QSizePolicy,
    QTextBrowser, QVBoxLayout, QWidget,
)

from . import __version__, updater

# 메뉴에 빨간 점을 켜 두는 상태. 새 버전이 있다는 것만은 변하지 않는다.
_DOT_STATES = ("available", "downloading", "download_failed", "applying")


def _quit() -> None:
    QApplication.quit()


class CheckJob(QThread):
    found = Signal(object)  # updater.Release
    latest = Signal()
    failed = Signal(str)

    def run(self) -> None:
        try:
            release = updater.fetch_latest()
        except updater.UpdateError as exc:
            self.failed.emit(str(exc))
            return
        if release:
            self.found.emit(release)
        else:
            self.latest.emit()


class SetupDownloadJob(QThread):
    """설치 파일 받기. 영상 받기(`downloader.DownloadJob`)와 다르다."""

    progress = Signal(int, int)
    done = Signal(str)
    failed = Signal(str)

    def __init__(self, release, parent=None):
        super().__init__(parent)
        self.release = release

    def run(self) -> None:
        try:
            path = updater.download(self.release, self.progress.emit)
        except updater.UpdateError as exc:
            self.failed.emit(str(exc))
            return
        self.done.emit(str(path))


class UpdatePage(QWidget):
    available_changed = Signal(bool)

    def __init__(self, confirm: Callable[[], bool] | None = None, parent: QWidget | None = None):
        super().__init__(parent)
        self._confirm = confirm
        self.state = ""
        self.release: updater.Release | None = None
        self.check_job: CheckJob | None = None
        self.download_job: SetupDownloadJob | None = None

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
        heading = QLabel("업데이트")
        heading.setObjectName("crumb")
        col.addWidget(heading)
        col.addSpacing(4)
        desc = QLabel(f"지금 버전 {__version__} · 앱을 켤 때 새 버전을 확인해요.")
        desc.setObjectName("viewHint")
        col.addWidget(desc)
        col.addSpacing(16)

        card = QFrame()
        card.setObjectName("settingsCard")
        # 숨긴 칸은 간격도 차지하지 않도록 고정 간격 대신 레이아웃 간격을 쓴다
        box = QVBoxLayout(card)
        box.setContentsMargins(20, 18, 20, 18)
        box.setSpacing(14)
        head = QVBoxLayout()
        head.setSpacing(2)
        self.title = QLabel()
        self.title.setObjectName("updateTitle")
        head.addWidget(self.title)
        self.sub = QLabel()
        self.sub.setWordWrap(True)
        head.addWidget(self.sub)
        box.addLayout(head)
        self.notes = QTextBrowser()
        self.notes.setObjectName("updateNotes")
        self.notes.setOpenExternalLinks(True)
        box.addWidget(self.notes)
        self.bar = QProgressBar()
        self.bar.setTextVisible(False)
        box.addWidget(self.bar)
        row = QHBoxLayout()
        row.setSpacing(10)
        self.action = QPushButton()
        self.action.setSizePolicy(QSizePolicy.Policy.Fixed, QSizePolicy.Policy.Fixed)
        self.action.clicked.connect(self._on_action)
        row.addWidget(self.action)
        self.status = QLabel()
        self.status.setWordWrap(True)
        row.addWidget(self.status, 1)
        row.addStretch(0)  # 안내 줄이 숨어도 단추가 가운데로 가지 않게
        box.addLayout(row)
        col.addWidget(card)
        col.addStretch(1)

        self._set_state("idle")

    # ---------- 상태 ----------
    def _set_state(self, state: str, message: str = "") -> None:
        had_dot = self.state in _DOT_STATES
        self.state = state
        new = state in _DOT_STATES
        release = self.release
        if state == "idle":
            self._show("아직 확인하지 않았어요", "", "업데이트 확인")
        elif state == "checking":
            self._show("확인하는 중…", "최신 버전을 알아보고 있어요", "업데이트 확인", enabled=False)
        elif state == "latest":
            self._show("최신 버전이에요", f"{datetime.now():%H:%M} 에 확인했어요", "업데이트 확인")
        elif state == "check_failed":
            self._show("확인하지 못했어요", message, "다시 확인", sub_error=True)
        else:
            title = f"새 버전 {release.version} 이 나왔어요"
            if state == "available":
                if updater.can_self_update():
                    self._show(title, "", "지금 업데이트", primary=True,
                               status="받고 나면 QuickMedia 가 잠깐 꺼졌다 다시 켜져요.")
                else:
                    self._show(title, "", "지금 업데이트", primary=True, enabled=False,
                               status="소스로 실행 중이라 자동 업데이트는 exe 에서만 돼요.")
            elif state == "downloading":
                self._show(title, "", "지금 업데이트", primary=True, enabled=False, status="받는 중…", bar=True)
            elif state == "download_failed":
                self._show(title, "", "다시 시도", primary=True, status=message, status_error=True)
            elif state == "applying":
                self._show(title, "", "지금 업데이트", primary=True, enabled=False, status="곧 다시 시작해요…")
            self.notes.setVisible(bool(release.notes.strip()))
        if new != had_dot:
            self.available_changed.emit(new)

    def _show(self, title: str, sub: str, action: str, *, enabled: bool = True, primary: bool = False,
              status: str = "", bar: bool = False, sub_error: bool = False, status_error: bool = False) -> None:
        self.title.setText(title)
        self.sub.setText(sub)
        self.sub.setVisible(bool(sub))
        self._style(self.sub, "fieldError" if sub_error else "hint")
        self.notes.hide()
        self.bar.setVisible(bar)
        if bar:
            self.bar.setRange(0, 0)
        self.status.setText(status)
        self.status.setVisible(bool(status))
        self._style(self.status, "fieldError" if status_error else "hint")
        self.action.setText(action)
        self.action.setEnabled(enabled)
        self._style(self.action, "primary" if primary else "")

    @staticmethod
    def _style(widget: QWidget, name: str) -> None:
        if widget.objectName() != name:
            widget.setObjectName(name)
            widget.style().unpolish(widget)
            widget.style().polish(widget)

    # ---------- 확인 ----------
    def check(self) -> None:
        if self.check_job is not None and self.check_job.isRunning():
            return
        self._set_state("checking")
        self.check_job = CheckJob(self)
        self.check_job.found.connect(self._on_found)
        self.check_job.latest.connect(self._on_latest)
        self.check_job.failed.connect(self._on_check_failed)
        self.check_job.start()

    def _on_found(self, release) -> None:
        self.release = release
        self._set_notes(release.notes)
        self._set_state("available")

    def _set_notes(self, markdown: str) -> None:
        """릴리스 설명. 마크다운 제목은 Qt 기본값이 너무 커서 본문보다 조금 굵은 13px 로 줄인다."""
        self.notes.setMarkdown(markdown)
        doc = self.notes.document()
        fmt = QTextCharFormat()
        fmt.setFontWeight(QFont.Weight.DemiBold)
        fmt.setProperty(QTextCharFormat.Property.FontPixelSize, 13)
        block = doc.firstBlock()
        while block.isValid():
            if block.blockFormat().headingLevel():
                cursor = QTextCursor(block)
                cursor.select(QTextCursor.SelectionType.BlockUnderCursor)
                cursor.setCharFormat(fmt)  # 덮어써야 마크다운이 붙인 point 크기가 빠진다
            block = block.next()
        self._fit_notes()

    def _fit_notes(self) -> None:
        """내용만큼만 높이를 잡는다(최대 220, 넘치면 그 안에서 스크롤)."""
        doc = self.notes.document()
        doc.setTextWidth(self.notes.viewport().width())
        frame = self.notes.height() - self.notes.viewport().height()
        self.notes.setFixedHeight(min(220, int(doc.size().height()) + frame))

    def resizeEvent(self, e) -> None:
        super().resizeEvent(e)
        if self.release is not None:
            self._fit_notes()

    def _on_latest(self) -> None:
        self._set_state("latest")

    def _on_check_failed(self, msg: str) -> None:
        self._set_state("check_failed", msg)

    # ---------- 받기·적용 ----------
    def _on_action(self) -> None:
        if self.state in ("idle", "latest", "check_failed"):
            self.check()
        elif self.state in ("available", "download_failed"):
            if self._confirm is None or self._confirm():
                self._start_download()

    def _start_download(self) -> None:
        self._set_state("downloading")
        self.download_job = SetupDownloadJob(self.release, self)
        self.download_job.progress.connect(self._on_progress)
        self.download_job.done.connect(self._on_downloaded)
        self.download_job.failed.connect(self._on_download_failed)
        self.download_job.start()

    def _on_progress(self, done: int, total: int) -> None:
        if total:
            self.bar.setRange(0, total)
            self.bar.setValue(done)
            self.status.setText(f"받는 중… {done / total * 100:.0f}%")

    def _on_downloaded(self, path: str) -> None:
        self._set_state("applying")
        try:
            updater.launch_apply(Path(path))
        except updater.UpdateError as exc:
            self._set_state("download_failed", str(exc))
            return
        _quit()

    def _on_download_failed(self, msg: str) -> None:
        self._set_state("download_failed", msg)

    def stop(self) -> None:
        """창을 닫을 때. 돌고 있는 스레드를 멈춘다."""
        for job in (self.check_job, self.download_job):
            if job is not None and job.isRunning():
                job.terminate()
                job.wait(2000)
