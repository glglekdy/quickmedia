"""임시 QuickMedia 아이콘을 그린다. 빨간 둥근 사각형 + 흰 재생 버튼 + 다운로드 화살표.

    python tools/make_icon.py   -> ytdl/assets/icon.png (256px), assets/icon.ico (16~256px)

정식 아이콘이 생기면 이 두 파일만 바꾸면 된다.
"""
from __future__ import annotations

import os
import struct
import sys
from pathlib import Path

SIZES = (16, 24, 32, 48, 64, 128, 256)
RED = "#dc2626"


def build_ico(pngs: dict[int, bytes]) -> bytes:
    """PNG 를 그대로 담은 ICO. 256px 은 너비·높이 바이트를 0 으로 적는다."""
    sizes = sorted(pngs)
    head = struct.pack("<HHH", 0, 1, len(sizes))
    offset = len(head) + 16 * len(sizes)
    entries, blobs = b"", b""
    for s in sizes:
        data = pngs[s]
        entries += struct.pack("<BBBBHHII", s % 256, s % 256, 0, 0, 1, 32, len(data), offset + len(blobs))
        blobs += data
    return head + entries + blobs


def render(size: int) -> bytes:
    from PySide6.QtCore import QBuffer, QByteArray, QIODevice, QPointF, QRectF, Qt
    from PySide6.QtGui import QColor, QImage, QPainter, QPolygonF

    s = float(size)
    img = QImage(size, size, QImage.Format.Format_ARGB32)
    img.fill(Qt.GlobalColor.transparent)
    p = QPainter(img)
    p.setRenderHint(QPainter.RenderHint.Antialiasing)
    p.setPen(Qt.PenStyle.NoPen)

    m = s * 0.06
    side = s - 2 * m
    p.setBrush(QColor(RED))
    p.drawRoundedRect(QRectF(m, m, side, side), side * 0.22, side * 0.22)

    p.setBrush(QColor("white"))
    # 재생 삼각형: 가운데에서 약간 위
    p.drawPolygon(QPolygonF([QPointF(s * 0.39, s * 0.22), QPointF(s * 0.39, s * 0.56),
                             QPointF(s * 0.67, s * 0.39)]))
    # 다운로드 화살표: 세로 막대 + 아래 화살촉
    p.drawRect(QRectF(s * 0.465, s * 0.60, s * 0.07, s * 0.12))
    p.drawPolygon(QPolygonF([QPointF(s * 0.36, s * 0.70), QPointF(s * 0.64, s * 0.70),
                             QPointF(s * 0.50, s * 0.84)]))
    p.end()

    buf = QBuffer(ba := QByteArray())
    buf.open(QIODevice.OpenModeFlag.WriteOnly)
    img.save(buf, "PNG")
    return bytes(ba)


def main() -> None:
    os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
    from PySide6.QtGui import QGuiApplication

    app = QGuiApplication(sys.argv)  # noqa: F841 - QPainter 에 필요
    root = Path(__file__).resolve().parent.parent
    pngs = {s: render(s) for s in SIZES}
    (root / "ytdl/assets/icon.png").write_bytes(pngs[256])
    (root / "assets/icon.ico").write_bytes(build_ico(pngs))
    print("ytdl/assets/icon.png, assets/icon.ico")


if __name__ == "__main__":
    main()
