# QuickMedia

링크만 붙여넣으면 영상이나 음원을 받는 Windows·안드로이드 앱. [yt-dlp](https://github.com/yt-dlp/yt-dlp) 기반.

**[최신 버전 받기](https://github.com/glglekdy/quickmedia/releases/latest)**: `QuickMedia-Setup-x.y.z.exe`를 받아 실행하면 설치돼요. 관리자 권한이나 Python은 필요 없어요.
코드 서명을 하지 않은 프로그램이라 Windows가 "PC 보호" 경고를 띄울 수 있어요. 그럴 때는 **추가 정보 → 실행**을 누르세요.

- 화질·파일 형식·코덱 고르기, 음원(MP3·M4A·OPUS·FLAC·WAV)
- 재생목록·채널 한 번에 받기(범위, 번호, 폴더, 이미 받은 영상 건너뛰기)
- 자막 받기·넣기
- 썸네일·메타데이터·챕터 넣기, 챕터별 나누기, 구간만 받기
- 브라우저 쿠키·프록시·속도 제한
- 파일 이름 규칙, 동시 다운로드
- 고급: yt-dlp 명령줄 옵션 직접 입력
- 유튜브 외 yt-dlp 지원 사이트, 카드에 사이트 아이콘과 이름 표시
- 새 버전이 나오면 앱이 알려 주고 스스로 업데이트

> **유튜브 다운로더를 쓰던 분**: 옛 앱은 QuickMedia로 자동 업데이트되지 않아요. 위 링크에서 설치 파일을 받아 설치하세요. 저장 위치 같은 설정은 처음 실행할 때 그대로 옮겨져요. 옛 `YoutubeDownloader.exe`와 바로가기는 지워도 돼요.

## 안드로이드

휴대폰에서 **[최신 버전](https://github.com/glglekdy/quickmedia/releases/latest)** 의 `QuickMedia-x.y.z.apk`를 받아 열면 설치돼요. 64비트 안드로이드 10 이상에서 돼요.
처음 설치할 때 "출처를 알 수 없는 앱" 경고가 나오면, 파일을 연 앱(브라우저나 파일 앱)의 설정에서 **이 출처 허용**을 켜고 다시 여세요.

- 링크 붙여넣기, 또는 유튜브 앱 등에서 **공유 → QuickMedia**
- 영상 화질·음원(MP3·M4A), 재생목록 전체·일부, 썸네일·메타데이터 넣기, 구간만 받기
- 받은 파일은 `다운로드/QuickMedia`에 저장돼요. 앱을 나가도 계속 받아요.
- 이후 업데이트는 앱 안에서 해요. 새 버전이 나오면 앱이 알려 줘요.
- 사이트가 바뀌어 받아지지 않으면 설정에서 **yt-dlp 갱신**을 눌러 보세요.

## 개발

```
pip install -r requirements.txt pytest pyinstaller
python app.py        # 실행
python -m pytest     # 테스트
./build.ps1          # dist/QuickMedia-Setup-<버전>.exe (PATH 의 ffmpeg 포함)
```

설치 파일을 만들려면 Inno Setup 6이 필요해요: `winget install JRSoftware.InnoSetup`

안드로이드 앱은 `android/`에 있어요(Android Studio로 열거나 `cd android && ./gradlew testDebugUnitTest assembleDebug`).

## 릴리즈

`v*` 태그를 push 하면 GitHub Actions가 설치 파일과 안드로이드 APK를 빌드해 릴리즈로 올리고, 설치된 앱은 다음 실행 때 업데이트돼요.
APK는 저장소 Secrets(`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`)의 키로 서명해요. 키스토어를 잃어버리면 이후 버전을 덮어 설치할 수 없으니 따로 백업해 두세요.

```
git tag v1.0.0
git push origin v1.0.0
```

앱은 `glglekdy/quickmedia`에서 새 버전을 확인해요. 저장소 이름을 `quickmedia`로 바꾼 다음 태그를 push 하세요.
