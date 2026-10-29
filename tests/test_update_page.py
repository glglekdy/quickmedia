import threading
from pathlib import Path

import pytest

from ytdl import update_page as up
from ytdl import updater

REL = updater.Release(version="1.2.0", notes="## 새 기능\n- 하나", asset_url="u", asset_size=0, digest="")


@pytest.fixture
def env(qapp, monkeypatch):
    """인터넷 없이. fetch/download/apply 가 무엇을 할지 바꿔 끼운다."""
    e = {"fetch": lambda: REL, "download": lambda r, p: Path("setup.exe"), "apply": lambda path: None,
         "downloads": 0, "applied": [], "quit": 0}

    def download(release, progress=None):
        e["downloads"] += 1
        return e["download"](release, progress)

    def apply(path):
        e["applied"].append(path)
        e["apply"](path)

    monkeypatch.setattr(updater, "fetch_latest", lambda: e["fetch"]())
    monkeypatch.setattr(updater, "download", download)
    monkeypatch.setattr(updater, "launch_apply", apply)
    monkeypatch.setattr(updater, "can_self_update", lambda: True)
    monkeypatch.setattr(up, "_quit", lambda: e.__setitem__("quit", e["quit"] + 1))
    return e


def make(confirm=None):
    page = up.UpdatePage(confirm=confirm)
    page.dots = []
    page.available_changed.connect(page.dots.append)
    return page


def run(qapp, page):
    """돌고 있는 스레드가 끝나길 기다렸다가 신호를 받는다."""
    for job in (page.check_job, page.download_job):
        if job is not None:
            job.wait(5000)
    qapp.processEvents()


def test_starts_idle(env):
    page = make()
    assert page.state == "idle" and page.title.text() == "아직 확인하지 않았어요"
    assert page.action.text() == "업데이트 확인" and page.dots == []


def test_found_new_version(qapp, env):
    page = make()
    page.check()
    run(qapp, page)
    assert page.state == "available" and page.title.text() == "새 버전 1.2.0 이 나왔어요"
    assert not page.notes.isHidden() and "새 기능" in page.notes.toPlainText()
    assert page.dots == [True] and page.action.text() == "지금 업데이트" and page.action.isEnabled()
    assert page.status.text() == "받고 나면 QuickMedia 가 잠깐 꺼졌다 다시 켜져요."


def test_latest(qapp, env):
    env["fetch"] = lambda: None
    page = make()
    page.check()
    run(qapp, page)
    assert page.state == "latest" and page.title.text() == "최신 버전이에요"
    assert page.sub.text().endswith(" 에 확인했어요") and page.dots == []


def test_check_failed(qapp, env):
    def fail():
        raise updater.UpdateError("GitHub 에 연결하지 못했어요.")
    env["fetch"] = fail
    page = make()
    page.check()
    run(qapp, page)
    assert page.state == "check_failed" and page.title.text() == "확인하지 못했어요"
    assert page.sub.text() == "GitHub 에 연결하지 못했어요." and page.action.text() == "다시 확인"
    assert page.dots == []


def test_check_button_disabled_while_checking(qapp, env):
    gate = threading.Event()
    env["fetch"] = lambda: gate.wait(5) and None
    page = make()
    page.check()
    job = page.check_job
    assert page.state == "checking" and not page.action.isEnabled()
    page.check()
    assert page.check_job is job
    gate.set()
    run(qapp, page)
    assert page.state == "latest"


def test_source_run_cannot_update(qapp, env, monkeypatch):
    monkeypatch.setattr(updater, "can_self_update", lambda: False)
    page = make()
    page.check()
    run(qapp, page)
    assert not page.action.isEnabled()
    assert page.status.text() == "소스로 실행 중이라 자동 업데이트는 exe 에서만 돼요."


def test_confirm_false_does_not_download(qapp, env):
    page = make(confirm=lambda: False)
    page.check()
    run(qapp, page)
    page.action.click()
    run(qapp, page)
    assert env["downloads"] == 0 and page.state == "available"


def test_download_failure_keeps_dot_and_retries(qapp, env):
    def fail(release, progress):
        raise updater.UpdateError("업데이트 파일을 받다가 끊겼어요. 다시 시도해 주세요.")
    env["download"] = fail
    page = make()
    page.check()
    run(qapp, page)
    page.action.click()
    run(qapp, page)
    assert page.state == "download_failed" and page.bar.isHidden()
    assert page.status.text() == "업데이트 파일을 받다가 끊겼어요. 다시 시도해 주세요."
    assert page.action.text() == "다시 시도" and page.action.isEnabled() and page.dots == [True]
    page.action.click()
    run(qapp, page)
    assert env["downloads"] == 2


def test_download_success_applies(qapp, env):
    page = make()
    page.check()
    run(qapp, page)
    page.action.click()
    run(qapp, page)
    assert env["applied"] == [Path("setup.exe")] and env["quit"] == 1
    assert page.state == "applying" and page.status.text() == "곧 다시 시작해요…"


def test_apply_failure_shows_error(qapp, env):
    def fail(path):
        raise updater.UpdateError("exe 로 실행할 때만 자동 업데이트할 수 있어요.")
    env["apply"] = fail
    page = make()
    page.check()
    run(qapp, page)
    page.action.click()
    run(qapp, page)
    assert page.state == "download_failed" and env["quit"] == 0
    assert page.status.text() == "exe 로 실행할 때만 자동 업데이트할 수 있어요."


def test_page_is_garbage_collected(qapp, env):
    import gc
    import weakref
    page = make()
    page.check()
    run(qapp, page)
    ref = weakref.ref(page)
    del page
    gc.collect()
    assert ref() is None
