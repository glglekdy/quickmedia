import pytest
from PySide6.QtCore import QSettings

from ytdl import main_window as mw
from ytdl.downloader import DownloadJob, PlaylistEntry, PlaylistResult
from ytdl.options import DownloadOptions
from ytdl.site import SiteIcons


@pytest.fixture
def win(qapp, tmp_path, monkeypatch):
    monkeypatch.setattr(DownloadJob, "start", lambda self: None)  # 네트워크 없이
    monkeypatch.setattr(mw, "SiteIcons", lambda d: SiteIcons(d, fetch=lambda u: None))
    s = QSettings(str(tmp_path / "s.ini"), QSettings.Format.IniFormat)
    s.setValue("folder", str(tmp_path))
    w = mw.MainWindow(settings=s, data_dir=tmp_path / "data")
    yield w
    w.jobs = mw.JobQueue(1, lambda j: None)  # closeEvent 가 가짜 작업을 건드리지 않게


def cards(w):
    return [w.list.itemAt(i).widget() for i in range(w.list.count()) if isinstance(w.list.itemAt(i).widget(), mw.JobCard)]


def test_legacy_format_is_migrated(qapp, tmp_path):
    s = QSettings(str(tmp_path / "old.ini"), QSettings.Format.IniFormat)
    s.setValue("format", "mp3")
    w = mw.MainWindow(settings=s, data_dir=tmp_path)
    assert w.opts.kind == "audio" and w.fmt_group.checkedButton().property("fmt") == "mp3"


def test_add_job_snapshots_options(win, tmp_path):
    win.url.setText("https://www.youtube.com/watch?v=abc")
    win.add_job()
    (card,) = cards(win)
    (job,) = win.jobs.running
    assert job.ydl_opts["paths"] == {"home": str(tmp_path)} and job.snapshot.opts == win.opts
    assert card.site_name.text() == "youtube.com"
    assert win.settings.value("options/kind") == "video"


def test_invalid_section_blocks_add(win):
    win.panel.set_options(DownloadOptions(section_start="abc"))
    win._panel_changed()
    win.url.setText("https://x.com/v")
    win.add_job()
    assert cards(win) == [] and win.form_error.text() == "시간은 1:30 처럼 써 주세요"


def test_concurrency_from_settings(win):
    win.app_settings.concurrency = 2
    win.jobs.set_limit(2)
    for i in range(3):
        win.url.setText(f"https://x.com/{i}"); win.add_job()
    assert len(win.jobs.running) == 2 and len(win.jobs.pending) == 1


def test_expand_playlist(win):
    win.panel.set_options(DownloadOptions(playlist="all", number_prefix=True)); win._panel_changed()
    win.url.setText("https://www.youtube.com/playlist?list=P"); win.add_job()
    (card,), (job,) = cards(win), win.jobs.running
    result = PlaylistResult("내 목록", "YoutubeTab", "https://www.youtube.com/playlist?list=P",
                            [PlaylistEntry(f"https://y/{i}", f"t{i}", i) for i in (1, 2, 3)], skipped=2)
    win._on_expanded(card, job, result)
    cs = cards(win)
    assert cs[0] is card and len(cs) == 4
    assert card.title.text() == "재생목록 · 내 목록" and card.status.text() == "영상 3개 추가 · 2개는 이미 받아서 건너뜀"
    new_jobs = [j for j in win.jobs.running + win.jobs.pending if j is not job]
    assert [j.url for j in new_jobs] == ["https://y/1", "https://y/2", "https://y/3"]
    assert new_jobs[0].ydl_opts["outtmpl"]["default"] == "01 - %(title)s.%(ext)s"
    assert new_jobs[0].ydl_opts["noplaylist"] is True


def test_expand_many_asks_and_can_cancel(win, monkeypatch):
    asked = []
    monkeypatch.setattr(win, "_confirm_many", lambda n: asked.append(n) or False)
    win.url.setText("https://x.com/list"); win.add_job()
    (card,), (job,) = cards(win), win.jobs.running
    win._on_expanded(card, job, PlaylistResult("big", "", "", [PlaylistEntry(f"u{i}", "", i) for i in range(201)], 0))
    assert asked == [201] and len(cards(win)) == 1 and card.status.text() == "취소했어요"


def test_clear_finished_keeps_running(win):
    for i in range(4):
        win.url.setText(f"https://x.com/{i}"); win.add_job()
    c = cards(win)
    c[0].set_done(""); c[1].set_skipped(); c[2].set_playlist("p", 0, 0)
    win.clear_finished()
    assert cards(win) == [c[3]]


def test_close_stops_every_running_job(win, monkeypatch):
    stopped = []
    monkeypatch.setattr(DownloadJob, "isRunning", lambda self: True)
    monkeypatch.setattr(DownloadJob, "terminate", lambda self: stopped.append(self))
    monkeypatch.setattr(DownloadJob, "wait", lambda self, ms=0: True)
    win.jobs.set_limit(3)
    for i in range(3):
        win.url.setText(f"https://x.com/{i}"); win.add_job()
    running = win.jobs.running
    win.close()
    assert stopped == running


def test_close_does_not_start_waiting_jobs(win, monkeypatch):
    # Windows 에서 terminate() 는 finished 를 바로 내보낸다. 그때 대기 중인 작업이 시작되면 안 된다.
    started = []
    monkeypatch.setattr(DownloadJob, "start", lambda self: started.append(self))
    monkeypatch.setattr(DownloadJob, "isRunning", lambda self: True)
    monkeypatch.setattr(DownloadJob, "terminate", lambda self: self.finished.emit())
    monkeypatch.setattr(DownloadJob, "wait", lambda self, ms=0: True)
    win.jobs.set_limit(2)
    for i in range(4):
        win.url.setText(f"https://x.com/{i}"); win.add_job()
    assert len(started) == 2
    win.close()
    assert len(started) == 2


def test_icons_follow_saved_proxy(qapp, tmp_path):
    s = QSettings(str(tmp_path / "p.ini"), QSettings.Format.IniFormat)
    s.setValue("settings/proxy", "http://127.0.0.1:8080")
    w = mw.MainWindow(settings=s, data_dir=tmp_path)
    assert w.icons.proxy == "http://127.0.0.1:8080"


def test_default_window_is_landscape(win):
    # 처음 열 때 가로로 긴 창
    assert (win.width(), win.height()) == (920, 640)
