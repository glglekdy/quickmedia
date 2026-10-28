"""메인 창: 링크 입력 · 형식 고르기 · 다운로드 목록 · 저장 위치 독."""

from __future__ import annotations

import os
from pathlib import Path

from PySide6.QtCore import QEasingCurve, QPropertyAnimation, QSettings, Qt
from PySide6.QtGui import QGuiApplication
from PySide6.QtWidgets import (
    QButtonGroup, QFileDialog, QFrame, QHBoxLayout, QLabel, QLineEdit, QMainWindow,
    QProgressBar, QPushButton, QScrollArea, QVBoxLayout, QWidget,
)

from .app_info import NAME, SETTINGS
from .downloader import FORMATS, DownloadJob


class JobCard(QFrame):
    """다운로드 한 건. 제목 · 상태 · 진행률."""

    def __init__(self, url: str, fmt_label: str):
        super().__init__()
        self.setObjectName("card")
        self.path = ""
        col = QVBoxLayout(self)
        col.setContentsMargins(14, 12, 14, 12)
        col.setSpacing(6)

        top = QHBoxLayout()
        self.title = QLabel(url)
        self.title.setObjectName("cardTitle")
        self.title.setMinimumWidth(0)
        top.addWidget(self.title, 1)
        self.percent = QLabel("")
        self.percent.setObjectName("percent")
        # 숫자 자릿수가 바뀌어도 옆 글자가 흔들리지 않게 폭 고정
        self.percent.setFixedWidth(40)
        self.percent.setAlignment(Qt.AlignmentFlag.AlignRight | Qt.AlignmentFlag.AlignVCenter)
        top.addWidget(self.percent)
        self.open_btn = QPushButton("열기")
        self.open_btn.setObjectName("ghost")
        self.open_btn.hide()
        self.open_btn.clicked.connect(self._reveal)
        top.addWidget(self.open_btn)
        col.addLayout(top)

        self.bar = QProgressBar()
        self.bar.setRange(0, 1000)
        self.bar.setTextVisible(False)
        col.addWidget(self.bar)
        # 값이 뚝뚝 바뀌지 않고 미끄러지듯 따라가게
        self.anim = QPropertyAnimation(self.bar, b"value", self)
        self.anim.setDuration(300)
        self.anim.setEasingCurve(QEasingCurve.Type.OutCubic)

        self.status = QLabel(f"{fmt_label} · 대기 중")
        self.status.setObjectName("cardStatus")
        col.addWidget(self.status)
        self._fmt_label = fmt_label

    def _set_state(self, state: str, text: str) -> None:
        self.status.setProperty("state", state)
        self.status.setText(text)
        self.status.style().unpolish(self.status)
        self.status.style().polish(self.status)

    def set_title(self, title: str) -> None:
        self.title.setText(self.title.fontMetrics().elidedText(title, Qt.TextElideMode.ElideRight, 380))
        self.title.setToolTip(title)

    def _animate_to(self, value: int) -> None:
        if value < self.bar.value():
            self.anim.stop()
            self.bar.setValue(value)
            return
        self.anim.stop()
        self.anim.setStartValue(self.bar.value())
        self.anim.setEndValue(value)
        self.anim.start()

    def set_progress(self, pct: float, text: str) -> None:
        self._animate_to(int(pct * 10))
        self.percent.setText(f"{pct:.0f}%")
        self._set_state("", f"{self._fmt_label} · {text}")

    def set_done(self, path: str) -> None:
        self.path = path
        self._animate_to(1000)
        self.percent.setText("")
        self._set_state("done", f"{self._fmt_label} · 완료")
        self.open_btn.show()

    def set_failed(self, msg: str) -> None:
        self.percent.setText("")
        self.bar.hide()
        self._set_state("error", f"실패 · {msg}")

    def _reveal(self) -> None:
        # 탐색기에서 파일을 선택한 채로 연다. 경로를 모르면 폴더만.
        p = Path(self.path)
        if self.path and p.exists():
            os.system(f'explorer /select,"{p}"')
        elif self.path:
            os.startfile(p.parent)


class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle(NAME)
        self.resize(640, 620)
        self.setMinimumSize(520, 480)
        self.settings = QSettings(*SETTINGS)
        self.folder = self.settings.value("folder", str(Path.home() / "Downloads"))
        self.queue: list[tuple[DownloadJob, JobCard]] = []
        self.running: DownloadJob | None = None
        self._build_ui()

    # ---------- 화면 ----------
    def _build_ui(self) -> None:
        main = QWidget()
        main.setObjectName("mainArea")
        root = QVBoxLayout(main)
        root.setContentsMargins(0, 0, 0, 0)
        root.setSpacing(0)
        self.setCentralWidget(main)

        head = QVBoxLayout()
        head.setContentsMargins(28, 24, 28, 8)
        head.setSpacing(4)
        crumb = QLabel(NAME)
        crumb.setObjectName("crumb")
        head.addWidget(crumb)
        hint = QLabel("링크를 붙여넣고 형식을 고른 뒤 다운로드를 누르세요.")
        hint.setObjectName("viewHint")
        head.addWidget(hint)
        root.addLayout(head)

        form = QVBoxLayout()
        form.setContentsMargins(28, 12, 28, 16)
        form.setSpacing(10)

        row = QHBoxLayout()
        row.setSpacing(6)
        self.url = QLineEdit()
        self.url.setPlaceholderText("https://www.youtube.com/watch?v=…")
        self.url.returnPressed.connect(self.add_job)
        row.addWidget(self.url, 1)
        paste = QPushButton("붙여넣기")
        paste.clicked.connect(self.paste)
        row.addWidget(paste)
        form.addLayout(row)

        row2 = QHBoxLayout()
        seg = QFrame()
        seg.setObjectName("segment")
        seg_row = QHBoxLayout(seg)
        seg_row.setContentsMargins(3, 3, 3, 3)
        seg_row.setSpacing(2)
        self.fmt_group = QButtonGroup(self)
        saved_fmt = self.settings.value("format", "best")
        for key, (label, _) in FORMATS.items():
            btn = QPushButton(label)
            btn.setObjectName("segButton")
            btn.setCheckable(True)
            btn.setProperty("fmt", key)
            btn.setChecked(key == saved_fmt)
            self.fmt_group.addButton(btn)
            seg_row.addWidget(btn)
        if not self.fmt_group.checkedButton():
            self.fmt_group.buttons()[0].setChecked(True)
        row2.addWidget(seg)
        row2.addStretch(1)
        self.download_btn = QPushButton("다운로드")
        self.download_btn.setObjectName("primary")
        self.download_btn.clicked.connect(self.add_job)
        row2.addWidget(self.download_btn)
        form.addLayout(row2)
        root.addLayout(form)

        list_head = QHBoxLayout()
        list_head.setContentsMargins(28, 4, 28, 6)
        title = QLabel("다운로드 목록")
        title.setObjectName("sectionTitle")
        list_head.addWidget(title)
        list_head.addStretch(1)
        self.clear_btn = QPushButton("끝난 항목 지우기")
        self.clear_btn.setObjectName("ghost")
        self.clear_btn.clicked.connect(self.clear_finished)
        list_head.addWidget(self.clear_btn)
        root.addLayout(list_head)

        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        holder = QWidget()
        holder.setObjectName("queueHolder")
        self.list = QVBoxLayout(holder)
        self.list.setContentsMargins(28, 0, 28, 16)
        self.list.setSpacing(8)
        self.empty = QLabel("아직 받은 영상이 없어요")
        self.empty.setObjectName("emptyState")
        self.empty.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.list.addWidget(self.empty)
        self.list.addStretch(1)
        scroll.setWidget(holder)
        root.addWidget(scroll, 1)

        dock = QFrame()
        dock.setObjectName("dock")
        d = QHBoxLayout(dock)
        d.setContentsMargins(28, 12, 28, 12)
        d.setSpacing(6)
        lbl = QLabel("저장 위치")
        lbl.setObjectName("hint")
        d.addWidget(lbl)
        self.folder_edit = QLineEdit(self.folder)
        self.folder_edit.setReadOnly(True)
        d.addWidget(self.folder_edit, 1)
        change = QPushButton("변경")
        change.clicked.connect(self.pick_folder)
        d.addWidget(change)
        open_btn = QPushButton("폴더 열기")
        open_btn.clicked.connect(lambda: os.startfile(self.folder))
        d.addWidget(open_btn)
        root.addWidget(dock)

    # ---------- 동작 ----------
    def paste(self) -> None:
        text = QGuiApplication.clipboard().text().strip()
        if text:
            self.url.setText(text)
            self.url.setFocus()

    def pick_folder(self) -> None:
        d = QFileDialog.getExistingDirectory(self, "저장 위치 고르기", self.folder)
        if d:
            self.folder = d
            self.folder_edit.setText(d)
            self.settings.setValue("folder", d)

    def add_job(self) -> None:
        url = self.url.text().strip()
        if not url:
            self.url.setFocus()
            self.url.setPlaceholderText("먼저 유튜브 링크를 넣어 주세요")
            return
        fmt = self.fmt_group.checkedButton().property("fmt")
        self.settings.setValue("format", fmt)
        card = JobCard(url, FORMATS[fmt][0])
        self.empty.hide()
        self.list.insertWidget(1, card)  # 새 항목이 위로

        job = DownloadJob(url, fmt, self.folder)
        job.titled.connect(card.set_title)
        job.progress.connect(card.set_progress)
        job.finished_ok.connect(card.set_done)
        job.failed.connect(card.set_failed)
        job.finished.connect(self._next)
        self.queue.append((job, card))
        self.url.clear()
        if self.running is None:
            self._next()

    def _next(self) -> None:
        self.running = None
        if self.queue:
            job, card = self.queue.pop(0)
            card.set_progress(0, "영상 정보 불러오는 중…")
            self.running = job
            job.start()

    def clear_finished(self) -> None:
        for i in reversed(range(self.list.count())):
            w = self.list.itemAt(i).widget()
            if isinstance(w, JobCard) and w.status.property("state") in ("done", "error"):
                w.deleteLater()
                self.list.takeAt(i)
        if not any(isinstance(self.list.itemAt(i).widget(), JobCard) for i in range(self.list.count())):
            self.empty.show()

    def closeEvent(self, e) -> None:
        if self.running and self.running.isRunning():
            self.running.terminate()
            self.running.wait(2000)
        super().closeEvent(e)
