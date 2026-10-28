import os
import sys
from pathlib import Path

import pytest

# 화면 없이 Qt 위젯을 만든다
os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# 저장소 루트의 ytdl, tools 를 불러올 수 있게
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))


@pytest.fixture(scope="session")
def qapp():
    from PySide6.QtWidgets import QApplication
    return QApplication.instance() or QApplication([])
