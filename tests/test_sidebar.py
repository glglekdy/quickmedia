import pytest


@pytest.fixture
def bar(qapp):
    from ytdl.sidebar import Sidebar
    b = Sidebar()
    b.emitted = []
    b.page_changed.connect(b.emitted.append)
    return b


def test_starts_on_download(bar):
    assert bar.current == "download" and not bar.expanded
    assert bar.nav["download"].isChecked() and not bar.nav["settings"].isChecked()


def test_settings_expands_to_login(bar):
    bar.nav["settings"].click()
    assert bar.emitted == ["login"] and bar.expanded and bar.current == "login"
    assert bar.nav["login"].isChecked() and not bar.nav["download"].isChecked()


def test_settings_again_collapses(bar):
    bar.nav["settings"].click()
    bar.nav["settings"].click()
    # 접기만 하고 화면은 그대로. 보던 설정 화면이 숨었으니 '설정'에 선택 표시
    assert bar.emitted == ["login"] and not bar.expanded and bar.current == "login"
    assert bar.nav["settings"].isChecked()


def test_category_and_download(bar):
    bar.nav["settings"].click()
    bar.nav["network"].click()
    assert bar.emitted == ["login", "network"] and bar.nav["network"].isChecked()
    bar.nav["download"].click()
    assert bar.emitted[-1] == "download" and not bar.expanded
    assert not bar.nav["settings"].isChecked()


def test_reexpand_keeps_settings_page(bar):
    bar.nav["settings"].click()
    bar.nav["file"].click()
    bar.nav["settings"].click()  # 접기
    bar.nav["settings"].click()  # 다시 펼치기
    assert bar.emitted == ["login", "file"] and bar.current == "file" and bar.expanded
    assert bar.nav["file"].isChecked() and not bar.nav["settings"].isChecked()


def test_same_item_no_signal(bar):
    bar.nav["download"].click()
    bar.nav["settings"].click()
    bar.nav["login"].click()
    assert bar.emitted == ["login"]


def test_select_from_code(bar):
    bar.select("advanced")
    assert bar.emitted == ["advanced"] and bar.expanded and bar.nav["advanced"].isChecked()
    bar.select("download")
    assert bar.emitted[-1] == "download" and not bar.expanded


def test_nav_hover_token():
    from ytdl import theme
    assert theme.LIGHT["nav_hover"] == "#f0f0f0" and theme.DARK["nav_hover"] == "#232323"


def test_sidebar_is_garbage_collected(qapp):
    import gc
    import weakref
    from ytdl.sidebar import Sidebar
    ref = weakref.ref(Sidebar())
    gc.collect()
    assert ref() is None
