"""GitHub 릴리즈의 설치 파일로 자기 자신을 업데이트한다.

1. `fetch_latest()`  releases/latest 를 읽어 지금보다 새 버전인지 본다.
2. `download()`      QuickMedia-Setup-*.exe 자산을 받아 임시 폴더에 둔다.
3. `launch_apply()`  도우미 스크립트를 띄우고 앱은 종료한다. 스크립트가 앱이
                     완전히 끝나기를 기다렸다가 설치 파일을 조용히 실행한다.
                     설치가 끝나면 설치 파일이 앱을 다시 띄운다.

실행 중인 앱 파일은 윈도우가 잠그고 있어 덮어쓸 수 없어서 마지막 단계는
바깥 프로세스가 맡는다.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

from . import __version__
from .app_info import NAME, REPO

LATEST_URL = f"https://api.github.com/repos/{REPO}/releases/latest"

UPDATE_DIR = Path(tempfile.gettempdir()) / f"{NAME}-update"
_SETUP_NAME = re.compile(rf"^{NAME}-Setup-.+\.exe$", re.IGNORECASE)


class UpdateError(RuntimeError):
    """사용자에게 그대로 보여줄 수 있는 업데이트 실패."""


def parse_version(text: str) -> tuple[int, ...]:
    """`v1.2.3`, `1.2`, `1.2.3-beta` 모두 숫자 부분만 비교한다."""
    match = re.match(r"\s*v?(\d+(?:\.\d+)*)", text or "")
    if not match:
        return ()
    parts = [int(p) for p in match.group(1).split(".")]
    while len(parts) > 1 and parts[-1] == 0:
        parts.pop()
    return tuple(parts)


def can_self_update() -> bool:
    """소스로 실행 중이면 덮어쓸 exe 가 없다."""
    return sys.platform == "win32" and getattr(sys, "frozen", False)


@dataclass
class Release:
    version: str
    notes: str
    asset_url: str
    asset_size: int
    digest: str  # "sha256:..." (GitHub 가 채워준다)


def _request(url: str, accept: str = "application/vnd.github+json"):
    req = urllib.request.Request(url, headers={
        "User-Agent": f"{NAME}/{__version__}", "Accept": accept})
    return urllib.request.urlopen(req, timeout=20)


def pick_setup_asset(assets: list[dict]) -> dict | None:
    """릴리즈 자산 중 설치 파일. 다른 exe 는 보지 않는다."""
    return next((a for a in assets if _SETUP_NAME.match(a.get("name") or "")), None)


def fetch_latest() -> Release | None:
    """지금보다 새 정식 릴리즈. 없으면 None."""
    try:
        with _request(LATEST_URL) as resp:
            data = json.load(resp)
    except urllib.error.HTTPError as exc:
        if exc.code == 404:  # 릴리즈가 하나도 없음
            return None
        raise UpdateError(f"릴리즈 정보를 가져오지 못했어요. (HTTP {exc.code})") from exc
    except (urllib.error.URLError, TimeoutError) as exc:
        raise UpdateError("GitHub 에 연결하지 못했어요.") from exc

    version = (data.get("tag_name") or "").lstrip("vV")
    if parse_version(version) <= parse_version(__version__):
        return None
    setup = pick_setup_asset(data.get("assets") or [])
    if setup is None:
        return None
    return Release(version=version, notes=data.get("body") or "",
                   asset_url=setup["browser_download_url"], asset_size=int(setup.get("size") or 0),
                   digest=setup.get("digest") or "")


def download(release: Release, progress: Callable[[int, int], None] | None = None) -> Path:
    UPDATE_DIR.mkdir(parents=True, exist_ok=True)
    target = UPDATE_DIR / f"{NAME}-Setup-{release.version}.exe"
    part = target.with_suffix(".part")
    hasher = hashlib.sha256()
    try:
        with _request(release.asset_url, accept="application/octet-stream") as resp, part.open("wb") as fh:
            total = int(resp.headers.get("Content-Length") or release.asset_size or 0)
            done = 0
            while chunk := resp.read(256 * 1024):
                fh.write(chunk)
                hasher.update(chunk)
                done += len(chunk)
                if progress:
                    progress(done, total)
    except (urllib.error.URLError, TimeoutError, OSError) as exc:
        part.unlink(missing_ok=True)
        raise UpdateError("업데이트 파일을 받다가 끊겼어요. 다시 시도해 주세요.") from exc

    if release.asset_size and part.stat().st_size != release.asset_size:
        part.unlink(missing_ok=True)
        raise UpdateError("받은 파일 크기가 맞지 않아요. 다시 시도해 주세요.")
    algo, _, expected = release.digest.partition(":")
    if algo == "sha256" and expected and hasher.hexdigest() != expected.lower():
        part.unlink(missing_ok=True)
        raise UpdateError("받은 파일이 손상되었어요. (SHA-256 불일치)")
    part.replace(target)
    return target


# 앱이 끝나기를 기다렸다가 설치 파일을 조용히 실행한다.
_APPLY_SCRIPT = r"""
param([int]$WaitPid, [string]$Setup, [string]$Exe)
Wait-Process -Id $WaitPid -Timeout 120 -ErrorAction SilentlyContinue
$p = Start-Process -FilePath $Setup -ArgumentList '/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART' -PassThru
# -Wait 는 설치 파일이 다시 띄운 앱까지 기다린다. 설치 파일 프로세스만 기다린다.
$p.WaitForExit()
# 성공하면 설치 파일의 [Run] 이 앱을 다시 띄운다. 실패하면 Inno 가 되돌려 둔 기존 앱을 띄운다.
if ($p.ExitCode -ne 0 -and (Test-Path $Exe)) { Start-Process -FilePath $Exe -WorkingDirectory (Split-Path $Exe) }
Remove-Item $Setup -Force -ErrorAction SilentlyContinue
"""


def _relaunch_env() -> dict[str, str]:
    """onefile 부트로더는 `_PYI_*` 변수로 자기 자식을 알아본다. 그대로 물려주면 새 exe 가
    옛 프로세스의 `_MEI...` 폴더를 쓰려다 "Failed to load Python DLL" 이 뜬다."""
    env = {k: v for k, v in os.environ.items()
           if not k.upper().startswith(("_PYI_", "_MEIPASS"))}
    env["PYINSTALLER_RESET_ENVIRONMENT"] = "1"
    return env


def apply_command(script: Path, pid: int, setup: Path, exe: Path) -> list[str]:
    """도우미 PowerShell 을 띄울 인자. 리스트로 넘기므로 공백·한글 경로도 그대로 간다."""
    return ["powershell.exe", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
            "-WindowStyle", "Hidden", "-File", str(script),
            "-WaitPid", str(pid), "-Setup", str(setup), "-Exe", str(exe)]


def launch_apply(setup: Path) -> None:
    """도우미를 띄운다. 호출한 쪽은 곧바로 앱을 종료해야 한다."""
    if not can_self_update():
        raise UpdateError("exe 로 실행할 때만 자동 업데이트할 수 있어요.")
    script = UPDATE_DIR / "apply_update.ps1"
    # PowerShell 5.1 은 BOM 이 없으면 ANSI 로 읽어 한글 경로가 깨진다.
    script.write_text(_APPLY_SCRIPT, encoding="utf-8-sig")
    # onedir 은 부모 부트로더가 없다. 부모(탐색기)를 기다리면 타임아웃까지 멈춘다.
    subprocess.Popen(
        apply_command(script, os.getpid(), setup, Path(sys.executable).resolve()),
        env=_relaunch_env(),
        creationflags=subprocess.CREATE_NO_WINDOW | subprocess.CREATE_NEW_PROCESS_GROUP,
        close_fds=True, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
