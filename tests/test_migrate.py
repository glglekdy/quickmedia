import threading

from PySide6.QtCore import QSettings

from ytdl.migrate import cleanup_step, delete_legacy, is_legacy_exe, plan_settings_migration


def test_plan_copies_old_keys_once():
    assert plan_settings_migration({"folder": "D:/v", "format": "mp3"}, set()) == \
        {"folder": "D:/v", "format": "mp3", "migrated": True}
    assert plan_settings_migration({"folder": "D:/v"}, {"migrated", "folder"}) == {}


def test_plan_with_empty_old_marks_migrated():
    assert plan_settings_migration({}, set()) == {"migrated": True}


def test_is_legacy_exe():
    assert is_legacy_exe(r"C:\Users\a\Downloads\YoutubeDownloader.exe")
    assert is_legacy_exe("c:/x/youtubedownloader.EXE")
    assert not is_legacy_exe(r"C:\x\QuickMedia-Setup-2.0.0.exe")


def test_delete_legacy_removes_file(tmp_path):
    f = tmp_path / "YoutubeDownloader.exe"
    f.write_bytes(b"x")
    assert delete_legacy(str(f), attempts=1, delay=0)
    assert not f.exists()


def test_delete_legacy_skips_other_names(tmp_path):
    f = tmp_path / "Other.exe"
    f.write_bytes(b"x")
    assert delete_legacy(str(f), attempts=1, delay=0)
    assert f.exists()


def test_delete_legacy_missing_file_is_done(tmp_path):
    assert delete_legacy(str(tmp_path / "YoutubeDownloader.exe"), attempts=1, delay=0)


def test_delete_legacy_retries_while_locked(tmp_path):
    f = tmp_path / "YoutubeDownloader.exe"
    f.write_bytes(b"x")
    fh = f.open("rb")  # Windows 에서는 열린 파일을 지울 수 없다
    threading.Timer(0.3, fh.close).start()
    assert delete_legacy(str(f), attempts=10, delay=0.1)
    assert not f.exists()


def test_cleanup_step_handles_pending_value(tmp_path):
    f = tmp_path / "YoutubeDownloader.exe"
    f.write_bytes(b"x")
    s = QSettings(str(tmp_path / "s.ini"), QSettings.Format.IniFormat)
    s.setValue("legacy_exe", str(f))
    s.sync()
    cleanup_step(s).join(5)
    assert not f.exists()
    s.sync()
    assert not s.contains("legacy_exe")
