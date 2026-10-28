import os
import time

from PySide6.QtCore import QBuffer, QByteArray, QIODevice, QThreadPool
from PySide6.QtGui import QColor, QImage
from PySide6.QtWidgets import QApplication

from ytdl.site import SiteIcons, display_name, domain_of, icon_url, letter_of


def test_domain_of():
    assert domain_of("https://www.youtube.com/watch?v=x") == "youtube.com"
    assert domain_of("https://m.youtube.com/watch?v=x") == "youtube.com"
    assert domain_of("https://tv.naver.com/v/1") == "tv.naver.com"
    assert domain_of("not a url") == ""


def test_display_name():
    assert display_name("Youtube", "youtube.com") == "YouTube"
    assert display_name("YoutubeTab", "youtube.com") == "YouTube"
    assert display_name("Twitch:vod", "twitch.tv") == "Twitch"
    assert display_name("Twitter", "x.com") == "X"
    assert display_name("SoopLive", "sooplive.co.kr") == "SOOP"
    assert display_name("Naver", "tv.naver.com") == "네이버 TV"
    assert display_name("Generic", "example.com") == "example.com"
    assert display_name("", "example.com") == "example.com"
    assert display_name("Rumble", "rumble.com") == "Rumble"


def test_icon_url_and_letter():
    assert icon_url("youtube.com") == "https://youtube.com/favicon.ico"
    assert letter_of("youtube.com") == "Y" and letter_of("네이버 TV") == "네" and letter_of("") == "?"


def _png():
    img = QImage(48, 48, QImage.Format.Format_ARGB32); img.fill(QColor("#00ff00"))
    ba = QByteArray(); buf = QBuffer(ba); buf.open(QIODevice.OpenModeFlag.WriteOnly); img.save(buf, "PNG")
    return bytes(ba)


def _settle():
    QThreadPool.globalInstance().waitForDone(3000)
    QApplication.processEvents()


def test_fetches_caches_and_signals(qapp, tmp_path):
    calls, got = [], []
    icons = SiteIcons(tmp_path, fetch=lambda u: calls.append(u) or _png())
    icons.ready.connect(lambda h, p: got.append(h))
    first = icons.icon("youtube.com")              # 처음엔 대체 아이콘
    assert first.toImage().pixelColor(16, 16) != QColor("#00ff00")
    _settle()
    assert got == ["youtube.com"] and calls == ["https://youtube.com/favicon.ico"]
    assert (tmp_path / "youtube.com.png").exists()
    assert icons.icon("youtube.com").toImage().pixelColor(16, 16) == QColor("#00ff00")
    again = SiteIcons(tmp_path, fetch=lambda u: calls.append(u))  # 디스크 캐시
    again.icon("youtube.com"); _settle()
    assert len(calls) == 1


def test_stale_cache_refetches(qapp, tmp_path):
    (tmp_path / "a.com.png").write_bytes(_png())
    old = time.time() - 31 * 86400
    os.utime(tmp_path / "a.com.png", (old, old))
    calls = []
    SiteIcons(tmp_path, fetch=lambda u: calls.append(u) or _png()).icon("a.com"); _settle()
    assert calls == ["https://a.com/favicon.ico"]


def test_html_response_keeps_fallback_and_is_not_retried(qapp, tmp_path):
    calls, got = [], []
    icons = SiteIcons(tmp_path, fetch=lambda u: calls.append(u) or b"<html>not found</html>")
    icons.ready.connect(lambda h, p: got.append(h))
    icons.icon("b.com"); _settle()
    icons.icon("b.com"); _settle()
    assert got == [] and len(calls) == 1 and not (tmp_path / "b.com.png").exists()


def test_icons_deleted_while_fetching_is_quiet(qapp, tmp_path, capfd):
    # 앱이 끝나며 Qt 객체가 먼저 지워져도 받던 작업이 오류를 내면 안 된다
    import threading

    import shiboken6
    gate = threading.Event()
    icons = SiteIcons(tmp_path, fetch=lambda u: gate.wait(2) and _png())
    icons.icon("slow.com")
    shiboken6.delete(icons._relay)
    gate.set()
    _settle()
    assert "Signal source has been deleted" not in capfd.readouterr().err


def test_default_fetch_uses_proxy(qapp, tmp_path, monkeypatch):
    # 프록시를 켠 사람의 실제 IP 로 사이트에 직접 붙으면 안 된다
    from ytdl import site
    calls = []
    monkeypatch.setattr(site, "fetch_bytes", lambda url, proxy="": calls.append((url, proxy)))
    icons = SiteIcons(tmp_path)
    icons.proxy = "http://127.0.0.1:8080"
    icons.icon("c.com"); _settle()
    assert calls == [("https://c.com/favicon.ico", "http://127.0.0.1:8080")]


def test_fetch_bytes_builds_proxy_opener(monkeypatch):
    from ytdl import site
    seen = {}

    class Opener:
        def open(self, req, timeout):
            seen["timeout"] = timeout
            raise OSError("no network in tests")

    def build_opener(*handlers):
        seen["proxies"] = [getattr(h, "proxies", None) for h in handlers]
        return Opener()

    monkeypatch.setattr(site.urllib.request, "build_opener", build_opener)
    assert site.fetch_bytes("https://c.com/favicon.ico", proxy="http://p:1") is None
    assert {"http": "http://p:1", "https": "http://p:1"} in seen["proxies"] and seen["timeout"] == 5
