import re
from pathlib import Path

ISS = (Path(__file__).resolve().parent.parent / "installer" / "QuickMedia.iss").read_text(encoding="utf-8-sig")


def test_update_clears_old_internal_folder():
    # 업데이트가 옛 _internal 위에 덮어쓰면 의존성을 올린 뒤 옛 DLL·Qt 플러그인이 남아 앱이 깨질 수 있다.
    section = re.search(r"^\[InstallDelete\]\s*$(.*?)(?=^\[|\Z)", ISS, re.M | re.S)
    assert section, "[InstallDelete] 섹션이 없다"
    assert re.search(r'Type:\s*filesandordirs;\s*Name:\s*"\{app\}\\_internal"', section.group(1))
