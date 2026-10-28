# QuickMedia

링크만 붙여넣으면 영상이나 음원을 받는 Windows 앱. [yt-dlp](https://github.com/yt-dlp/yt-dlp) 기반.

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

> **유튜브 다운로더를 쓰던 분**: 앱에서 업데이트하면 QuickMedia 설치 화면이 떠요. 설치를 마치면 옛 파일(`YoutubeDownloader.exe`)은 자동으로 지워지고 저장 위치 같은 설정은 그대로예요. 옛 바로가기는 지우고 새로 생긴 QuickMedia 바로가기를 쓰세요.

## 개발

```
pip install -r requirements.txt pytest pyinstaller
python app.py        # 실행
python -m pytest     # 테스트
./build.ps1          # dist/QuickMedia-Setup-<버전>.exe (PATH 의 ffmpeg 포함)
```

설치 파일을 만들려면 Inno Setup 6이 필요해요: `winget install JRSoftware.InnoSetup`

## 릴리즈

`v*` 태그를 push 하면 GitHub Actions가 설치 파일을 빌드해 릴리즈로 올리고, 설치된 앱은 다음 실행 때 업데이트돼요.

```
git tag v1.1.0
git push origin v1.1.0
```

저장소 이름을 `quickmedia`로 바꾼 다음 첫 QuickMedia 태그를 push 하세요. 그래야 옛 앱이 옛 주소로 확인해도 GitHub가 새 저장소로 넘겨줘요.
