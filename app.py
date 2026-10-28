"""QuickMedia 진입점."""
import sys
from pathlib import Path

from PySide6.QtGui import QIcon
from PySide6.QtWidgets import QApplication

from ytdl import app_info, migrate, theme
from ytdl.main_window import MainWindow
from ytdl.update_dialog import check_on_startup

ICON = Path(getattr(sys, "_MEIPASS", Path(__file__).parent)) / "ytdl" / "assets" / "icon.png"


def main() -> None:
    app = QApplication(sys.argv)
    app.setApplicationName(app_info.NAME)
    app.setWindowIcon(QIcon(str(ICON)))
    theme.apply(app)
    migrate.run()
    win = MainWindow()
    win.show()
    check_on_startup(win)
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
