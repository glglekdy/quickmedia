from ytdl.options import AppSettings, DownloadOptions, validate_options, validate_settings


def test_valid_defaults():
    assert validate_options(DownloadOptions()) == {} and validate_settings(AppSettings()) == {}


def test_sections():
    assert validate_options(DownloadOptions(section_start="1:30", section_end="3:00")) == {}
    assert validate_options(DownloadOptions(section_start="abc"))["section"] == "시간은 1:30 처럼 써 주세요"
    assert validate_options(DownloadOptions(section_start="3:00", section_end="1:00"))["section"] == "끝이 시작보다 뒤여야 해요"


def test_playlist_items_only_when_all():
    assert validate_options(DownloadOptions(playlist="single", playlist_items="x")) == {}
    assert validate_options(DownloadOptions(playlist="all", playlist_items="1-10, 15")) == {}
    assert validate_options(DownloadOptions(playlist="all", playlist_items="x"))["playlist_items"] == "범위는 1-10, 15 처럼 써 주세요"


def test_rate_limit():
    assert validate_settings(AppSettings(rate_limit="500K")) == {}
    assert validate_settings(AppSettings(rate_limit="fast"))["rate_limit"] == "속도 제한은 5M, 500K 처럼 써 주세요"


def test_cookie_file(tmp_path):
    assert validate_settings(AppSettings(cookie_file="Z:/none.txt")) == {}  # 쿠키 모드가 file 이 아닐 때는 보지 않음
    assert validate_settings(AppSettings(cookies="file", cookie_file=str(tmp_path / "no.txt")))["cookie_file"] == "쿠키 파일을 찾을 수 없어요"
    f = tmp_path / "c.txt"; f.write_text("x")
    assert validate_settings(AppSettings(cookies="file", cookie_file=str(f))) == {}


def test_custom_template():
    assert validate_settings(AppSettings(name_template="custom", custom_template=" "))["custom_template"] == "파일 이름 규칙을 입력해 주세요"


def test_extra_args():
    assert validate_settings(AppSettings(extra_args="--limit-rate 1M --no-mtime")) == {}
    msg = validate_settings(AppSettings(extra_args="--nope"))["extra_args"]
    assert msg.startswith("고급 옵션을 읽지 못했어요: ") and "--nope" in msg and "Usage" not in msg
    assert validate_settings(AppSettings(extra_args="https://x.com/v"))["extra_args"] == "고급 옵션에는 링크를 넣을 수 없어요"


def test_webm_cannot_hold_h264():
    # yt-dlp 가 H.264 를 골라 WEBM 으로 합치려다 알 수 없는 오류로 실패한다
    assert validate_options(DownloadOptions(container="webm", codec="h264"))["codec"] == "WEBM 에는 H.264 를 넣을 수 없어요. 코덱이나 파일 형식을 바꿔 주세요"
    assert validate_options(DownloadOptions(container="webm", codec="h264", kind="audio")) == {}
    assert validate_options(DownloadOptions(container="mkv", codec="h264")) == {}
