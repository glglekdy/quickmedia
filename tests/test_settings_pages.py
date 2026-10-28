import pytest

from ytdl.options import AppSettings


def make(s=None):
    from ytdl.settings_pages import PAGES, SettingsModel
    model = SettingsModel(s or AppSettings())
    model.hits = []
    model.changed.connect(model.hits.append)
    return model, {key: cls(model) for key, cls in PAGES.items()}


def test_pages(qapp):
    _, pages = make()
    assert list(pages) == ["login", "network", "file", "advanced"]
    assert [p.title for p in pages.values()] == ["로그인", "네트워크", "파일", "고급"]


def test_round_trip(qapp):
    s = AppSettings(cookies="browser", cookie_browser="edge", cookie_file="C:/c.txt", proxy="http://p:1",
                    rate_limit="5M", retries=3, name_template="custom", custom_template="%(id)s.%(ext)s",
                    overwrite=True, concurrency=4, extra_args="--no-mtime")
    _, p = make(s)
    login, net, file, adv = p["login"], p["network"], p["file"], p["advanced"]
    assert login.cookie_browser.isChecked() and login.browser.currentData() == "edge"
    assert login.cookie_path.text() == "C:/c.txt" and not login.cookie_path.isEnabled()
    assert (net.proxy.text(), net.rate_limit.text(), net.retries.value(), net.concurrency.value()) == ("http://p:1", "5M", 3, 4)
    assert file.template.currentData() == "custom" and file.custom_template.text() == "%(id)s.%(ext)s"
    assert file.custom_template.isEnabled() and file.overwrite.isChecked()
    assert adv.extra_args.text() == "--no-mtime"


def test_checkbox_saves_immediately(qapp):
    model, p = make()
    p["file"].overwrite.click()
    assert model.value.overwrite is True and len(model.hits) == 1
    assert not p["file"].saved_badge.isHidden()


def test_spin_saves_immediately(qapp):
    model, p = make()
    p["network"].concurrency.setValue(4)
    assert model.value.concurrency == 4 and not p["network"].saved_badge.isHidden()


def test_bad_rate_shows_error(qapp):
    model, p = make()
    net = p["network"]
    net.rate_limit.setText("fast")
    net.rate_limit.editingFinished.emit()
    assert net.errors["rate_limit"].text() == "속도 제한은 5M, 500K 처럼 써 주세요"
    assert not net.errors["rate_limit"].isHidden() and net.rate_limit.property("invalid") is True
    assert model.value.rate_limit == "" and model.hits == [] and net.saved_badge.isHidden()
    net.rate_limit.setText("5M")
    net.rate_limit.editingFinished.emit()
    assert model.value.rate_limit == "5M" and net.errors["rate_limit"].isHidden()
    assert net.rate_limit.property("invalid") is False


def test_error_clears_when_reverted_to_saved(qapp):
    model, p = make()
    net = p["network"]
    net.rate_limit.setText("fast")
    net.rate_limit.editingFinished.emit()
    net.rate_limit.setText("")
    net.rate_limit.editingFinished.emit()
    assert net.errors["rate_limit"].isHidden() and model.hits == [] and net.saved_badge.isHidden()


def test_cookie_file_without_file(qapp):
    model, p = make()
    login = p["login"]
    login.cookie_file.click()
    assert login.errors["cookie_file"].text() == "쿠키 파일을 찾을 수 없어요"
    assert not login.errors["cookie_file"].isHidden() and login.cookie_path.property("invalid") is True
    assert model.value.cookies == "none"


def test_cookie_file_with_file(qapp, tmp_path):
    f = tmp_path / "c.txt"
    f.write_text("x")
    model, p = make()
    login = p["login"]
    login.cookie_path.setText(str(f))
    login.cookie_file.click()
    assert model.value.cookies == "file" and model.value.cookie_file == str(f)
    assert login.errors["cookie_file"].isHidden()


def test_custom_template_saved_with_name(qapp):
    model, p = make()
    file = p["file"]
    file.custom_template.setText("%(id)s.%(ext)s")
    file.template.setCurrentIndex(file.template.findData("custom"))
    assert model.value.name_template == "custom" and model.value.custom_template == "%(id)s.%(ext)s"


def test_leave_saves_valid_pending_text(qapp):
    model, p = make()
    p["network"].proxy.setText("http://x:1")  # Enter 도, 다른 곳 클릭도 없이
    p["network"].leave()
    assert model.value.proxy == "http://x:1"


def test_leave_reverts_invalid(qapp):
    model, p = make(AppSettings(rate_limit="1M"))
    net = p["network"]
    net.rate_limit.setText("fast")
    net.rate_limit.editingFinished.emit()
    net.leave()
    assert net.rate_limit.text() == "1M" and net.errors["rate_limit"].isHidden()
    assert net.rate_limit.property("invalid") is False and model.value.rate_limit == "1M"


def test_leave_restores_unsaved_cookie_mode(qapp):
    model, p = make()
    login = p["login"]
    login.cookie_file.click()
    login.leave()
    assert login.cookie_none.isChecked() and login.errors["cookie_file"].isHidden()
    assert not login.cookie_path.isEnabled()


def test_stale_saved_error_hidden_until_edited(qapp):
    # 이미 틀린 저장값은 건드리기 전에는 보여 주지 않는다
    _, p = make(AppSettings(rate_limit="fast"))
    assert p["network"].errors["rate_limit"].isHidden()
