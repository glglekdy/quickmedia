# QuickMedia 안드로이드 앱 설계 (v1.1.0)

작성일: 2026-10-29
범위: 안드로이드 앱 첫 버전과, 그것을 데스크톱과 같은 릴리스로 내보내는 v1.1.0.

## 목표

- 폰에서 링크를 붙여 넣거나 유튜브 앱의 **공유 → QuickMedia** 로 영상·음성을 받는다.
- APK 는 GitHub 릴리스에 데스크톱 설치 파일과 함께 올린다. 한 번 설치하면 이후 버전은 **앱 안에서** 받아 설치한다.
- yt-dlp 가 사이트 변경으로 고장 나면 앱 업데이트 없이 yt-dlp 만 앱 안에서 갱신할 수 있다.
- 버전 1.1.0 은 이 앱의 첫 릴리스와 데스크톱의 "같은 이름 파일이면 번호 붙여 새로 받기" 수정을 함께 담는다.

## 하지 않는 것

- iOS, 플레이스토어·앱스토어 등록(유튜브 다운로더는 스토어 정책에 걸린다).
- 자막, 쿠키, 프록시, 코덱·컨테이너 선택, 파일 이름 규칙, 동시 다운로드, 저장 폴더 바꾸기.
- 다운로드 목록을 앱을 껐다 켜도 남기기. 목록은 앱 프로세스가 살아 있는 동안만 있다.
- 32비트(armeabi-v7a)·x86 기기. 에뮬레이터 시험은 arm64 이미지를 쓰거나 디버그 빌드에 x86_64 를 더한다.
- 데스크톱 코드와 로직 공유. 옵션 → yt-dlp 인자 변환은 Kotlin 으로 다시 쓴다(필요한 옵션이 적다).

## 기술 선택

| 항목 | 값 |
|---|---|
| 언어 / 화면 | Kotlin, Jetpack Compose (Material 3) |
| 다운로드 엔진 | `youtubedl-android` (yt-dlp + Python + ffmpeg 를 안드로이드용으로 묶은 라이브러리). 정확한 버전은 구현 계획에서 최신 안정판으로 고정한다 |
| 패키지 이름 | `com.glglekdy.quickmedia` |
| 최소 / 대상 SDK | minSdk 29 (안드로이드 10), targetSdk 는 최신 안정 SDK |
| ABI | 릴리스 APK 는 `arm64-v8a` 만 (약 40MB) |
| 위치 | 저장소의 `android/` 폴더. 데스크톱 코드는 건드리지 않는다 |

## 화면

### 메인 화면

위에서 아래로:

1. **링크 입력칸.** 공유로 열리면 받은 글에서 첫 `http(s)://` 링크를 뽑아 채운다. 링크가 없으면 "링크를 찾지 못했어요".
2. **영상 / 음성** 토글.
3. **화질.** 영상: 최고 · 1080p · 720p · 480p. 음성: mp3 · m4a.
4. **옵션 더보기** (접힘):
   - 재생목록: 이 영상만(기본) · 전체 · 일부 항목(예 `1-5,8`).
   - 썸네일·메타데이터 넣기 (켜면 둘 다).
   - 구간 자르기: 시작 · 끝 (`1:23`, `1:02:03`, 초 단위 `83` 모두 허용, 하나만 써도 된다).
5. **다운로드** 버튼. 입력값이 틀리면 그 칸 아래에 문구를 띄우고 시작하지 않는다.
6. **작업 목록.** 카드마다 제목, 상태 문구, 진행 막대, 취소 버튼. 끝난 카드를 누르면 파일을 연다(재생목록이면 저장 폴더를 보여 준다).

고른 옵션은 기기에 저장해 다음에 켜도 그대로다.

### 설정 화면

- 앱 버전, **업데이트 확인** 버튼.
- yt-dlp 버전, **yt-dlp 갱신** 버튼.
- 저장 위치 안내: `다운로드/QuickMedia`.

## 다운로드 흐름

1. 다운로드 버튼 → 작업을 대기열에 넣고 **포그라운드 서비스**(`dataSync`)를 띄운다. 알림에 현재 작업 제목과 진행률을 보인다. 안드로이드 13 이상은 처음 받을 때 알림 권한을 묻는다(거절해도 다운로드는 된다).
2. 서비스는 대기열을 **한 번에 하나씩** 처리한다.
3. 작업마다 앱 캐시의 빈 폴더(`cache/dl/<작업id>/`)에 yt-dlp 로 받는다. 출력 템플릿은 `%(title)s.%(ext)s`.
4. 끝나면 그 폴더의 결과 파일(임시·부분 파일 제외)을 **MediaStore** 로 `Download/QuickMedia/` 에 복사하고 캐시 폴더를 지운다. 저장소 권한은 필요 없다(안드로이드 10+).
   - 같은 이름이 있으면 **데스크톱과 같이 `제목 (1).mp4`** 로 저장한다. MediaStore 가 알아서 번호를 붙이면 그대로 쓰고, 아니면 앱이 비어 있는 번호를 찾는다.
   - 음성은 음악 앱에서 보이도록 MIME 형식을 정확히 넣는다(`audio/mpeg`, `audio/mp4`).
5. 상태 문구: 대기 중 → 영상 정보 불러오는 중… → 받는 중 n% → 파일 정리 중… → 완료 / 실패(이유) / 취소됨.
6. 취소: yt-dlp 프로세스를 죽이고 캐시 폴더를 지운다.

### 옵션 → yt-dlp 인자

`YtdlpArgs.build(options)` 하나가 맡고, 데스크톱 `build_args` 와 같은 값을 쓴다.

| 옵션 | 인자 |
|---|---|
| 영상 최고 | `-S ext:mp4:m4a` `--merge-output-format mp4` |
| 영상 1080/720/480 | 위에 `-S res:<높이>,ext:mp4:m4a` |
| 음성 mp3 / m4a | `-x --audio-format mp3\|m4a --audio-quality 0` |
| 이 영상만 | `--no-playlist` |
| 전체 | `--yes-playlist` |
| 일부 항목 | `--yes-playlist -I <항목, 공백 제거>` |
| 썸네일·메타데이터 | `--embed-thumbnail --embed-metadata` |
| 구간 | `--download-sections *<시작 또는 0>-<끝 또는 inf>` `--force-keyframes-at-cuts` |
| 항상 | `--no-mtime` `-o %(title)s.%(ext)s` `-P <캐시 폴더>` |

입력 검사(`validate`): 일부 항목 형식(`1-5,8` 같은 숫자·범위 목록), 구간 시간 형식, 시작 < 끝.

### 오류 문구

yt-dlp 오류를 데스크톱 `friendly_error` 와 같은 표로 바꾼다: 지원하지 않는 링크 / 로그인이 필요한 영상 / 삭제되었거나 볼 수 없는 영상 / 고른 화질·형식이 없음 / 인터넷 연결 확인. 그 밖에는 원문 앞 160자. "로그인이 필요한" 문구는 쿠키 기능이 없으므로 "로그인이 필요한 영상이라 받을 수 없어요"로 쓴다.

## 앱 업데이트

데스크톱 `ytdl/updater.py` 와 같은 순서다.

1. **확인.** 앱을 켤 때 한 번(그리고 설정의 버튼) `https://api.github.com/repos/glglekdy/quickmedia/releases/latest` 를 읽는다. 태그 `vX.Y.Z` 의 숫자가 앱 `versionName` 보다 크고, 자산 중 `QuickMedia-<버전>.apk` 가 있으면 새 버전이다. 시작할 때의 확인은 실패해도 아무것도 띄우지 않는다.
2. **안내.** "새 버전 X.Y.Z" 대화상자에 릴리스 설명을 보이고 [지금 업데이트] [나중에].
3. **받기.** 앱 캐시에 받으며 진행률을 보인다. 끝나면 크기와 GitHub 가 주는 SHA-256(`digest`)을 확인하고, 다르면 지우고 "받은 파일이 손상되었어요".
4. **설치.** `PackageInstaller` 세션으로 설치를 요청한다. 이 앱의 "출처를 알 수 없는 앱 설치" 허용이 꺼져 있으면 먼저 그 설정 화면으로 보낸 뒤, 돌아오면 이어서 설치한다. 설치 확인 화면은 안드로이드가 띄운다.

같은 서명 키로 서명된 APK 만 덮어 설치된다. 그래서 릴리스 APK 는 늘 같은 키로 서명한다(아래).

### yt-dlp 갱신

설정의 버튼으로 `youtubedl-android` 의 yt-dlp 갱신 기능(안정 채널)을 부른다. 결과: "최신 버전이에요" / "X 로 갱신했어요" / 실패 문구. 자동 갱신은 하지 않는다.

## 버전

- 데스크톱과 안드로이드가 **같은 태그**를 쓴다. `v1.1.0` 을 push 하면 둘 다 빌드되어 같은 릴리스에 올라간다.
- 안드로이드 `versionName` = 태그에서 `v` 를 뺀 값, `versionCode` = `주*10000 + 부*100 + 수` (1.1.0 → 10100). Gradle 이 `-PappVersion=1.1.0` 으로 받고, 없으면 `0.0.0-dev` / 1.
- 데스크톱 `ytdl/__init__.py` 는 지금처럼 워크플로가 태그로 채운다.
- 데스크톱 업데이트는 `QuickMedia-Setup-*.exe` 만, 옛 유튜브 다운로더는 첫 `.exe` 만 보므로 같은 릴리스에 APK 가 있어도 영향이 없다.

## 서명 키

- 릴리스 키스토어(`quickmedia-release.jks`)를 한 번 만든다(Android Studio 에 들어 있는 `keytool`).
- 키스토어 파일은 **저장소에 넣지 않는다**(`.gitignore` 에 `*.jks`, `*.keystore`). 사용자가 안전한 곳에 따로 보관한다. **잃어버리면 이후 업데이트를 덮어 설치할 수 없어** 모든 사용자가 지우고 다시 깔아야 한다.
- GitHub 저장소 Secrets 네 개: `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`.
- 디버그 빌드는 기본 디버그 키로 서명한다(릴리스 앱 위에 덮어 설치되지 않는다).

## 릴리스 워크플로

`.github/workflows/release.yml` 에 `android` 작업을 더한다.

- `ubuntu-latest`, JDK 17, Gradle 래퍼.
- `./gradlew testDebugUnitTest` → `assembleRelease -PappVersion=<버전>`.
- Secrets 의 키스토어로 서명하고 `QuickMedia-<버전>.apk` 로 이름을 바꾼다.
- 데스크톱 `build` 작업이 릴리스를 만든 **뒤에**(`needs: build`) 같은 릴리스에 APK 를 올린다. 두 작업이 동시에 릴리스를 만들다 둘로 갈라지는 일을 막는다.

## 코드 구조 (`android/app/src/main/java/com/glglekdy/quickmedia/`)

| 파일 | 하는 일 | 의존 |
|---|---|---|
| `options/DownloadOptions.kt` | 옵션 값(data class)과 저장·불러오기 | 없음 |
| `options/YtdlpArgs.kt` | 옵션 → yt-dlp 인자, 입력 검사, 시간 해석 | 없음 (순수 Kotlin) |
| `download/FriendlyError.kt` | 오류 문구 표 | 없음 |
| `download/SharedLink.kt` | 공유받은 글에서 링크 뽑기 | 없음 |
| `download/DownloadService.kt` | 포그라운드 서비스, 대기열, yt-dlp 실행, 취소 | youtubedl-android |
| `download/MediaSaver.kt` | 캐시 파일 → MediaStore, 이름 번호 붙이기 | Android |
| `download/Jobs.kt` | 작업 목록 상태(StateFlow). 화면과 서비스가 같이 본다 | 없음 |
| `update/Releases.kt` | 버전 비교, 릴리스 JSON 에서 APK 자산 고르기 | 없음 (순수 Kotlin) |
| `update/Updater.kt` | 확인·받기·검증·설치 | Android, 네트워크 |
| `ui/MainScreen.kt`, `ui/SettingsScreen.kt`, `ui/UpdateDialog.kt` | 화면 | Compose |
| `MainActivity.kt`, `App.kt` | 공유 받기, 라이브러리 초기화 | |

순수 Kotlin 파일은 안드로이드 없이 JVM 단위 테스트로 검사한다.

## 시험

- **단위 테스트(JVM):** 옵션 → 인자 표 전체, 입력 검사, 시간 해석, 공유 글 링크 뽑기, 오류 문구, 버전 비교(`1.1.0` vs `1.1`, `v1.2.0-beta`), APK 자산 고르기, 이름 번호 붙이기 규칙.
- **에뮬레이터 수동 확인:** 링크 붙여 넣어 영상·음성 받기, 공유로 열기, 같은 영상 두 번 받기 → `(1)`, 재생목록 일부 항목, 구간 자르기, 취소, 앱을 나가도 계속 받기, yt-dlp 갱신.
- **업데이트 확인:** 릴리스 키로 서명한 `1.0.99` 를 설치해 두고 실제 v1.1.0 릴리스를 받아 덮어 설치되는지 본다. 태그를 올린 뒤에야 가능하므로 마지막에 한다.

## 문서

`README.md` 에 "안드로이드" 절을 더한다: 릴리스에서 `QuickMedia-<버전>.apk` 받기, 출처를 알 수 없는 앱 허용 안내, 64비트 안드로이드 10 이상, 이후 업데이트는 앱 안에서.
