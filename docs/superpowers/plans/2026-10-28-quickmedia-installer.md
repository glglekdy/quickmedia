# QuickMedia 이름 변경 + 설치 파일 배포 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** "유튜브 다운로더"를 QuickMedia로 바꾸고, 단일 exe 대신 Inno Setup 설치 파일로 배포·업데이트하며, 기존 v1.0.1 사용자를 자동으로 옮긴다.

**Architecture:** PyInstaller onedir 결과물을 Inno Setup이 `QuickMedia-Setup-<버전>.exe`로 묶는다. 업데이터는 그 설치 파일을 받아 조용히 실행한다. 이름·저장소 상수는 `ytdl/app_info.py`에, 옛 설정·옛 exe 처리는 `ytdl/migrate.py`에 둔다. 판단 로직은 Qt 없이 테스트할 수 있는 함수로 뺀다.

**Tech Stack:** Python 3.13 (CI) / PySide6-Essentials, yt-dlp, PyInstaller 6 (onedir), Inno Setup 6, PowerShell 5.1, pytest

**Spec:** `docs/superpowers/specs/2026-10-28-quickmedia-installer-design.md`

## Global Constraints

- 표시 이름 `QuickMedia`, 실행 파일 `QuickMedia.exe`, 설치 파일 `QuickMedia-Setup-<버전>.exe`, 저장소 `glglekdy/quickmedia`, 기본 설치 폴더 `%LOCALAPPDATA%\Programs\QuickMedia`, 업데이트 임시 폴더 `%TEMP%\QuickMedia-update`.
- QSettings 조직·앱 이름은 둘 다 `QuickMedia`이고, 옛 이름은 `YoutubeDownloader`/`YoutubeDownloader`다.
- 내부 패키지 이름 `ytdl`은 그대로 둔다. 유튜브 전용 오류 문구와 README 스크린샷도 그대로 둔다.
- `ytdl/__init__.py`는 릴리스 워크플로가 `__version__` 한 줄로 통째로 덮어쓴다. 여기에 다른 상수를 두지 않는다.
- 릴리스에는 `.exe` 자산을 **정확히 하나**(설치 파일)만 올린다.
- PowerShell 스크립트 파일은 `utf-8-sig`(BOM)로 쓴다. 5.1이 한글 경로를 깨뜨리지 않게 하기 위해서다.
- 커밋 메시지는 저장소 관례대로 한국어 한 줄로 쓰고, 끝에 `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`를 붙인다.
- 작업은 `main`이 아닌 `quickmedia-installer` 브랜치에서 한다.

## Review Focus

1. **한글·공백이 든 설치 경로**(예: `D:\내 프로그램\QuickMedia`): 업데이트 도우미가 경로를 따옴표로 감싸 설치 파일 실행과 실패 시 재실행에 성공해야 한다. → Task 2 테스트(스크립트 인자 목록), Task 6 수동 확인 2.
2. **옛 exe가 아직 잠겨 있음**: 설치 화면의 "실행"으로 앱이 뜰 때 설치 파일이 아직 안 끝났을 수 있다. 정리 함수는 잠깐 다시 시도해서 결국 지워야 한다. → Task 3 `test_delete_legacy_retries_while_locked`.
3. **옛 설치에서 "QuickMedia 실행" 체크를 끈 경우**: 레지스트리 표시가 남아 있다가 다음 실행 때 지워져야 한다. → Task 3 `test_cleanup_step_handles_pending_value`.
4. **옛 설정이 아예 없는 새 사용자**: 매 실행마다 옮기기를 다시 하지 않도록 `migrated`만 적고 끝나야 한다. → Task 3 `test_plan_with_empty_old_marks_migrated`.
5. **설치 파일 자산이 없는 릴리스**(옛 이름 exe만 있거나 자산 없음): 업데이트가 있다고 띄우지 않아야 한다. → Task 2 `test_pick_setup_asset_ignores_other_exe`.

---

### Task 1: 개발 환경, 테스트 틀, `app_info`

**Files:**
- Create: `ytdl/app_info.py`, `tests/__init__.py`(빈 파일), `tests/conftest.py`, `tests/test_app_info.py`
- Modify: `.gitignore`(`.pytest_cache/` 추가)

**Interfaces:**
- Produces: `ytdl.app_info.NAME = "QuickMedia"`, `EXE = "QuickMedia.exe"`, `REPO = "glglekdy/quickmedia"`, `SETTINGS = ("QuickMedia", "QuickMedia")`, `LEGACY_SETTINGS = ("YoutubeDownloader", "YoutubeDownloader")`, `LEGACY_EXE = "YoutubeDownloader.exe"`

- [ ] **Step 1: 브랜치를 만들고, 커밋되지 않은 테마 변경을 먼저 커밋한다**

```bash
git switch -c quickmedia-installer
git add ytdl/theme.py
git commit -m "강조색을 보라에서 빨강 계열로"   # + Co-Authored-By 줄
```

- [ ] **Step 2: 의존성을 설치한다** (이 PC에는 PySide6가 없다)

Run: `python -m pip install -r requirements.txt pytest pyinstaller`
Expected: 설치 성공, `python -c "import PySide6, yt_dlp"`가 오류 없음.

- [ ] **Step 3: 실패하는 테스트를 쓴다**

`tests/conftest.py`는 저장소 루트를 `sys.path`에 넣는다. `tests/test_app_info.py`:

```python
from ytdl import app_info

def test_names():
    assert app_info.NAME == "QuickMedia"
    assert app_info.EXE == "QuickMedia.exe"
    assert app_info.REPO == "glglekdy/quickmedia"
    assert app_info.SETTINGS == ("QuickMedia", "QuickMedia")
    assert app_info.LEGACY_SETTINGS == ("YoutubeDownloader", "YoutubeDownloader")
    assert app_info.LEGACY_EXE == "YoutubeDownloader.exe"
```

- [ ] **Step 4: 실패를 확인한다**

Run: `python -m pytest tests -v` → FAIL (`ImportError: cannot import name 'app_info'`)

- [ ] **Step 5: `ytdl/app_info.py`를 만든다** (모듈 설명 한 줄 + 위 상수)

- [ ] **Step 6: 통과를 확인한다**: `python -m pytest tests -v` → PASS

- [ ] **Step 7: 커밋** — `테스트 틀과 앱 이름 상수 추가`

---

### Task 2: 업데이터를 설치 파일 기반으로

**Files:**
- Modify: `ytdl/updater.py` (전체적으로; `REPO`/`LATEST_URL`/`UPDATE_DIR`, `_request` User-Agent, `fetch_latest`, `download`, `_APPLY_SCRIPT`, `launch_apply`, 모듈 설명)
- Test: `tests/test_updater.py`

**Interfaces:**
- Consumes: `app_info.NAME`, `app_info.REPO`
- Produces:
  - `pick_setup_asset(assets: list[dict]) -> dict | None`: `name`이 정규식 `^QuickMedia-Setup-.+\.exe$`(대소문자 무시)에 맞는 첫 자산.
  - `apply_command(script: Path, pid: int, setup: Path, exe: Path) -> list[str]`: 도우미를 띄울 PowerShell 인자 목록.
  - `launch_apply(setup: Path) -> None`: 이름은 그대로이고 의미만 "설치 파일 적용"으로 바뀐다. `update_dialog.py`는 바꿀 필요가 없다.
  - `UPDATE_DIR = Path(tempfile.gettempdir()) / "QuickMedia-update"`, 받은 파일 이름 `QuickMedia-Setup-<버전>.exe`.

- [ ] **Step 1: 실패하는 테스트를 쓴다**

```python
def test_pick_setup_asset_matches_setup_name():
    assets = [{"name": "notes.txt"}, {"name": "QuickMedia-Setup-1.2.0.exe", "browser_download_url": "u"}]
    assert pick_setup_asset(assets)["browser_download_url"] == "u"

def test_pick_setup_asset_case_insensitive():
    assert pick_setup_asset([{"name": "quickmedia-setup-1.2.0.EXE"}]) is not None

def test_pick_setup_asset_ignores_other_exe():
    assert pick_setup_asset([{"name": "YoutubeDownloader.exe"}, {"name": "QuickMedia.exe"}]) is None
    assert pick_setup_asset([]) is None

def test_parse_version_unchanged():
    assert parse_version("v1.2.0") == (1, 2)
    assert parse_version("1.2.3-beta") == (1, 2, 3)
    assert parse_version("") == ()

def test_apply_command_passes_paths_verbatim():
    cmd = apply_command(Path("C:/t/a.ps1"), 42, Path("C:/임시 폴더/QuickMedia-Setup-2.0.0.exe"),
                        Path("D:/내 프로그램/QuickMedia/QuickMedia.exe"))
    assert cmd[:2] == ["powershell.exe", "-NoProfile"]
    assert cmd[cmd.index("-WaitPid") + 1] == "42"
    assert cmd[cmd.index("-Setup") + 1] == str(Path("C:/임시 폴더/QuickMedia-Setup-2.0.0.exe"))
    assert cmd[cmd.index("-Exe") + 1] == str(Path("D:/내 프로그램/QuickMedia/QuickMedia.exe"))
```

- [ ] **Step 2: 실패를 확인한다**: `python -m pytest tests/test_updater.py -v` → FAIL (`ImportError: pick_setup_asset`)

- [ ] **Step 3: 구현한다**
  - `fetch_latest()`는 기존의 `.exe` 고르기 대신 `pick_setup_asset(data.get("assets") or [])`를 쓴다.
  - `apply_command`는 기존 인자(`-NoProfile -NonInteractive -ExecutionPolicy Bypass -WindowStyle Hidden -File <script>`)에 `-WaitPid <pid> -Setup <setup> -Exe <exe>`를 붙인다. 리스트로 넘기므로 공백·한글 경로는 `subprocess`가 감싼다.
  - `launch_apply`는 `pid=os.getpid()`만 넘긴다. `getppid()`를 빼는데, onedir에서는 부모가 탐색기이기 때문이다. `exe=Path(sys.executable).resolve()`이고, `_relaunch_env()`와 `creationflags`는 그대로 둔다.
  - `_APPLY_SCRIPT`(새 본문):

```powershell
param([int]$WaitPid, [string]$Setup, [string]$Exe)
Wait-Process -Id $WaitPid -Timeout 120 -ErrorAction SilentlyContinue
$p = Start-Process -FilePath $Setup -ArgumentList '/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART' -Wait -PassThru
# 성공하면 설치 파일의 [Run] 이 앱을 다시 띄운다. 실패하면 Inno 가 되돌려 둔 기존 앱을 띄운다.
if ($p.ExitCode -ne 0 -and (Test-Path $Exe)) { Start-Process -FilePath $Exe -WorkingDirectory (Split-Path $Exe) }
Remove-Item $Setup -Force -ErrorAction SilentlyContinue
```

- [ ] **Step 4: 통과를 확인한다**: `python -m pytest tests -v` → 모두 PASS

- [ ] **Step 5: 커밋** — `업데이트를 설치 파일 방식으로 바꿈`

---

### Task 3: 기존 사용자 이전 (`ytdl/migrate.py`)

**Files:**
- Create: `ytdl/migrate.py`
- Test: `tests/test_migrate.py`

**Interfaces:**
- Consumes: `app_info.SETTINGS`, `app_info.LEGACY_SETTINGS`, `app_info.LEGACY_EXE`
- Produces:
  - `plan_settings_migration(old: dict[str, object], new_keys: set[str]) -> dict[str, object]`: `"migrated"`가 `new_keys`에 있으면 `{}`. 아니면 `old`의 모든 키와 `"migrated": True`.
  - `is_legacy_exe(path: str) -> bool`: 파일 이름이 `YoutubeDownloader.exe`인지(대소문자 무시).
  - `delete_legacy(path: str, attempts: int = 15, delay: float = 2.0) -> bool`: 옛 exe가 아니면 지우지 않고 `True`(정리 끝). 파일이 없으면 `True`. `PermissionError`/`OSError`면 `delay`초 쉬고 다시 시도하고, 끝내 못 지우면 `False`. 기본값은 최대 30초.
  - `cleanup_step(settings) -> threading.Thread | None`: `settings`에 `legacy_exe`가 있으면 데몬 스레드를 시작해 반환한다. 스레드는 `delete_legacy` 결과가 `True`일 때 **스레드 안에서 새로 만든** QSettings로 `legacy_exe`를 지운다.
  - `run() -> None`: `QSettings(*SETTINGS)`와 `QSettings(*LEGACY_SETTINGS)`로 설정을 옮긴 뒤 `cleanup_step`을 부른다. 모든 예외를 삼킨다(스펙: 앱 시작을 막지 않음).

- [ ] **Step 1: 실패하는 테스트를 쓴다**

```python
def test_plan_copies_old_keys_once():
    assert plan_settings_migration({"folder": "D:/v", "format": "mp3"}, set()) == \
        {"folder": "D:/v", "format": "mp3", "migrated": True}
    assert plan_settings_migration({"folder": "D:/v"}, {"migrated", "folder"}) == {}

def test_plan_with_empty_old_marks_migrated():
    assert plan_settings_migration({}, set()) == {"migrated": True}

def test_is_legacy_exe():
    assert is_legacy_exe(r"C:\Users\a\Downloads\YoutubeDownloader.exe")
    assert is_legacy_exe("c:/x/youtubedownloader.EXE")
    assert not is_legacy_exe(r"C:\x\QuickMedia-Setup-2.0.0.exe")

def test_delete_legacy_removes_file(tmp_path):
    f = tmp_path / "YoutubeDownloader.exe"; f.write_bytes(b"x")
    assert delete_legacy(str(f), attempts=1, delay=0) and not f.exists()

def test_delete_legacy_skips_other_names(tmp_path):
    f = tmp_path / "Other.exe"; f.write_bytes(b"x")
    assert delete_legacy(str(f), attempts=1, delay=0) and f.exists()

def test_delete_legacy_missing_file_is_done(tmp_path):
    assert delete_legacy(str(tmp_path / "YoutubeDownloader.exe"), attempts=1, delay=0)

def test_delete_legacy_retries_while_locked(tmp_path):
    f = tmp_path / "YoutubeDownloader.exe"; f.write_bytes(b"x")
    fh = f.open("rb")                      # Windows 에서는 열린 파일을 지울 수 없다
    threading.Timer(0.3, fh.close).start()
    assert delete_legacy(str(f), attempts=10, delay=0.1) and not f.exists()

def test_cleanup_step_handles_pending_value(tmp_path):
    f = tmp_path / "YoutubeDownloader.exe"; f.write_bytes(b"x")
    s = QSettings(str(tmp_path / "s.ini"), QSettings.Format.IniFormat)
    s.setValue("legacy_exe", str(f)); s.sync()
    cleanup_step(s).join(5)
    assert not f.exists()
```

마지막 테스트에서 스레드가 스스로 만드는 QSettings가 같은 ini 파일을 보도록, `cleanup_step`은 받은 `settings`의 `fileName()`과 `format()`으로 새 QSettings를 만든다.

- [ ] **Step 2: 실패를 확인한다**: `python -m pytest tests/test_migrate.py -v` → FAIL (`ModuleNotFoundError: ytdl.migrate`)

- [ ] **Step 3: `ytdl/migrate.py`를 구현한다** (위 Interfaces 그대로)

- [ ] **Step 4: 통과를 확인한다**: `python -m pytest tests -v` → 모두 PASS

- [ ] **Step 5: 커밋** — `옛 설정과 옛 exe 를 정리하는 이전 모듈 추가`

---

### Task 4: 앱 안의 이름 바꾸기와 이전 연결

**Files:**
- Modify: `app.py` (모듈 설명, `setApplicationName(app_info.NAME)`, `theme.apply` 다음·`MainWindow()` 앞에서 `migrate.run()`)
- Modify: `ytdl/main_window.py:111-116` (`setWindowTitle(app_info.NAME)`, `QSettings(*app_info.SETTINGS)`), `:131` (머리 글자 `QLabel(app_info.NAME)`), 모듈 설명
- Modify: `tools/version_info.py` (`NAME = "QuickMedia"`, `InternalName` `QuickMedia`, `OriginalFilename` `QuickMedia.exe`)

**Interfaces:**
- Consumes: `app_info.NAME`, `app_info.SETTINGS`, `migrate.run()`

- [ ] **Step 1: 위 수정을 한다.** 안내 문구 "링크를 붙여넣고 형식을 고른 뒤 다운로드를 누르세요."는 그대로 둔다.

- [ ] **Step 2: 남은 옛 이름이 없는지 확인한다**

Run: `git grep -n -e "유튜브 다운로더" -e "YoutubeDownloader" -- ':!docs' ':!tests'`
Expected: `ytdl/app_info.py`(LEGACY 상수) 외에는 `build.ps1`, `.github/workflows/release.yml`, `README.md`만 남는다. 이 셋은 Task 6·7에서 바꾼다.

- [ ] **Step 3: 소스로 실행해 확인한다**

Run: `python app.py`
Expected: 창 제목과 머리 글자가 `QuickMedia`이고, 저장 위치가 옛 앱에서 쓰던 폴더 그대로다(옛 설정이 있을 때). `reg query HKCU\Software\QuickMedia\QuickMedia`에 `migrated`가 보인다.

- [ ] **Step 4: 테스트**: `python -m pytest tests -v` → PASS

- [ ] **Step 5: 커밋** — `앱 이름을 QuickMedia 로 바꾸고 시작할 때 이전 처리`

---

### Task 5: 임시 아이콘

**Files:**
- Create: `tools/make_icon.py`
- Modify (생성물): `ytdl/assets/icon.png`, `assets/icon.ico`
- Test: `tests/test_make_icon.py`

**Interfaces:**
- Produces: `tools/make_icon.py`의 `build_ico(pngs: dict[int, bytes]) -> bytes`(PNG 압축 항목으로 된 ICO: 6바이트 헤더 + 16바이트 디렉터리 항목 × N + PNG 데이터, 256은 너비·높이 바이트 0) 및 `render(size: int) -> bytes`(QPainter로 그린 PNG). 실행: `python tools/make_icon.py`가 두 파일을 쓴다.

- [ ] **Step 1: 실패하는 테스트를 쓴다**

```python
def test_build_ico_layout():
    pngs = {16: b"\x89PNG-a", 256: b"\x89PNG-bb"}
    data = build_ico(pngs)
    assert data[:6] == struct.pack("<HHH", 0, 1, 2)
    w, h, _, _, planes, bpp, size, offset = struct.unpack_from("<BBBBHHII", data, 6)
    assert (w, h, planes, bpp, size, offset) == (16, 16, 1, 32, 6, 6 + 16 * 2)
    w2, h2, *_ , size2, offset2 = struct.unpack_from("<BBBBHHII", data, 22)
    assert (w2, h2, size2) == (0, 0, 7) and data[offset2:offset2 + 7] == b"\x89PNG-bb"
```

- [ ] **Step 2: 실패를 확인한다**: `python -m pytest tests/test_make_icon.py -v` → FAIL

- [ ] **Step 3: 구현한다.** 그림 규칙(스펙): 투명 배경, 둥근 사각형(가장자리 여백 크기의 6%, 모서리 반경 22%)을 `#dc2626`으로 채우고, 흰 재생 삼각형(가운데에서 약간 위)과 그 아래 흰 다운로드 화살표(세로 막대 + 아래 화살촉)를 그린다. 안티에일리어싱을 켠다. 크기는 16·24·32·48·64·128·256이다. `QGuiApplication`이 필요하므로 `QT_QPA_PLATFORM=offscreen`을 설정한 뒤 만든다.

- [ ] **Step 4: 통과 확인 후 아이콘을 만든다**

Run: `python -m pytest tests -v` → PASS; `python tools/make_icon.py`
Expected: 두 파일이 갱신된다. `ytdl/assets/icon.png`를 열어 보고 16px로 줄여도 재생 버튼과 화살표가 알아보이는지 눈으로 확인한다.

- [ ] **Step 5: 커밋** — `유튜브 로고 대신 임시 QuickMedia 아이콘`

---

### Task 6: onedir 빌드 + Inno Setup 설치 파일

**Files:**
- Create: `installer/QuickMedia.iss`
- Modify: `build.ps1`

**Interfaces:**
- Consumes: `ytdl/__init__.py`의 `__version__`, `assets/icon.ico`, `tools/version_info.py`
- Produces: `dist/QuickMedia/QuickMedia.exe`(+`_internal/`), `dist/QuickMedia-Setup-<버전>.exe`. 사용법: `./build.ps1 [-FFmpeg <경로>]`

- [ ] **Step 1: 빌드 도구를 설치한다** (이 PC에는 둘 다 없다. 설치 전에 사용자에게 알린다.)

Run: `winget install JRSoftware.InnoSetup` 그리고 `winget install Gyan.FFmpeg`
Expected: `ISCC.exe`가 `%LOCALAPPDATA%\Programs\Inno Setup 6` 또는 `Program Files (x86)\Inno Setup 6`에 있다. `Korean.isl`이 그 안의 `Languages\`에 있는지 확인한다. 없으면 Inno Setup 저장소의 공식 번역 파일을 `installer/Korean.isl`로 넣고 `.iss`에서 그 경로를 쓴다.

- [ ] **Step 2: `installer/QuickMedia.iss`를 쓴다**. 정해진 값:

```
#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
[Setup]
AppId={{8F3C2A71-5D4E-4B9A-9C61-2E7D0F4A3B85}
AppName=QuickMedia
AppVersion={#AppVersion}
AppPublisher=glglekdy
DefaultDirName={autopf}\QuickMedia
DefaultGroupName=QuickMedia
PrivilegesRequired=lowest
OutputDir=..\dist
OutputBaseFilename=QuickMedia-Setup-{#AppVersion}
SetupIconFile=..\assets\icon.ico
UninstallDisplayIcon={app}\QuickMedia.exe
WizardStyle=modern
Compression=lzma2
SolidCompression=yes
[Languages]   Name: "korean"; MessagesFile: "compiler:Languages\Korean.isl"
[Tasks]       desktopicon (기본 체크)
[Files]       Source: "..\dist\QuickMedia\*"; DestDir: "{app}"; Flags: recursesubdirs ignoreversion
[Icons]       {group}\QuickMedia, {autodesktop}\QuickMedia (Tasks: desktopicon) → {app}\QuickMedia.exe
[Run]         "{app}\QuickMedia.exe"; Description: "QuickMedia 실행"; Flags: postinstall nowait skipifsilent
              "{app}\QuickMedia.exe"; Flags: nowait skipifnotsilent
```

`[Code]`: `CurStepChanged(CurStep)`에서 `CurStep = ssPostInstall`이고 `CompareText(ExtractFileName(ExpandConstant('{srcexe}')), 'YoutubeDownloader.exe') = 0`이면 `RegWriteStringValue(HKCU, 'Software\QuickMedia\QuickMedia', 'legacy_exe', ExpandConstant('{srcexe}'))`.

- [ ] **Step 3: `build.ps1`을 바꾼다**
  - 맨 위 설명을 QuickMedia·설치 파일 기준으로 고친다.
  - PyInstaller에서 `--onefile`을 `--onedir`로, `--name`을 `QuickMedia`로 바꾼다. 나머지 인자는 그대로 둔다.
  - 버전은 `ytdl/__init__.py`에서 정규식으로 읽는다.
  - `ISCC.exe` 찾기: `Get-Command ISCC` → `${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe` → `$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe` 순서로 찾는다. 없으면 `throw "Inno Setup 6 이 필요해요: winget install JRSoftware.InnoSetup"`.
  - `& $iscc "/DAppVersion=$version" installer/QuickMedia.iss`를 실행하고 종료 코드를 확인한다.
  - 마지막에 `dist/QuickMedia-Setup-$version.exe`의 크기를 출력한다.

- [ ] **Step 4: 빌드한다**

Run: `./build.ps1` (PATH에 ffmpeg가 없으면 `-FFmpeg <경로>`)
Expected: `완료: dist/QuickMedia-Setup-1.0.1.exe (NN MB)`가 출력되고, `dist/QuickMedia/_internal/ffmpeg.exe`가 있다.

- [ ] **Step 5: 설치 → 실행 → 제거를 수동으로 확인한다**
  1. 설치 파일을 실행한다. 한국어 화면에서 기본 폴더가 `%LOCALAPPDATA%\Programs\QuickMedia`이고, UAC 창이 뜨지 않는지 본다.
  2. 바탕화면·시작 메뉴 바로가기가 생기고, "설정 → 앱"에 `QuickMedia 1.0.1`이 새 아이콘과 함께 보이는지 본다.
  3. 실행해서 짧은 영상 하나를 MP3로 받는다. `_internal`의 ffmpeg가 쓰이는지 확인하는 단계다.
  4. 제거한 뒤 설치 폴더가 사라지고 `HKCU\Software\QuickMedia\QuickMedia`는 남는지 본다.

- [ ] **Step 6: 업데이트 경로와 한글 경로를 수동으로 확인한다** (Review Focus 1)
  1. `D:\내 프로그램\QuickMedia`처럼 공백과 한글이 든 폴더에 설치한다.
  2. `ytdl/__init__.py`를 잠시 `1.0.2`로 바꿔 다시 빌드하고 원래대로 되돌린다.
  3. 다음을 실행한다.
     ```
     python -c "import sys; sys.frozen=True; sys.executable=r'D:\내 프로그램\QuickMedia\QuickMedia.exe'; from pathlib import Path; from ytdl import updater; updater.launch_apply(Path(r'dist\QuickMedia-Setup-1.0.2.exe').resolve())"
     ```
  4. 화면 없이 설치가 끝나고, 같은 폴더의 앱이 1.0.2로 다시 뜨는지 본다(파일 속성 → 버전). 받은 설치 파일이 지워졌는지도 확인한다.

- [ ] **Step 7: 커밋** — `onedir 빌드와 Inno Setup 설치 파일`

---

### Task 7: 릴리스 워크플로와 README

**Files:**
- Modify: `.github/workflows/release.yml`, `README.md`

- [ ] **Step 1: 워크플로를 바꾼다**
  - 맨 위 설명을 "설치 파일을 빌드해 올린다"로 고친다.
  - "빌드" 단계 앞에 `Inno Setup 설치` 단계를 넣는다(`if (-not (Get-Command ISCC -ErrorAction SilentlyContinue) -and -not (Test-Path "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe")) { choco install innosetup -y --no-progress }`).
  - 릴리스 단계의 `files:`를 `dist/QuickMedia-Setup-*.exe`로 바꾼다. 이 패턴은 exe 하나에만 맞는다.

- [ ] **Step 2: README를 고친다**
  - 제목을 `# QuickMedia`로 바꾼다.
  - 받기 링크를 `https://github.com/glglekdy/quickmedia/releases/latest`로 바꾸고, `QuickMedia-Setup-x.y.z.exe`를 받아 설치한다고 쓴다.
  - SmartScreen 안내(RCM README 문구와 같은 결: "추가 정보 → 실행")를 넣는다.
  - 기존 사용자 안내를 한 줄 넣는다: "유튜브 다운로더에서 업데이트하면 설치 화면이 떠요. 설치를 마치면 옛 파일은 자동으로 지워지고 설정은 그대로예요. 옛 바로가기는 지우고 새 바로가기를 쓰세요."
  - 개발 절에 `pip install pytest`와 `python -m pytest`, Inno Setup 설치(`winget install JRSoftware.InnoSetup`)를 적고, `./build.ps1`의 결과물을 `dist/QuickMedia-Setup-<버전>.exe`로 고친다.
  - 릴리스 절에는 "저장소 이름을 `quickmedia`로 바꾼 뒤 태그를 push"라고 순서를 적는다.

- [ ] **Step 3: 확인한다**

Run: `git grep -n -e "유튜브 다운로더" -e "YoutubeDownloader" -e "youtube-downloader" -- ':!docs' ':!tests'`
Expected: `ytdl/app_info.py`의 LEGACY 상수, `installer/QuickMedia.iss`의 `[Code]`, README의 기존 사용자 안내만 남는다.

Run: `python -c "import yaml,sys; yaml.safe_load(open('.github/workflows/release.yml', encoding='utf-8'))"` (`pip install pyyaml`이 필요하면 설치) → 오류 없음.

- [ ] **Step 4: 커밋** — `릴리스를 QuickMedia 설치 파일로, README 갱신`

---

### Task 8: 옛 사용자 이전 수동 확인

**Files:** 없음 (검증만)

- [ ] **Step 1: 옛 설정을 준비한다**: 옛 v1.0.1을 소스로 실행할 필요 없이 `reg add HKCU\Software\YoutubeDownloader\YoutubeDownloader /v folder /d "D:\옛폴더" /f`를 실행한다. 이미 Task 4에서 이전이 끝났다면 `reg delete HKCU\Software\QuickMedia /f`로 새 설정을 먼저 비운다.

- [ ] **Step 2: 옛 앱의 업데이트를 흉내 낸다**: `dist/QuickMedia-Setup-1.0.1.exe`를 `%USERPROFILE%\Downloads\YoutubeDownloader.exe`로 복사해 실행하고, 설치하고, 마지막 화면에서 "QuickMedia 실행"에 체크한다.

Expected: 앱이 뜨고 30초 안에 `Downloads\YoutubeDownloader.exe`가 사라진다. 저장 위치는 `D:\옛폴더`이고, `reg query HKCU\Software\QuickMedia\QuickMedia /v legacy_exe`는 "찾을 수 없음"이다.

- [ ] **Step 3: "실행" 체크를 끈 경우를 확인한다** (Review Focus 3): Step 2를 반복하되 체크를 끈다. 설치가 끝나면 `legacy_exe` 값이 있고 파일도 남아 있다. 바탕화면 바로가기로 앱을 켜면 파일과 값이 모두 사라진다.

- [ ] **Step 4: 정리**: 테스트용 레지스트리 키(`HKCU\Software\YoutubeDownloader`의 테스트 값)와 설치본을 지운다. 이 확인 결과는 최종 보고에 그대로 적는다.
