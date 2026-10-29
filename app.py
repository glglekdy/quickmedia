"""QuickMedia 진입점."""
import sys
from pathlib import Path

from PySide6.QtGui import QIcon
from PySide6.QtWidgets import QApplication

from ytdl import app_info, migrate, theme
from ytdl.main_window import MainWindow

ICON = Path(getattr(sys, "_MEIPASS", Path(__file__).parent)) / "ytdl" / "assets" / "icon.png"


def main() -> None:
    app = QApplication(sys.argv)
    app.setApplicationName(app_info.NAME)
    app.setWindowIcon(QIcon(str(ICON)))
    theme.apply(app)
    migrate.run()
    win = MainWindow()
    win.show()
    win.update_page.check()  # 뒤에서 확인. 새 버전이면 메뉴에 빨간 점만
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
