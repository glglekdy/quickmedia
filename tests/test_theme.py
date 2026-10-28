import re
from pathlib import Path

from ytdl import theme


def test_stylesheet_images_exist():
    # 체크 표시·화살표 그림이 빠지면 체크박스와 콤보박스가 텅 비어 보인다
    css = theme.stylesheet(theme.LIGHT)
    urls = re.findall(r"url\(([^)]+)\)", css)
    assert len(urls) >= 3
    for u in urls:
        assert Path(u).is_file(), u
    assert "$" not in css
