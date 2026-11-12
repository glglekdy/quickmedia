# QuickMedia: Rust 재작성 설계

작성일: 2026-11-12
범위: Windows 앱을 Python + PySide6 에서 Rust + Slint 로 다시 만든다. 안드로이드 앱은 지원을 끝내고 지운다. 버전은 2.0.0.

## 목표

- 설치 용량, 켜지는 속도, 메모리 사용량을 모두 줄인다. Qt 와 Python 런타임을 뺀다.
- 기능과 화면은 1.2.0 과 같다. 새 기능은 **yt-dlp 갱신** 하나뿐이다.
- 설치된 1.x 앱이 평소처럼 2.0.0 으로 자동 업데이트되고, 설정·받은 기록이 그대로 남는다.

## 하지 않는 것

- ffmpeg 줄이기. 지금 빌드(BtbN win64-gpl)를 그대로 넣는다. 구간만 받기(`--force-keyframes-at-cuts`)가 재인코딩을 하므로 인코더가 빠진 빌드로 바꾸면 기능이 깨질 수 있다. 줄이려면 따로 한다.
- yt-dlp 자동 갱신(앱을 켤 때나 주기적으로). 사용자가 누를 때만 갱신한다.
- 화면 구성·문구 바꾸기, 새 옵션 더하기.
- Windows 외 운영체제.
- 저장소 Secrets 의 안드로이드 키스토어 지우기(사람이 GitHub 설정에서 한다).

## 정한 것

| 항목 | 결정 | 이유 |
|---|---|---|
| UI 도구 | Slint | 한글 조합 입력이 안정적이고 exe·메모리가 작다. Tauri 는 WebView2 가 메모리를 많이 쓰고, egui/iced 는 한글 입력이 불안하다. |
| yt-dlp | 공식 `yt-dlp.exe` 를 자식 프로세스로 실행 | 추출기는 Python 이고 수시로 바뀐다. Rust 로 옮길 수 없다. |
| yt-dlp 배포 | 설치 파일에 포함 + 앱 안에서 갱신 | 설치 직후 바로 쓰고, 사이트가 바뀌어도 앱 릴리즈 없이 고친다. |
| 설치 파일 | Inno Setup, 같은 AppId·파일 이름 | 1.x 의 자동 업데이트가 그대로 동작한다. |
| 스레드 | 작업마다 `std::thread`, 결과는 `slint::invoke_from_event_loop` 로 UI 스레드에 | 비동기 런타임 없이 충분하다. |

쓰는 크레이트: `slint`, `serde`/`serde_json`, `winreg`, `ureq`, `sha2`, `windows`(Job Object, 클립보드 등 필요한 것만). 폴더·파일 고르기는 `rfd`.

## 달라지는 점

1. 다운로드 시작이 건당 1~2초쯤 늦어질 수 있다. 공식 `yt-dlp.exe` 는 실행할 때마다 임시 폴더에 자기를 푼다. 재생목록 항목은 정보 조회를 건너뛰어 실행을 한 번으로 줄인다.
2. 콤보 상자·체크 상자·스크롤 막대는 Slint 위에 직접 스타일을 입힌다. 색과 간격은 맞추지만 픽셀까지 같지는 않다.
3. 고급 옵션(yt-dlp 인자) 검사는 yt-dlp 를 실행해 확인하므로 결과가 잠깐 뒤에 나온다.
4. 앱을 닫으면 yt-dlp 와 ffmpeg 가 함께 끝난다(1.x 는 스레드만 강제 종료했다).
5. 업데이트 화면 아래에 "Made with Slint" 를 표시한다(Slint 무료 라이선스 조건).

## 저장소 구조

```
Cargo.toml            버전은 여기 한 곳
build.rs              Slint 컴파일, exe 아이콘·버전 리소스
src/
  main.rs             진입점. 테마, 이전 처리, 창, 업데이트 확인
  app_info.rs         이름, 저장소, 레지스트리 경로
  options/            모델, 프리셋, 인자 만들기, 검사, 요약 문구
  ytdlp/              실행 파일 찾기, 정보 조회, 받기, 진행률, 오류 문구
  queue.rs            동시 N개 대기열
  store.rs            레지스트리 설정 읽기·쓰기
  migrate.rs          옛 유튜브 다운로더 설정 옮기기, 옛 exe 지우기
  site.rs             사이트 이름, 아이콘 받기·캐시
  updater.rs          앱 업데이트
  ytdlp_update.rs     yt-dlp 갱신
  ui.rs               Slint 와 로직 잇기
ui/                   *.slint (theme, sidebar, download, options_panel, settings, update)
assets/               icon.ico, 아이콘 svg·png
tests/fixtures/       yt-dlp 출력 샘플
```

`options`, `queue`, `ytdlp` 의 파싱, `updater` 의 판단 부분은 Slint 와 Windows API 없이 테스트할 수 있어야 한다.

지우는 것: `ytdl/`, `tests/*.py`, `app.py`, `requirements.txt`, `run.bat`, `tools/`, `android/`, `.gitignore`·`.gitattributes` 의 안드로이드·Python 줄. `docs/` 의 지난 문서와 지난 릴리즈 노트는 둔다.

## 호환

- **설정**: `HKCU\Software\QuickMedia\QuickMedia` 를 그대로 쓴다. 최상위에 `folder`, `format`(옛 값), `migrated`, `legacy_exe`, 하위 키 `options`·`settings` 에 필드 이름 그대로. QSettings 가 쓴 값은 문자열(`"true"`/`"false"`, 숫자 문자열)일 수도 DWORD 일 수도 있으므로 둘 다 읽는다. 목록 밖의 값·범위 밖의 숫자는 기본값으로 둔다(지금 `_coerce` 와 같다). 쓸 때는 문자열로 쓴다.
- **데이터 폴더**: `%APPDATA%\QuickMedia` (`archive.txt`, `favicons\`, 새로 `bin\`).
- **이전 처리**: `HKCU\Software\YoutubeDownloader\YoutubeDownloader` 의 값을 한 번 옮기고, `legacy_exe` 가 가리키는 `YoutubeDownloader.exe` 를 지운다. 실패해도 앱은 켜진다.
- **업데이트**: `releases/latest` 에서 `QuickMedia-Setup-*.exe` 를 고르는 규칙, 버전 비교(끝의 0 은 뗀다)가 같다.

## 다운로드 흐름

1. **넣기**: 링크가 비었거나 옵션이 틀리면 지금과 같은 안내. 누른 순간의 옵션·설정·폴더를 복사해 작업에 붙인다. 대기열은 동시 N개(설정의 1~5), 줄여도 돌던 작업은 끝까지 간다.
2. **정보 조회**(사용자가 넣은 링크만): `yt-dlp --ignore-config <앱 인자> <고급 인자> -J --flat-playlist <링크>`. 받은 기록(`--download-archive`)은 이때 넘기지 않는다.
   - JSON 의 `_type` 이 `playlist` 면 항목을 펼친다. 항목마다 `url`(없으면 `webpage_url`), `title`, `playlist_index`(없으면 목록 위치). 받은 기록 건너뛰기가 켜져 있으면 Rust 가 `archive.txt` 를 읽어 `<ie_key 소문자> <id>` 로 대조해 건너뛴 개수를 센다. 200개를 넘으면 묻는다. 원래 카드는 "재생목록 · N개 추가(M개 건너뜀)" 로 끝난다.
   - 아니면 제목, `extractor_key`, `webpage_url`, `requested_formats` 개수(스트림 수, 없으면 1), 파일 이름을 얻는다. 받은 기록에 있으면 "이미 받음" 으로 끝난다.
3. **이름 번호 붙이기**(덮어쓰기가 꺼져 있고 템플릿이 `.%(ext)s` 로 끝날 때): 조회 결과의 파일 이름과, 변환을 마친 최종 확장자(음원이면 고른 형식, 영상이면 고른 파일 형식)로 바꾼 이름 중 하나라도 있으면 템플릿 줄기에 ` (1)`, ` (2)` … 를 붙여 빈 이름을 찾는다. 템플릿에 넣을 때 `%` 는 `%%` 로.
4. **받기**: 같은 인자에 `--no-playlist`(항목일 때), `--newline`, `--no-warnings`, `--encoding utf-8`, `--ffmpeg-location <설치 폴더>`, `--progress-template`(받은 바이트·전체 바이트·추정 전체·속도를 구분자로 한 줄), `--print after_move:filepath`, `--no-simulate` 를 더해 실행한다. 재생목록 항목은 2~3 을 건너뛰고(펼칠 때 받은 제목을 쓴다) 스트림 수는 진행률 줄을 보며 센다.
5. **진행률**: `(끝난 스트림 + 지금 스트림 비율) / 스트림 수`. 뒤로 가지 않게 하고, 0.12초 안에 5%p 미만으로 변하면 알리지 않는다. 문구는 `받는 중 · <속도>`, 마지막 스트림이 끝나면 `파일 정리 중…`.
6. **끝**: 종료 코드 0 이고 경로가 찍혔으면 완료(카드를 누르면 탐색기에서 보기). 경로가 없으면 "이미 받음". 아니면 stderr 의 `ERROR:` 줄을 지금 `friendly_error` 표로 바꿔 보인다. 표에 없는 오류는 앞 160자에 "업데이트 화면에서 yt-dlp 갱신을 눌러 보세요" 를 덧붙인다.

프로세스는 창 없이(`CREATE_NO_WINDOW`) 띄우고, 앱이 가진 Job Object(`KILL_ON_JOB_CLOSE`)에 넣는다. 환경 변수 `PYTHONUTF8=1`, `PYTHONIOENCODING=utf-8` 를 준다.

**먼저 검증할 것**(구현 첫 단계): `-J` 결과에 파일 이름이 들어 있는지, `--progress-template` 과 `--print after_move:filepath` 가 한 실행에서 함께 동작하는지, 한글 제목이 깨지지 않는지. `-J` 에 파일 이름이 없으면 조회 때 `--print filename` 을 쓰는 실행으로 바꾼다.

## 인자 만들기와 검사

`build_args`, `split_extra`, `_safe_folder`, `_playlist_prefix`, 프리셋, `describe`, `changed_summary` 를 그대로 옮긴다. 1.x 가 인자를 dict 로 바꾼 뒤 덧씌우던 값은 인자로 바꾼다.

- 자막을 영상에 넣을 때(`--embed-subs`)는 `--write-subs` 를 함께 준다. `--write-auto-subs` 만 있으면 일반 자막이 꺼진다. 1.x 는 넣은 뒤 자막 파일을 남기지 않았으므로, 남는지 실제로 확인하고 남으면 지운다.
- 고급 인자는 맨 뒤에 붙여 사용자 값이 이긴다.

yt-dlp 유틸에 기대던 검사는 Rust 로 쓴다: 구간 시각(`parse_duration`: 초, `mm:ss`, `hh:mm:ss`), 속도 제한(`parse_bytes`: `500K`, `5M`, `1.5M`), 재생목록 범위(`1-5,8,10-`), 폴더 이름 정리(`sanitize_filename` 에서 Windows 금지 문자 바꾸기). 고급 인자는 `yt-dlp --ignore-config <인자> --version` 을 뒤에서 실행해 종료 코드로 본다.

## yt-dlp 갱신

- 기본: `<설치 폴더>\yt-dlp.exe`. 갱신본: `%APPDATA%\QuickMedia\bin\yt-dlp-<버전>.exe`.
- 앱을 켤 때 둘 중 버전이 높은 것을 고른다. 갱신본 버전은 파일 이름에서, 기본 버전은 `--version` 으로 얻어 exe 크기·수정 시각과 함께 레지스트리에 기억한다. 고른 것보다 낮은 갱신본은 지운다.
- 갱신: `api.github.com/repos/yt-dlp/yt-dlp/releases/latest` 에서 `yt-dlp.exe` 자산을 `.part` 로 받고, 크기와 SHA-256(자산의 `digest`)을 확인하고, `--version` 이 릴리즈 버전을 찍는지 본 뒤 이름을 바꾼다. 이후 시작하는 작업부터 새 파일을 쓴다.
- 화면: 업데이트 화면의 앱 영역 아래 "yt-dlp" 영역. 지금 버전, `갱신` 버튼, 상태(확인하는 중 / 받는 중 N% / 최신이에요 / 실패 이유).

## 앱 업데이트

1.x 와 같다. 켤 때 뒤에서 확인하고 새 버전이면 메뉴에 빨간 점. 받기 → 크기·SHA-256 확인 → `%TEMP%\QuickMedia-update` 에 도우미 PowerShell 스크립트(BOM 있는 UTF-8)를 쓰고 띄운 뒤 앱 종료 → 스크립트가 앱이 끝나길 기다려 `/VERYSILENT` 로 설치 → 설치 파일이 앱을 다시 띄움. 받는 중인 영상이 있으면 묻는다. 디버그 빌드에서는 자동 업데이트 버튼을 끈다. PyInstaller 환경 변수 처리는 뺀다.

## 화면

1.2.0 과 같은 구성·문구·색 토큰(라이트/다크, OS 설정을 따름). 창 920×640, 최소 690×480.

- 왼쪽 메뉴: 다운로드 / 설정(펼치면 로그인·네트워크·파일·고급) / 맨 아래 업데이트(빨간 점), 버전 표시. 펼침 애니메이션 200ms.
- 다운로드: 링크 칸·붙여넣기·다운로드 버튼, 프리셋(최고 화질·720p·MP3 음원)과 바뀐 옵션 요약, 세부 옵션 패널(형식·자막·후처리·재생목록), 오류 한 줄, 카드 목록(사이트 아이콘·이름, 제목, 상태, 진행률)과 빈 상태, 끝난 항목 지우기, 저장 위치 독.
- 설정 4개: 바꾸면 바로 저장, 틀린 값은 칸 아래 오류, 화면을 떠날 때 틀린 값은 저장된 값으로 되돌림.
- 업데이트: 1.2.0 의 상태들 + yt-dlp 영역 + "Made with Slint".

화면이 무엇을 보일지(카드 상태, 업데이트 화면 상태, 켜고 끄는 규칙)는 Rust 쪽에서 정하고 Slint 는 그리기만 한다.

## 오류 처리

- `yt-dlp.exe` 나 `ffmpeg.exe` 가 없으면 카드에 이유를 보인다. 앱은 켜진다.
- 레지스트리·이전 처리·아이콘 받기 실패는 조용히 넘어가고 기본값을 쓴다.
- 네트워크 실패(업데이트, yt-dlp 갱신)는 해당 화면에 문구로 보이고 다시 시도할 수 있다. 받다 만 `.part` 는 지운다.
- 작업 스레드의 패닉은 그 카드의 실패로 끝나고 대기열은 계속 돈다.

## 시험

- 단위 시험(cargo test): 인자 만들기, 옵션·설정 검사, 프리셋, 요약 문구, 저장값 되돌리기, 대기열, 재생목록 펼치기와 건너뛴 개수, 번호 붙이기, 진행률 계산, 오류 문구, 버전 비교, 자산 고르기·검증, 사이트 이름, 이전 처리 계획, yt-dlp 고르기. 지금 pytest 의 경우를 옮긴다.
- yt-dlp 출력 파싱은 `tests/fixtures` 의 실제 출력 샘플로.
- 화면 위젯 시험은 옮기지 않는다. 화면 상태를 정하는 Rust 로직을 시험한다.
- 손으로 확인: 영상 한 건, 음원 한 건, 재생목록(범위·폴더·번호·건너뛰기), 같은 이름 번호 붙이기, 한글 제목, 자막 넣기, 구간만 받기, 받는 중 앱 닫기, yt-dlp 갱신, 1.2.0 설치본에서 2.0.0 으로 업데이트, 라이트/다크.

## 빌드와 릴리즈

- `build.ps1 [-FFmpeg 경로] [-YtDlp 경로]`: `cargo build --release` → `dist\QuickMedia\` 에 `QuickMedia.exe`, `yt-dlp.exe`, `ffmpeg.exe` → Inno Setup. `-YtDlp` 가 없으면 최신 릴리즈를 받는다.
- `installer/QuickMedia.iss`: `[InstallDelete]` 의 `_internal` 지우기는 남긴다(1.x 에서 올라올 때 필요). 나머지는 같다.
- `release.yml`: Python 대신 Rust 툴체인과 캐시, `cargo test`, 태그 버전을 `Cargo.toml` 에 반영, yt-dlp 받기. 안드로이드 작업은 지운다.
- README: 개발 방법을 고치고 안드로이드 부분을 지운다. `release-notes/v2.0.0.md` 에 다시 만든 것과 안드로이드 지원 종료를 적는다.

## 작업 순서

1. yt-dlp 연동 검증(위 "먼저 검증할 것")
2. 순수 로직과 시험
3. yt-dlp 실행과 대기열
4. 화면
5. 앱 업데이트와 yt-dlp 갱신
6. 빌드·설치 파일·워크플로
7. Python 과 안드로이드 지우기, README·릴리즈 노트

Python 코드는 Rust 앱이 동작하는 것을 확인한 뒤 7에서 지운다.
