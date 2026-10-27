"""exe 버전 리소스를 만든다. 이게 없으면 작업 관리자·속성 창이 파일명을 앱 이름으로 쓴다.

    python tools/version_info.py build/version_info.txt
"""
import re
import sys
from pathlib import Path

NAME = "유튜브 다운로더"

version = re.search(r'__version__ = "([^"]+)"',
                    Path("ytdl/__init__.py").read_text(encoding="utf-8")).group(1)
nums = (tuple(int(p) for p in version.split(".")) + (0, 0, 0, 0))[:4]

text = f"""VSVersionInfo(
  ffi=FixedFileInfo(filevers={nums}, prodvers={nums}, mask=0x3F, flags=0x0,
                    OS=0x40004, fileType=0x1, subtype=0x0, date=(0, 0)),
  kids=[
    StringFileInfo([StringTable('041204B0', [
      StringStruct('CompanyName', '{NAME}'),
      StringStruct('FileDescription', '{NAME}'),
      StringStruct('FileVersion', '{version}'),
      StringStruct('InternalName', 'YoutubeDownloader'),
      StringStruct('OriginalFilename', 'YoutubeDownloader.exe'),
      StringStruct('ProductName', '{NAME}'),
      StringStruct('ProductVersion', '{version}')])]),
    VarFileInfo([VarStruct('Translation', [0x0412, 1200])])
  ]
)
"""
out = Path(sys.argv[1])
out.parent.mkdir(parents=True, exist_ok=True)
out.write_text(text, encoding="utf-8")
