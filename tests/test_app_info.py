from ytdl import app_info


def test_names():
    assert app_info.NAME == "QuickMedia"
    assert app_info.EXE == "QuickMedia.exe"
    assert app_info.REPO == "glglekdy/quickmedia"
    assert app_info.SETTINGS == ("QuickMedia", "QuickMedia")
    assert app_info.LEGACY_SETTINGS == ("YoutubeDownloader", "YoutubeDownloader")
    assert app_info.LEGACY_EXE == "YoutubeDownloader.exe"
