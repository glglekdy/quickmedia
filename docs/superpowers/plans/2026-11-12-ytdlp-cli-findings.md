# yt-dlp CLI 검증 결과

2026-11-12, yt-dlp 2026.10.26 (공식 `yt-dlp.exe`), Windows 11. 구현 계획 Task 1.

## 계획과 다른 점

1. **`-J` 결과에 파일 이름이 없다.** 조회에 `--print "QMFN|%(filename)s"` 를 함께 준다. stdout 에 `QMFN|…` 줄이 먼저, JSON 한 줄이 뒤에 나온다. 재생목록이면 항목마다 `QMFN|… .NA` 줄이 나오므로 무시한다. `parse_probe` 는 stdout 전체를 받는다. 샘플 이름은 `probe_single.txt`, `probe_playlist.txt`.
2. **`--print` 는 진행률을 끈다.** `--progress` 를 더해야 `QMP` 줄이 나온다.
3. **자막을 넣은 뒤 자막 파일이 남는다**(`--embed-subs --write-subs`). `--compat-options no-keep-subs` 를 더하면 yt-dlp 가 지운다. 앱이 지우는 코드는 필요 없다.
4. **유튜브는 JS 런타임(deno)이 없으면 경고가 나오고 일부 형식이 빠질 수 있다.** 1.x(라이브러리 호출)도 같은 처지라 이번 범위 밖이다. `--no-warnings` 로 경고는 숨긴다.

## 확인한 것

| 질문 | 결과 |
|---|---|
| 조회 명령 | `--ignore-config <인자> --no-warnings --encoding utf-8 --print "QMFN|%(filename)s" -J --flat-playlist <링크>`. 단일 영상 약 3초. |
| 조회 JSON | 단일: `_type: "video"`, `id`, `title`, `extractor_key`, `webpage_url`. 재생목록: `_type: "playlist"`, `entries[]` 에 `ie_key`, `id`, `url`, `title`. `playlist_index` 와 `webpage_url` 은 `null` 이다. |
| 받기 명령의 줄 | `--newline --progress --no-warnings --encoding utf-8 --no-simulate`, `--progress-template "download:QMP|%(progress.downloaded_bytes)s|%(progress.total_bytes)s|%(progress.total_bytes_estimate)s|%(progress._speed_str)s"`, `--print "before_dl:QMN|%(requested_formats.:.format_id)j"`, `--print "after_move:QMF|%(filepath)s"` 가 한 실행에서 모두 stdout 에 나온다. |
| `QMN` | 두 스트림 `QMN|["243", "251"]`, 한 스트림 `QMN|[]`. 스트림 수 = max(배열 길이, 1). |
| `QMP` | `QMP|<받은>|<전체>|<추정>|<속도>`. 없는 값은 `NA`, 속도 앞에 공백이 붙는다. 스트림이 바뀌면 받은 바이트가 다시 작아진다. 스트림마다 마지막에 받은 == 전체 줄이 한두 번 나온다. |
| 한글 | `--encoding utf-8` 이면 한글 폴더·제목의 `QMF` 경로가 온전하다. 없으면 깨진다. |
| 받은 기록에 있는 영상 | 종료 코드 0, 출력 없음(`QMF` 없음). |
| 같은 이름 파일이 있을 때 | 받지 않고 있던 파일 경로를 `QMF` 로 찍는다. 종료 코드 0. |
| 오류 | stderr 에 `ERROR: …`, 종료 코드 1. 예: `ERROR: [youtube] aaaaaaaaaaa: This video is unavailable`, `ERROR: Unsupported URL: …`, `ERROR: [generic] 'x' is not a valid URL`. |
| 고급 인자 검사 | 링크 없이 `yt-dlp --ignore-config <인자>` → 맞으면 `yt-dlp.exe: error: You must provide at least one URL.`, 틀리면 `yt-dlp.exe: error: no such option: --nope`, `invalid rate limit "abc" given`, `--limit-rate option requires 1 argument`. 약 1.2초. |
| 실행 기본 시간 | `--version` 1.2초. |

`tests/fixtures/download_unknown_size.txt` 는 실제 출력이 아니라 형식에 맞춰 손으로 쓴 것이다(전체 크기가 `NA`).
