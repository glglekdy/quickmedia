"""메인 창: 링크 입력 · 형식 고르기 · 세부 옵션 · 다운로드 목록 · 저장 위치 독."""

from __future__ import annotations

import dataclasses
import os
from dataclasses import dataclass
from pathlib import Path

from PySide6.QtCore import QEasingCurve, QPropertyAnimation, QSettings, QSize, QStandardPaths, Qt
from PySide6.QtGui import QGuiApplication, QIcon, QPixmap
from PySide6.QtWidgets import (
    QButtonGroup, QDialog, QFileDialog, QFrame, QHBoxLayout, QLabel, QLineEdit, QMainWindow,
    QMessageBox, QProgressBar, QPushButton, QScrollArea, QVBoxLayout, QWidget,
)

from .app_info import NAME, SETTINGS
from .downloader import FFMPEG_DIR, DownloadJob, PlaylistResult
from .job_queue import JobQueue
from .options import (
    PRESET_LABELS, AppSettings, DownloadOptions, PlaylistContext, apply_preset, changed_summary, describe,
    matching_preset, options_from_dict, settings_from_dict, to_dict, to_ydl_opts, validate_options,
)
from .options_panel import OptionsPanel
from .settings_dialog import SettingsDialog
from .theme import ASSETS
from .site import SiteIcons, display_name, domain_of

# 이보다 많은 영상을 한 번에 넣기 전에는 물어본다
CONFIRM_OVER = 200
# 끝나서 "끝난 항목 지우기"로 지워도 되는 카드 상태
FINISHED_STATES = ("done", "error", "skipped", "playlist")


@dataclass
class Snapshot:
    """다운로드를 누른 순간의 값. 대기 중에 패널을 바꿔도 이미 넣은 항목은 그대로 받는다."""
    opts: DownloadOptions
    settings: AppSettings
    folder: str


class JobCard(QFrame):
    """다운로드 한 건. 사이트 · 제목 · 상태 · 진행률."""

    def __init__(self, url: str, fmt_label: str, icons: SiteIcons):
        super().__init__()
        self.setObjectName("card")
        self.path = ""
        self.icons = icons
        self.host = domain_of(url)
        col = QVBoxLayout(self)
        col.setContentsMargins(14, 10, 14, 12)
        col.setSpacing(6)

        site = QHBoxLayout()
        site.setSpacing(6)
        self.site_icon = QLabel()
        self.site_icon.setFixedSize(16, 16)
        self.site_icon.setPixmap(icons.icon(self.host))
        site.addWidget(self.site_icon)
        self.site_name = QLabel(self.host)
        self.site_name.setObjectName("cardSite")
        site.addWidget(self.site_name)
        site.addStretch(1)
        col.addLayout(site)
        icons.ready.connect(self._icon_ready)

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

    def _icon_ready(self, host: str, pix: QPixmap) -> None:
        if host == self.host:
            self.site_icon.setPixmap(pix)

    def set_site(self, extractor_key: str, webpage_url: str) -> None:
        host = domain_of(webpage_url) or self.host
        if host != self.host:
            self.host = host
            self.site_icon.setPixmap(self.icons.icon(host))
        self.site_name.setText(display_name(extractor_key, host))

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

    def _finish_quietly(self, state: str, text: str) -> None:
        self.percent.setText("")
        self.bar.hide()
        self._set_state(state, text)

    def set_skipped(self) -> None:
        self._finish_quietly("skipped", "이미 받은 영상이라 건너뛰었어요")

    def set_cancelled(self) -> None:
        self._finish_quietly("skipped", "취소했어요")

    def set_playlist(self, title: str, added: int, skipped: int) -> None:
        self.set_title(f"재생목록 · {title}")
        text = f"영상 {added}개 추가"
        if skipped:
            text += f" · {skipped}개는 이미 받아서 건너뜀"
        self._finish_quietly("playlist", text)

    def _reveal(self) -> None:
        # 탐색기에서 파일을 선택한 채로 연다. 경로를 모르면 폴더만.
        p = Path(self.path)
        if self.path and p.exists():
            os.system(f'explorer /select,"{p}"')
        elif self.path:
            os.startfile(p.parent)


class MainWindow(QMainWindow):
    def __init__(self, settings: QSettings | None = None, data_dir: Path | None = None):
        super().__init__()
        self.setWindowTitle(NAME)
        self.resize(640, 680)
        self.setMinimumSize(520, 480)
        self.settings = settings if settings is not None else QSettings(*SETTINGS)
        if data_dir is None:
            data_dir = Path(QStandardPaths.writableLocation(QStandardPaths.StandardLocation.AppDataLocation))
        self.data_dir = data_dir
        self.archive = data_dir / "archive.txt"
        self.icons = SiteIcons(data_dir / "favicons")
        self.folder = self.settings.value("folder", str(Path.home() / "Downloads"))
        self.opts = options_from_dict(self._read_group("options", DownloadOptions),
                                      legacy_format=self.settings.value("format"))
        self.app_settings = settings_from_dict(self._read_group("settings", AppSettings))
        self.icons.proxy = self.app_settings.proxy
        self.jobs: JobQueue[DownloadJob] = JobQueue(self.app_settings.concurrency, self._start_job)
        self._cards: dict[DownloadJob, JobCard] = {}
        self._closing = False
        self._build_ui()
        self.panel.set_options(self.opts)
        self._sync_format_row()

    # ---------- 저장 ----------
    def _read_group(self, group: str, cls) -> dict:
        keys = (f.name for f in dataclasses.fields(cls))
        return {k: self.settings.value(f"{group}/{k}") for k in keys if self.settings.contains(f"{group}/{k}")}

    def _write_group(self, group: str, obj) -> None:
        for k, v in to_dict(obj).items():
            self.settings.setValue(f"{group}/{k}", v)

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
        title_row = QHBoxLayout()
        crumb = QLabel(NAME)
        crumb.setObjectName("crumb")
        title_row.addWidget(crumb)
        title_row.addStretch(1)
        # 기호 글자(⚙ ▾)는 글꼴에 따라 네모로 나와서 그림을 쓴다
        gear = QPushButton()
        gear.setIcon(QIcon(str(ASSETS / "gear.svg")))
        gear.setIconSize(QSize(18, 18))
        gear.setObjectName("ghost")
        gear.setToolTip("설정")
        gear.clicked.connect(self.open_settings)
        title_row.addWidget(gear)
        head.addLayout(title_row)
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
        self.url.setPlaceholderText("영상·재생목록 링크를 붙여넣으세요 (유튜브 외 사이트도 돼요)")
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
        for key, label in PRESET_LABELS.items():
            btn = QPushButton(label)
            btn.setObjectName("segButton")
            btn.setCheckable(True)
            btn.setProperty("fmt", key)
            self.fmt_group.addButton(btn)
            seg_row.addWidget(btn)
        self.fmt_group.buttonClicked.connect(self._preset_clicked)
        row2.addWidget(seg)
        self.more_btn = QPushButton()
        self.more_btn.setObjectName("ghost")
        self.more_btn.setLayoutDirection(Qt.LayoutDirection.RightToLeft)  # 화살표를 글자 뒤에
        self.more_btn.clicked.connect(self._toggle_panel)
        row2.addWidget(self.more_btn)
        row2.addStretch(1)
        self.download_btn = QPushButton("다운로드")
        self.download_btn.setObjectName("primary")
        self.download_btn.clicked.connect(self.add_job)
        row2.addWidget(self.download_btn)
        form.addLayout(row2)

        self.form_error = QLabel()
        self.form_error.setObjectName("fieldError")
        self.form_error.hide()
        form.addWidget(self.form_error)

        self.panel = OptionsPanel()
        self.panel.changed.connect(self._panel_changed)
        self.panel.setVisible(str(self.settings.value("ui/panel_open", "false")).lower() == "true")
        form.addWidget(self.panel)
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

    def _sync_format_row(self) -> None:
        """프리셋 버튼 선택과 세부 옵션 버튼 글자를 지금 값에 맞춘다."""
        key = matching_preset(self.opts)
        if key is None:
            # 배타 그룹은 모두 해제할 수 없어 잠깐 푼다
            self.fmt_group.setExclusive(False)
            for b in self.fmt_group.buttons():
                b.setChecked(False)
            self.fmt_group.setExclusive(True)
        else:
            for b in self.fmt_group.buttons():
                if b.property("fmt") == key:
                    b.setChecked(True)
        arrow = "arrow-down.svg" if self.panel.isHidden() else "arrow-up.svg"
        self.more_btn.setIcon(QIcon(str(ASSETS / arrow)))
        text = "세부 옵션"
        changed = changed_summary(self.opts)
        if changed:
            text += " · " + ", ".join(changed)
        self.more_btn.setText(text)

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

    def open_settings(self) -> None:
        dialog = SettingsDialog(self.app_settings, self)
        if dialog.exec() == QDialog.DialogCode.Accepted:
            self.app_settings = dialog.value()
            self._write_group("settings", self.app_settings)
            self.jobs.set_limit(self.app_settings.concurrency)
            self.icons.proxy = self.app_settings.proxy

    def _toggle_panel(self) -> None:
        show = self.panel.isHidden()
        self.panel.setVisible(show)
        self.settings.setValue("ui/panel_open", show)
        self._sync_format_row()

    def _preset_clicked(self, btn: QPushButton) -> None:
        self.opts = apply_preset(self.opts, btn.property("fmt"))
        self.panel.set_options(self.opts)
        self._write_group("options", self.opts)
        self._sync_format_row()
        self.form_error.hide()

    def _panel_changed(self) -> None:
        self.opts = self.panel.options()
        self._write_group("options", self.opts)
        self._sync_format_row()
        self.form_error.hide()

    def add_job(self) -> None:
        url = self.url.text().strip()
        if not url:
            self.url.setFocus()
            self.url.setPlaceholderText("먼저 링크를 넣어 주세요")
            return
        problems = validate_options(self.opts)
        if problems:
            self.form_error.setText(next(iter(problems.values())))
            self.form_error.show()
            return
        self.form_error.hide()
        self._write_group("options", self.opts)
        snap = Snapshot(dataclasses.replace(self.opts), dataclasses.replace(self.app_settings), self.folder)
        self._enqueue(url, snap, insert_at=1)
        self.url.clear()

    def _enqueue(self, url: str, snap: Snapshot, insert_at: int,
                 playlist: PlaylistContext | None = None) -> JobCard:
        opts = to_ydl_opts(snap.opts, snap.settings, folder=snap.folder, archive=str(self.archive),
                           ffmpeg_dir=FFMPEG_DIR, playlist=playlist)
        if snap.opts.skip_archived:
            self.data_dir.mkdir(parents=True, exist_ok=True)
        card = JobCard(url, describe(snap.opts), self.icons)
        self.empty.hide()
        self.list.insertWidget(insert_at, card)

        job = DownloadJob(url, opts)
        job.snapshot = snap
        job.titled.connect(card.set_title)
        job.progress.connect(card.set_progress)
        job.finished_ok.connect(card.set_done)
        job.failed.connect(card.set_failed)
        job.site_found.connect(card.set_site)
        job.skipped.connect(card.set_skipped)
        # 작업 스레드에서 오는 신호는 메인 창 메서드로 받아야 메인 스레드에서 처리된다
        job.expanded.connect(self._job_expanded)
        job.finished.connect(self._job_finished)
        self._cards[job] = card
        self.jobs.add(job)
        return card

    def _start_job(self, job: DownloadJob) -> None:
        self._cards[job].set_progress(0, "영상 정보 불러오는 중…")
        job.start()

    def _job_expanded(self, result: PlaylistResult) -> None:
        job = self.sender()
        if job in self._cards:
            self._on_expanded(self._cards[job], job, result)

    def _job_finished(self) -> None:
        if self._closing:
            return  # 닫는 중에 끝난 작업 자리에 대기 중인 작업을 시작하지 않는다
        job = self.sender()
        self.jobs.done(job)
        self._cards.pop(job, None)

    def _confirm_many(self, n: int) -> bool:
        answer = QMessageBox.question(self, "재생목록", f"영상 {n}개를 모두 받을까요?")
        return answer == QMessageBox.StandardButton.Yes

    def _on_expanded(self, card: JobCard, job: DownloadJob, result: PlaylistResult) -> None:
        entries = result.entries
        if len(entries) > CONFIRM_OVER and not self._confirm_many(len(entries)):
            card.set_cancelled()
            return
        card.set_site(result.site_key, result.webpage_url)
        card.set_playlist(result.title, len(entries), result.skipped)
        at = self.list.indexOf(card) + 1
        for i, entry in enumerate(entries):
            ctx = PlaylistContext(result.title, entry.index, len(entries))
            child = self._enqueue(entry.url, job.snapshot, insert_at=at + i, playlist=ctx)
            child.set_title(entry.title)

    def clear_finished(self) -> None:
        for i in reversed(range(self.list.count())):
            w = self.list.itemAt(i).widget()
            if isinstance(w, JobCard) and w.status.property("state") in FINISHED_STATES:
                w.deleteLater()
                self.list.takeAt(i)
        if not any(isinstance(self.list.itemAt(i).widget(), JobCard) for i in range(self.list.count())):
            self.empty.show()

    def closeEvent(self, e) -> None:
        # Windows 의 terminate() 는 finished 를 바로 내보내므로 먼저 표시해 둔다
        self._closing = True
        for job in self.jobs.running:
            if job.isRunning():
                job.terminate()
                job.wait(2000)
        super().closeEvent(e)
