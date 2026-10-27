"""유튜브 다운로더 진입점."""
import sys
from pathlib import Path

from PySide6.QtGui import QIcon
from PySide6.QtWidgets import QApplication

from ytdl import theme
from ytdl.main_window import MainWindow
from ytdl.update_dialog import check_on_startup

ICON = Path(getattr(sys, "_MEIPASS", Path(__file__).parent)) / "ytdl" / "assets" / "icon.png"


def main() -> None:
    app = QApplication(sys.argv)
    app.setApplicationName("유튜브 다운로더")
    app.setWindowIcon(QIcon(str(ICON)))
    theme.apply(app)
    win = MainWindow()
    win.show()
    check_on_startup(win)
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
