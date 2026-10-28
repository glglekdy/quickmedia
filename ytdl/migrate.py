"""유튜브 다운로더(v1.0.1 이하)에서 넘어온 사용자 정리.

- 옛 설정(저장 위치, 형식)을 새 이름의 설정으로 한 번 옮긴다.
- 옛 앱이 자기 자리에 덮어쓴 설치 파일 사본(YoutubeDownloader.exe)을 지운다.
  설치 파일이 끝날 때 그 경로를 설정의 `legacy_exe` 에 적어 둔다.

여기서 나는 실패는 앱 시작을 막지 않는다. 다음 실행에 다시 시도한다.
"""

from __future__ import annotations

import threading
import time
from pathlib import Path

from PySide6.QtCore import QSettings

from .app_info import LEGACY_EXE, LEGACY_SETTINGS, SETTINGS


def plan_settings_migration(old: dict[str, object], new_keys: set[str]) -> dict[str, object]:
    """새 설정에 쓸 값. 이미 옮겼으면 빈 dict."""
    if "migrated" in new_keys:
        return {}
    return {**old, "migrated": True}


def is_legacy_exe(path: str) -> bool:
    return Path(path).name.lower() == LEGACY_EXE.lower()


def delete_legacy(path: str, attempts: int = 15, delay: float = 2.0) -> bool:
    """정리가 끝났으면 True. 설치 파일이 아직 잡고 있으면 잠깐씩 다시 시도한다."""
    if not is_legacy_exe(path):
        return True
    for i in range(attempts):
        try:
            Path(path).unlink(missing_ok=True)
            return True
        except OSError:
            if i < attempts - 1:
                time.sleep(delay)
    return False


def cleanup_step(settings: QSettings) -> threading.Thread | None:
    path = settings.value("legacy_exe")
    if not path:
        return None
    # QSettings 객체는 스레드끼리 나눠 쓰지 않는다. 같은 저장소를 가리키는 새 객체를 만든다.
    name, fmt = settings.fileName(), settings.format()

    def work() -> None:
        if delete_legacy(str(path)):
            s = QSettings(name, fmt)
            s.remove("legacy_exe")
            s.sync()

    t = threading.Thread(target=work, daemon=True)
    t.start()
    return t


def run() -> None:
    try:
        new, old = QSettings(*SETTINGS), QSettings(*LEGACY_SETTINGS)
        values = plan_settings_migration({k: old.value(k) for k in old.allKeys()}, set(new.allKeys()))
        for k, v in values.items():
            new.setValue(k, v)
        new.sync()
        cleanup_step(new)
    except Exception:  # noqa: BLE001 - 정리 실패로 앱이 안 켜지면 안 된다
        pass
