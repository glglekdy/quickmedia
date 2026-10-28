# QuickMedia: 이름 변경 + 설치 파일 배포 설계

작성일: 2026-10-28
범위: 하위 작업 ① (배포 방식 + 이름 변경). yt-dlp 기능 확장은 하위 작업 ②로 따로 설계한다.

## 목표

- 앱 이름을 "유튜브 다운로더"에서 **QuickMedia**로 바꾼다.
- 단일 exe 배포를 RCM처럼 **설치 파일**(`QuickMedia-Setup-x.y.z.exe`) 배포로 바꾼다.
- 자동 업데이트를 설치 파일 기반으로 바꾸고, 기존 `YoutubeDownloader.exe`(v1.0.1 이하) 사용자가 따로 안내받지 않아도 새 설치판으로 넘어오게 한다.
- 유튜브 로고 모양 아이콘을 임시 자체 아이콘으로 바꾼다.

## 하지 않는 것

- yt-dlp 기능 확장(형식·자막·재생목록·후처리·쿠키·다른 사이트 등): 하위 작업 ②.
- 그래서 이번에는 "올바른 유튜브 링크가 아니에요" 같은 유튜브 전용 문구를 그대로 둔다. 다른 사이트 지원과 함께 ②에서 바꾼다.
- RCM의 비공개 보관 저장소(RCM-Archive)와 옛 릴리스 지우기.
- 코드 서명. 지금처럼 서명 없이 배포하고, README에 SmartScreen 경고 안내를 넣는다.
- 내부 Python 패키지 이름(`ytdl`)은 바꾸지 않는다.

## 이름

| 자리 | 값 |
|---|---|
| 창 제목, 설치 화면, 제어판 표시 이름, exe 설명 | `QuickMedia` |
| 앱 실행 파일 | `QuickMedia.exe` |
| 설치 파일 | `QuickMedia-Setup-<버전>.exe` |
| GitHub 저장소 | `glglekdy/quickmedia` |
| 기본 설치 폴더 | `%LOCALAPPDATA%\Programs\QuickMedia` |
| QSettings | 조직·앱 모두 `QuickMedia` (레지스트리 `HKCU\Software\QuickMedia\QuickMedia`) |
| 업데이트 임시 폴더 | `%TEMP%\QuickMedia-update` |

바꿀 곳: `app.py`(애플리케이션 이름·주석), `ytdl/main_window.py`(창 제목, 머리 글자, QSettings), `tools/version_info.py`, `build.ps1`, `.github/workflows/release.yml`, `ytdl/updater.py`(저장소 주소, User-Agent, 임시 폴더), `README.md`.

GitHub 저장소 이름은 사용자가 직접 바꾼다(Settings → Repository name). GitHub는 옛 주소(`glglekdy/youtube-downloader`)로 오는 웹·API 요청을 새 주소로 넘겨 주므로, 옛 앱의 업데이트 확인도 새 저장소의 릴리스를 본다. 그래서 **저장소 이름을 바꾼 뒤에** 첫 QuickMedia 태그를 push 한다.

## 빌드

- PyInstaller를 `--onefile`에서 `--onedir`로 바꾼다. 결과물은 `dist/QuickMedia/QuickMedia.exe`와 `_internal/` 폴더다.
  - 실행할 때마다 임시 폴더에 압축을 풀지 않아 빨리 켜진다.
  - `sys._MEIPASS`는 onedir에서도 `_internal` 폴더를 가리키므로 ffmpeg 위치(`downloader.BASE_DIR`)와 아이콘 경로(`app.py`)는 그대로 동작한다.
- 이어서 Inno Setup 6 컴파일러(`ISCC.exe`)로 `installer/QuickMedia.iss`를 컴파일해 `dist/QuickMedia-Setup-<버전>.exe`를 만든다. 버전은 `/DAppVersion=<버전>`으로 넘긴다.
- `build.ps1`
  - `ISCC.exe`를 PATH와 `Program Files (x86)\Inno Setup 6`, `%LOCALAPPDATA%\Programs\Inno Setup 6`에서 찾는다. 없으면 설치 방법(`winget install JRSoftware.InnoSetup`)을 알려 주고 멈춘다.
  - 끝나면 설치 파일 경로와 크기를 출력한다.
- 릴리스 워크플로
  - Inno Setup이 러너에 없으면 `choco install innosetup -y`로 설치한다.
  - `./build.ps1`을 실행한다.
  - `dist/QuickMedia-Setup-<버전>.exe` **하나만** 릴리스에 올린다. 옛 앱의 업데이터가 `.exe`로 끝나는 첫 자산을 받으므로, 릴리스에 exe가 둘 이상 있으면 안 된다.

## 설치 파일 (`installer/QuickMedia.iss`)

- `AppId`: 고정 GUID. 업데이트할 때 같은 앱으로 알아보고, 이전 설치 폴더를 그대로 쓰게 한다(`UsePreviousAppDir`, 기본값).
- `PrivilegesRequired=lowest`, `DefaultDirName={autopf}\QuickMedia`: 관리자 권한 없이 `%LOCALAPPDATA%\Programs\QuickMedia`에 설치된다. 폴더 선택 화면을 보여 줘서 다른 폴더로 바꿀 수 있다.
- 설치 화면 언어: 한국어(`compiler:Languages\Korean.isl`).
- 바로가기: 시작 메뉴는 항상 만들고, 바탕화면은 선택 항목(기본 체크).
- 제어판 "앱 제거" 목록에 이름 `QuickMedia`, 아이콘, 버전, 게시자 `glglekdy`로 등록한다.
- 앱 실행 항목 `[Run]`
  - 일반 설치: 마지막 화면에 "QuickMedia 실행" 체크박스(`postinstall skipifsilent`).
  - 조용한 설치(업데이트): 끝나면 앱을 바로 다시 띄운다(`skipifnotsilent nowait`).
- 설치 파일 아이콘과 제거 프로그램 아이콘은 새 `assets/icon.ico`를 쓴다.
- 제거할 때는 설치 폴더만 지운다. 받은 파일과 설정(레지스트리)은 남긴다.
- 옛 exe 정리 표시(아래 "기존 사용자 이전")를 `[Code]`에서 처리한다.

## 자동 업데이트 (`ytdl/updater.py`, `ytdl/update_dialog.py`)

흐름은 지금과 같다. 시작할 때 조용히 확인 → 새 버전이 있으면 창 → "지금 업데이트" → 받기 → 적용.

바뀌는 점:

1. **자산 고르기**: `.exe`로 끝나는 아무 파일이 아니라 `QuickMedia-Setup-*.exe` 이름에 맞는 자산을 고른다. 없으면 업데이트가 없는 것으로 본다.
2. **받기**: `%TEMP%\QuickMedia-update\QuickMedia-Setup-<버전>.exe`로 받는다. 크기와 SHA-256 확인은 지금과 같다.
3. **적용**: 도우미 PowerShell 스크립트가 **앱 프로세스 하나(자기 PID)만** 끝나기를 기다린 뒤 설치 파일을 `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART`로 실행한다. 설치가 끝나면 설치 파일의 `[Run]`이 앱을 다시 띄운다. 도우미는 설치 파일이 끝나면 받은 파일을 지운다.
   - onedir에는 부트로더 부모 프로세스가 없으므로 지금처럼 `getppid()`를 기다리면 안 된다. 부모가 탐색기라서 120초 타임아웃까지 멈춰 있게 된다.
   - `_relaunch_env()`(PyInstaller 환경 변수 지우기)는 그대로 둔다. 설치 파일과 다시 뜨는 앱이 옛 환경 변수를 물려받지 않게 한다.
4. `can_self_update()`는 지금처럼 "exe로 실행 중"인지만 본다.

## 기존 사용자 이전

### 옛 exe에서 넘어오는 과정

v1.0.1 앱은 최신 릴리스에서 `.exe` 자산을 받아 **자기 파일(`YoutubeDownloader.exe`)을 덮어쓰고 다시 실행**한다. 새 릴리스의 exe는 설치 파일 하나뿐이므로, 덮어쓴 뒤 다시 실행되는 것은 QuickMedia 설치 화면이다. 사용자는 평소처럼 설치를 마친다.

그러면 원래 자리에 설치 파일 사본(`YoutubeDownloader.exe`라는 이름)이 남는다. 이것을 치운다.

- 설치 파일 `[Code]`: 자기 경로(`{srcexe}`)의 파일 이름이 `YoutubeDownloader.exe`(대소문자 무시)이면, 설치를 마칠 때 그 전체 경로를 `HKCU\Software\QuickMedia\QuickMedia`의 `legacy_exe` 값으로 적는다.
- 앱 시작 시: `legacy_exe` 값이 있으면 백그라운드에서 그 파일을 지운다. 설치 파일이 아직 안 끝났을 수 있으므로 최대 30초 동안 몇 번 다시 시도한다. 지운 뒤나 파일이 이미 없으면 값을 지운다.
  - 안전장치: 파일 이름이 `YoutubeDownloader.exe`가 아니면 지우지 않고 값만 지운다.
- 옛 exe를 가리키던 바탕화면 바로가기는 깨진다. 새 설치가 바탕화면 바로가기를 새로 만드므로 README에 안내만 한다.

### 설정 옮기기

앱 시작 시 새 설정(`QuickMedia`)에 `migrated` 표시가 없으면, 옛 설정(`YoutubeDownloader/YoutubeDownloader`)의 모든 키를 새 설정으로 복사하고 `migrated=true`를 적는다. 옛 키는 지우지 않는다. 대상은 지금 있는 `folder`, `format`이다.

### 새 모듈

이전 처리(설정 옮기기, 옛 exe 지우기)는 `ytdl/migrate.py` 한 곳에 둔다. `app.py`에서 창을 띄우기 전에 `migrate.run()`을 부른다. 옛 exe 지우기만 백그라운드 스레드로 돈다. 판단 로직(옮길 키 고르기, 지워도 되는 경로인지)은 Qt 없이 테스트할 수 있는 함수로 뺀다.

## 아이콘 (임시)

- 빨간 둥근 사각형(앱 강조색 `#dc2626` 계열) 위에 흰 재생 삼각형과 그 아래 흰 다운로드 화살표를 그린다. 유튜브 로고 모양은 쓰지 않는다.
- `tools/make_icon.py`가 QPainter로 그려서 `ytdl/assets/icon.png`(256px)와 `assets/icon.ico`(16·24·32·48·64·128·256px)를 만든다. 나중에 정식 아이콘으로 바꿀 때는 이 두 파일만 교체하면 된다.
- README 상단 스크린샷(`Youtube_Downloader.png`)은 이번에 바꾸지 않는다. 새 화면은 하위 작업 ② 이후 다시 찍는다.

## 오류 처리

- 업데이트 실패(연결·크기·해시)는 지금처럼 업데이트 창에 문구로 보여 주고 "다시 시도"를 띄운다.
- 조용한 설치가 실패하면 Inno Setup이 기존 파일을 되돌린다. 이 경우 앱이 다시 뜨지 않으므로 사용자는 바로가기로 옛 버전을 다시 연다. 도우미는 설치 파일의 종료 코드가 0이 아니면 설치 폴더의 기존 `QuickMedia.exe`를 띄운다.
- 이전 처리(설정 옮기기, 옛 exe 지우기)의 실패는 앱 시작을 막지 않는다. 조용히 넘어가고 다음 실행에 다시 시도한다.

## 테스트

자동 테스트(pytest, Qt 없이 도는 함수만):

- 릴리스 자산 목록에서 `QuickMedia-Setup-*.exe` 고르기. 여러 자산 섞임, 없음, 대소문자.
- `parse_version` 기존 동작 유지.
- 옛 exe 경로 판단. `YoutubeDownloader.exe`만 지우고 다른 이름은 건너뛴다.
- 설정 옮기기. 새 설정이 비어 있을 때만 옮기고, 이미 옮겼으면 아무것도 안 한다.

수동 확인(로컬 Windows):

1. `./build.ps1` → 설치 파일 생성, 설치 → 바로가기·제어판 등록·실행 확인 → 제거 시 설정 유지 확인.
2. 업데이트: 낮은 버전으로 빌드해 설치하고, 업데이트 로직을 로컬 설치 파일로 돌려 조용한 설치와 재실행을 확인.
3. 옛 사용자 이전: 설치 파일을 `YoutubeDownloader.exe`로 이름 바꿔 실행 → 설치 → 앱 첫 실행 후 그 파일이 지워지고 옛 설정(저장 위치)이 그대로인지 확인.

`requirements.txt`는 앱 실행용으로 두고, 개발용 `pytest`는 README 개발 절에 적는다.
