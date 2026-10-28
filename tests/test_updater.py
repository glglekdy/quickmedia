from pathlib import Path

from ytdl.updater import apply_command, parse_version, pick_setup_asset


def test_pick_setup_asset_matches_setup_name():
    assets = [{"name": "notes.txt"}, {"name": "QuickMedia-Setup-1.2.0.exe", "browser_download_url": "u"}]
    assert pick_setup_asset(assets)["browser_download_url"] == "u"


def test_pick_setup_asset_case_insensitive():
    assert pick_setup_asset([{"name": "quickmedia-setup-1.2.0.EXE"}]) is not None


def test_pick_setup_asset_ignores_other_exe():
    assert pick_setup_asset([{"name": "YoutubeDownloader.exe"}, {"name": "QuickMedia.exe"}]) is None
    assert pick_setup_asset([]) is None


def test_parse_version_unchanged():
    assert parse_version("v1.2.0") == (1, 2)
    assert parse_version("1.2.3-beta") == (1, 2, 3)
    assert parse_version("") == ()


def test_apply_command_passes_paths_verbatim():
    setup = Path("C:/임시 폴더/QuickMedia-Setup-2.0.0.exe")
    exe = Path("D:/내 프로그램/QuickMedia/QuickMedia.exe")
    cmd = apply_command(Path("C:/t/a.ps1"), 42, setup, exe)
    assert cmd[:2] == ["powershell.exe", "-NoProfile"]
    assert cmd[cmd.index("-WaitPid") + 1] == "42"
    assert cmd[cmd.index("-Setup") + 1] == str(setup)
    assert cmd[cmd.index("-Exe") + 1] == str(exe)
