# 유튜브 다운로더

링크만 붙여넣으면 유튜브 영상(MP4)이나 음원(MP3)을 받는 Windows 앱. [yt-dlp](https://github.com/yt-dlp/yt-dlp) 기반.

**[최신 버전 받기](https://github.com/glglekdy/youtube-downloader/releases/latest)**: `YoutubeDownloader.exe`를 받아 실행하세요. 설치나 Python이 필요 없어요.

- 최고 화질 / 720p / MP3 음원
- 여러 링크를 넣으면 차례로 받기
- 새 버전이 나오면 앱이 알려 주고 스스로 업데이트

## 개발

```
pip install -r requirements.txt
python app.py        # 실행
./build.ps1          # dist/YoutubeDownloader.exe (PATH 의 ffmpeg 포함)
```

## 릴리즈

`v*` 태그를 push 하면 GitHub Actions가 exe를 빌드해 릴리즈로 올리고, 설치된 앱은 다음 실행 때 업데이트돼요.

```
git tag v1.0.1
git push origin v1.0.1
```
