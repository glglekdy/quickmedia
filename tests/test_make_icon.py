import struct

from tools.make_icon import build_ico


def test_build_ico_layout():
    pngs = {16: b"\x89PNG-a", 256: b"\x89PNG-bb"}
    data = build_ico(pngs)
    assert data[:6] == struct.pack("<HHH", 0, 1, 2)
    w, h, _, _, planes, bpp, size, offset = struct.unpack_from("<BBBBHHII", data, 6)
    assert (w, h, planes, bpp, size, offset) == (16, 16, 1, 32, 6, 6 + 16 * 2)
    assert data[offset:offset + size] == b"\x89PNG-a"
    w2, h2, _, _, _, _, size2, offset2 = struct.unpack_from("<BBBBHHII", data, 22)
    assert (w2, h2, size2) == (0, 0, 7)
    assert data[offset2:offset2 + 7] == b"\x89PNG-bb"
