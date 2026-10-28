# QuickMedia: yt-dlp 기능 확장 설계

작성일: 2026-10-28
범위: 하위 작업 ② (yt-dlp 기능을 한 앱에서 쉽게 쓰기 + 사이트 표시). 하위 작업 ①(이름 변경·설치 파일)은 끝나서 `main`에 병합됐다.

## 목표

- 화질·형식, 자막, 후처리, 재생목록·채널, 로그인·네트워크, 파일 이름 규칙, 동시 다운로드를 앱 안에서 고를 수 있게 한다.
- 화면에 버튼이 없는 yt-dlp 기능도 "고급: 명령줄 옵션"으로 쓸 수 있게 한다.
- 유튜브 외 사이트도 자연스럽게 받는다. 다운로드 카드마다 **어느 사이트에서 받는지**(사이트 아이콘 + 이름)를 작게 보여 준다.
- 처음 쓰는 사람에게는 지금 화면과 똑같이 단순해 보여야 한다. 세부 옵션은 펼쳐야 보인다.

## 하지 않는 것

- 미디어 변환(PC에 있는 파일을 다른 형식으로 바꾸기), SponsorBlock.
- 조각 동시 받기 같은 세부 튜닝. 필요하면 고급 칸에 직접 쓴다.
- 받는 중인 항목 취소·일시 정지(지금도 없다).
- 재생목록 카드 묶음(접기/펼치기). 재생목록은 요약 카드 한 장 + 영상별 카드로 보여 준다.

## 구조

| 파일 | 역할 |
|---|---|
| `ytdl/options.py` (신규, Qt 없음) | `DownloadOptions`(패널 값), `AppSettings`(설정 창 값), 프리셋, dict 변환, `build_args()`·`to_ydl_opts()`, 입력값 검사, 카드에 쓸 요약 문구 |
| `ytdl/site.py` (신규) | 도메인 꺼내기, 사이트 표시 이름, 사이트 아이콘 받기·캐시·대체 아이콘(`SiteIcons`) |
| `ytdl/job_queue.py` (신규, Qt 없음) | `JobQueue`: 동시에 N개까지만 시작시키는 대기열 |
| `ytdl/options_panel.py` (신규) | 펼치는 "세부 옵션" 패널 (형식 / 자막 / 후처리 / 재생목록 탭) |
| `ytdl/settings_dialog.py` (신규) | ⚙ 설정 창 |
| `ytdl/downloader.py` | `DownloadJob`이 완성된 yt-dlp 옵션 dict 를 받는다. 재생목록이면 항목을 펼쳐 알린다. 사이트 정보를 알린다. 오류 문구 일반화 |
| `ytdl/main_window.py` | ⚙ 버튼, 패널, `JobQueue` 연결, 재생목록 펼치기, 카드의 사이트 줄, 설정 저장·불러오기 |

화면 코드는 `DownloadOptions`/`AppSettings`를 채우기만 하고, 다운로드 코드는 완성된 옵션 dict 를 받아 쓰기만 한다. 옵션 변환은 화면 없이 단독으로 테스트한다.

## 옵션 → yt-dlp 옵션 변환

`build_args(opts, settings, *, folder, archive, playlist=None) -> list[str]`가 고른 값을 **yt-dlp 명령줄 인자**로 만든다. `to_ydl_opts(...)`는 다음 순서로 dict 를 만든다.

1. `yt_dlp.parse_options(["--ignore-config", *build_args(...), *split_extra(settings.extra_args)])`로 해석한다. 고급 옵션을 맨 뒤에 붙이므로, 같은 옵션이면 사용자가 쓴 값이 이긴다.
2. 앱이 꼭 쥐고 있어야 하는 값을 덮어쓴다: `progress_hooks`, `quiet=True`, `no_warnings=True`, `noprogress=True`, `extract_flat="in_playlist"`, `ffmpeg_location`(묶인 ffmpeg 가 있을 때).

명령줄로 만드는 이유: 후처리기 순서(자막 넣기 → 메타데이터 → 썸네일), 썸네일 변환, 오디오 추출 옵션 같은 세부 사항이 yt-dlp CLI 와 똑같이 맞춰진다. 고급 옵션도 별도 병합 로직 없이 뒤에 붙이기만 하면 된다. `--ignore-config`로 사용자 PC 의 yt-dlp 설정 파일은 읽지 않는다.

### 패널 값 (`DownloadOptions`)

| 필드 | 값 (기본값 굵게) | 인자 |
|---|---|---|
| `kind` | **video** / audio | audio → `-x` |
| `quality` | **best** / 2160 / 1440 / 1080 / 720 / 480 / 360 | 숫자면 `-S` 맨 앞에 `res:<h>` |
| `container` | **mp4** / mkv / webm | `--merge-output-format <c>`. mp4 → `-S`에 `ext:mp4:m4a`, webm → `ext:webm:webm` |
| `codec` | **auto** / h264 / vp9 / av1 | `-S`에 `vcodec:h264` / `vcodec:vp9` / `vcodec:av01` |
| `audio_format` | **mp3** / m4a / opus / flac / wav | `--audio-format <f>` |
| `audio_quality` | best / 320 / **192** / 128 | best → `--audio-quality 0`, 숫자 → `<n>K`. flac·wav 는 화면에서 비활성 |
| `subs` | **꺼짐** | 켜면 `--write-subs --sub-langs <langs>` |
| `sub_langs` | **ko** / en / all / 직접 입력(`ko,en`) | all → `all,-live_chat` |
| `auto_subs` | **꺼짐** | `--write-auto-subs` |
| `sub_mode` | **embed** / file | embed → `--embed-subs` (음원일 때는 파일로 저장), file → `--convert-subs <sub_format>` |
| `sub_format` | **srt** / vtt / ass | file 일 때만 |
| `embed_thumbnail` | **꺼짐** | `--embed-thumbnail` |
| `save_thumbnail` | **꺼짐** | `--write-thumbnail --convert-thumbnails jpg` |
| `embed_metadata` | **꺼짐** | `--embed-metadata` |
| `embed_chapters` | **꺼짐** | `--embed-chapters` |
| `split_chapters` | **꺼짐** | `--split-chapters` |
| `section_start`, `section_end` | **빈칸** | 둘 중 하나라도 있으면 `--download-sections "*<시작>-<끝>" --force-keyframes-at-cuts`. 빈 시작은 0, 빈 끝은 `inf` |
| `playlist` | **single** / all | single → `--no-playlist`, all → `--yes-playlist` |
| `playlist_items` | **빈칸** | `-I <items>` |
| `number_prefix` | **꺼짐** | 재생목록 항목 파일 이름 앞에 번호 (아래 "재생목록") |
| `playlist_folder` | **꺼짐** | 재생목록 이름으로 하위 폴더 (아래 "재생목록") |
| `skip_archived` | **꺼짐** | `--download-archive <앱 데이터>/archive.txt` |

`playlist=single`은 "영상 + 재생목록" 링크(`watch?v=…&list=…`)에서 영상만 받는다는 뜻이다. 재생목록 전용 링크(`playlist?list=…`, 채널)는 single 이어도 펼친다(yt-dlp 동작 그대로).

영상일 때 형식은 `-f "bv*+ba/b"`와 `-S <정렬>`로 고른다. 정렬 순서는 해상도 → 코덱 → 확장자. 고른 것이 하나도 없으면 `-S`를 넣지 않는다.

**프리셋** (메인 화면 버튼은 그대로 둔다)

| 버튼 | 채우는 값 |
|---|---|
| 최고 화질 | kind=video, quality=best, container=mp4, codec=auto |
| 720p | kind=video, quality=720, container=mp4, codec=auto |
| MP3 음원 | kind=audio, audio_format=mp3, audio_quality=192 |

프리셋은 형식 탭 값만 바꾼다. 형식 탭 값이 어느 프리셋과도 같지 않으면 세 버튼 모두 선택이 풀린다(= 사용자 지정).

### 설정 창 값 (`AppSettings`)

| 필드 | 값 (기본값 굵게) | 인자 |
|---|---|---|
| `cookies` | **none** / browser / file | browser → `--cookies-from-browser <browser>`, file → `--cookies <path>` |
| `cookie_browser` | **firefox** / edge / chrome / whale / brave / opera / vivaldi / chromium | |
| `cookie_file` | 빈칸 | |
| `proxy` | 빈칸 | `--proxy <url>` |
| `rate_limit` | 빈칸 | `-r <값>` (예: `5M`) |
| `retries` | **10** (0~50) | `-R <n>` |
| `name_template` | **title** / title_id / uploader_title / date_title / custom | `-o <템플릿>` |
| `custom_template` | `%(title)s.%(ext)s` | custom 일 때 |
| `overwrite` | **꺼짐** | `--force-overwrites` |
| `concurrency` | **2** (1~5) | 인자 아님. `JobQueue` 한도 |
| `extra_args` | 빈칸 | 맨 뒤에 붙인다 |

파일 이름 템플릿:

| 값 | 템플릿 |
|---|---|
| title | `%(title)s.%(ext)s` |
| title_id | `%(title)s [%(id)s].%(ext)s` |
| uploader_title | `%(uploader)s - %(title)s.%(ext)s` |
| date_title | `%(upload_date>%Y-%m-%d)s %(title)s.%(ext)s` |

저장 위치는 `-P <폴더>`로 넘긴다(템플릿과 분리).

### 고급 옵션 나누기

`split_extra(text)`는 Windows 경로를 깨지 않게 나눈다. 공백으로 나누고, 큰따옴표·작은따옴표로 묶은 부분은 한 덩어리로 보고, 바깥 따옴표만 벗긴다. 백슬래시는 그대로 둔다.

고급 옵션은 막지 않는다. 예를 들어 `-o`를 쓰면 이름 규칙과 재생목록 번호·폴더 접두어도 사용자가 쓴 값으로 바뀐다. 앱 고정값(위 2번)만은 항상 앱 값이 이긴다.

### 입력값 검사 (`validate_*` → 오류 문구 또는 None)

| 값 | 검사 | 오류 문구 |
|---|---|---|
| 고급 옵션 | `parse_options` 가 `OptParseError`/`ValueError`를 내면 실패. 해석 결과에 링크가 남으면 실패 | "고급 옵션을 읽지 못했어요: <yt-dlp 메시지 마지막 줄>" / "고급 옵션에는 링크를 넣을 수 없어요" |
| 속도 제한 | `yt_dlp.utils.parse_bytes` | "속도 제한은 5M, 500K 처럼 써 주세요" |
| 구간 시작·끝 | `yt_dlp.utils.parse_duration`, 시작 < 끝 | "시간은 1:30 처럼 써 주세요" / "끝이 시작보다 뒤여야 해요" |
| 받을 범위 | `PlaylistEntries.parse_playlist_items` | "범위는 1-10, 15 처럼 써 주세요" |
| 쿠키 파일 | 파일이 있어야 함 | "쿠키 파일을 찾을 수 없어요" |
| 직접 입력 템플릿 | 비어 있지 않음 | "파일 이름 규칙을 입력해 주세요" |

설정 창은 오류가 있으면 저장하지 않고 해당 칸 아래에 문구를 띄운다. 패널 값(구간·범위)이 잘못됐으면 다운로드 버튼을 눌렀을 때 문구를 띄우고 대기열에 넣지 않는다.

### 저장

- `DownloadOptions.to_dict()` / `from_dict(d)`, `AppSettings`도 같다. Qt 를 쓰지 않는다.
- `from_dict`는 필드 형식대로 되돌린다: 레지스트리에서 `"true"`/`"false"` 문자열로 돌아온 bool, 문자열 숫자 → int. 허용 목록에 없는 값이나 알 수 없는 키는 기본값으로 둔다.
- 화면 쪽이 QSettings 에 `options/<필드>`, `settings/<필드>`로 저장한다. 패널 펼침 상태는 `ui/panel_open`.
- **옛 값 이전:** `options/kind`가 없고 옛 `format` 키(best·720p·mp3)가 있으면 해당 프리셋을 적용한다.

### 카드에 쓸 요약 문구 (`describe(opts)`)

예: `최고 화질 · MP4`, `1080p · MKV`, `MP3 · 192k`, `FLAC`, 자막을 켜면 끝에 `· 자막`. 지금 카드의 `최고 화질 · 받는 중` 자리에 들어간다.

## 화면

- **헤더:** 오른쪽 위에 ⚙ 버튼(설정 창).
- **형식 줄:** `[최고 화질 | 720p | MP3 음원]  [세부 옵션 ▾]  ……  [다운로드]`. 세부 옵션 버튼은 기본값과 다른 탭이 있으면 `세부 옵션 · 자막, 썸네일`처럼 바뀐 항목을 요약한다.
- **패널:** 형식 줄 아래에 펼쳐지는 탭 위젯. 처음에는 닫혀 있다. 값을 바꾸면 바로 저장한다.
  - 형식 탭: 종류에 따라 영상 칸 / 음원 칸 중 하나만 보인다.
  - 자막 탭: 자막 받기가 꺼져 있으면 나머지 칸 비활성.
  - 재생목록 탭: "이 영상만"이면 범위·번호·폴더 칸 비활성.
- **설정 창:** 구역 4개(로그인·네트워크 / 파일 / 다운로드 / 고급)를 한 화면에 세로로. 저장·취소 버튼.
  - 쿠키 브라우저 목록은 Firefox 가 맨 위. "Chrome·Edge 는 켜져 있으면 쿠키를 못 읽을 수 있어요" 안내 한 줄.
- **링크 입력칸 안내 문구:** `영상·재생목록 링크를 붙여넣으세요 (유튜브 외 사이트도 돼요)`. 빈칸에서 누르면 `먼저 링크를 넣어 주세요`.
- **빈 목록 문구:** `아직 받은 영상이 없어요` 그대로.

### 다운로드 카드

```
 [아이콘] YouTube                        ← 사이트 줄 (16px 아이콘 + 작은 회색 글씨)
 영상 제목 ……………………              42%   [열기]
 ████████░░░░░░░░
 1080p · MP4 · 받는 중 · 3.2MiB/s
```

- 카드가 생기면 사이트 줄에 링크의 도메인(`youtube.com`)과 대체 아이콘을 먼저 보여 준다.
- 영상 정보가 오면 표시 이름과 사이트 아이콘으로 바꾼다.
- 재생목록 요약 카드: 사이트 줄 + 제목 `재생목록 · <이름>` + 상태 `영상 12개 추가` (이미 받은 게 있으면 `· 3개는 이미 받아서 건너뜀`). 진행 막대는 없다.

## 사이트 표시 (`ytdl/site.py`)

**순수 함수 (테스트 대상)**

- `domain_of(url) -> str`: 호스트에서 `www.`, `m.` 을 뗀다. 해석 못 하면 빈 문자열.
- `display_name(extractor_key, host) -> str`: `extractor_key`의 `:` 앞부분을 소문자로 바꿔 아래 표의 접두어와 맞춰 본다. 없으면 `extractor_key` 그대로. `Generic`이거나 비어 있으면 `host`.

| 접두어 | 이름 |
|---|---|
| youtube | YouTube |
| twitter | X |
| instagram | Instagram |
| tiktok | TikTok |
| facebook | Facebook |
| twitch | Twitch |
| vimeo | Vimeo |
| soundcloud | SoundCloud |
| dailymotion | Dailymotion |
| reddit | Reddit |
| bilibili | bilibili |
| niconico | 니코니코 |
| naver | 네이버 TV |
| chzzk | 치지직 |
| afreecatv, soop | SOOP |
| kakao | 카카오TV |

- `icon_url(host) -> str`: `https://<host>/favicon.ico`.
- `letter_of(name) -> str`: 대체 아이콘에 쓸 첫 글자(대문자).

**`SiteIcons` (Qt)**

- `icon(host) -> QPixmap`: 메모리 캐시 → 디스크 캐시(`<앱 데이터>/favicons/<host>.png`, 30일 지나면 다시 받음) 순으로 찾는다. 없으면 대체 아이콘(빨강 강조색 동그라미 + 흰 첫 글자)을 돌려주고 백그라운드로 받기 시작한다.
- 받기가 끝나면 `ready(host, QPixmap)` 신호. 카드는 자기 host 와 같으면 아이콘을 바꾼다.
- 받기: `QThreadPool`의 작업에서 `urllib`로 GET. 시간 제한 5초, 256KB 넘으면 버림. `QImage.loadFromData`가 실패하면 버림(ico·png·webp 모두 읽힘). 16px 로 줄여 png 로 저장.
- 실패한 host 는 이번 실행 동안 다시 시도하지 않는다. 오류는 화면에 띄우지 않는다.
- 아이콘은 그 사이트에서 직접 받는다. 제3자 favicon 서비스에 주소를 보내지 않는다.

## 다운로드 흐름

1. **다운로드 누름:** 패널 값 검사 → 실패 시 문구. 성공 시 그 순간의 `DownloadOptions`·`AppSettings` 사본으로 `to_ydl_opts()`를 만들어 카드와 `DownloadJob`을 만들고 `JobQueue`에 넣는다. 이후 패널을 바꿔도 이미 넣은 항목은 바뀌지 않는다.
2. **`DownloadJob.run()`:** `extract_info(url, download=False)` (`extract_flat="in_playlist"`).
   - 결과가 영상이면 `site(extractor_key, webpage_url)`, `titled(title)`을 알리고, 지금처럼 `process_ie_result(info, download=True)`로 받는다. 정보는 한 번만 불러온다.
   - 결과의 `_type`이 `playlist`이면 `expanded(PlaylistResult)`를 알리고 끝난다. `PlaylistResult` = 재생목록 제목, 사이트, 항목 목록(`url`, `title`, `index`), 건너뛴 수. 항목은 `entries`에서 `url`이 있는 것만 쓰고, `index`는 항목의 `playlist_index`가 있으면 그것, 없으면 1부터 센 순번. `skip_archived`면 `ydl.in_download_archive(entry)`인 항목은 빼고 건너뛴 수에 센다.
   - 받기가 끝났는데 파일 경로가 없으면(이미 받은 기록 등) `skipped`로 알리고 카드에 `이미 받은 영상이라 건너뛰었어요`.
3. **펼치기 (`main_window`):**
   - 항목이 200개를 넘으면 `영상 <N>개를 모두 받을까요?` 확인 창. 아니요면 요약 카드에 `취소했어요`.
   - 원래 카드를 요약 카드로 바꾸고, 그 바로 아래에 항목 순서대로 카드를 넣는다.
   - 항목마다 원래 옵션 사본에 재생목록 정보(`playlist=PlaylistContext(title, index, width)`)를 붙여 다시 `to_ydl_opts()`를 만든다. 항목 작업은 항상 `--no-playlist`.
   - 항목도 재생목록이면(채널의 탭 등) 같은 방식으로 다시 펼친다.
4. **재생목록 파일 이름:** 항목 작업은 영상 하나만 받으므로 yt-dlp 의 재생목록 필드를 쓸 수 없다. 그래서 `build_args`가 템플릿 앞에 글자를 직접 붙인다.
   - `playlist_folder`: `<sanitize_filename(재생목록 제목)>/` (yt-dlp `sanitize_filename`, `%`는 `%%`로).
   - `number_prefix`: `<index 를 width 자리로 0 채움> - ` (width = 항목 수의 자릿수, 최소 2).
5. **`JobQueue`:** `JobQueue(limit, start)`. `add(job)`, `done(job)`, `set_limit(n)`, `running`, `pending`. 빈 자리가 생기면 들어온 순서대로 `start(job)`. 한도를 줄여도 이미 돌던 것은 멈추지 않는다. `main_window`는 작업의 `finished`에 `done`을 잇는다. 설정 창에서 동시 다운로드 수를 바꾸면 `set_limit`.
6. **앱 종료:** 돌고 있는 작업을 모두 `terminate()` + `wait(2000)`.

## 오류 문구 (`friendly_error`)

| yt-dlp 메시지에 있으면 | 문구 |
|---|---|
| `Unsupported URL`, `is not a valid URL` | 지원하지 않는 링크예요 |
| `could not find * cookies database`, `Failed to decrypt`, `Could not copy * cookie database` | 브라우저 쿠키를 읽지 못했어요. 브라우저를 닫고 다시 시도하거나 cookies.txt 를 써 주세요 |
| `Sign in`, `login required`, `Private video`, `members-only` | 로그인이 필요한 영상이에요. ⚙ 설정에서 쿠키를 켜 보세요 |
| `Video unavailable`, `This video is not available` | 삭제되었거나 볼 수 없는 영상이에요 |
| `Requested format is not available` | 고른 화질·형식이 없는 영상이에요. 세부 옵션을 바꿔 보세요 |
| `ffmpeg`, `ffprobe` 가 없다는 메시지 | 변환 도구(ffmpeg)를 찾지 못했어요 |
| `getaddrinfo`, `timed out`, `Unable to connect`, `ProxyError` | 인터넷 연결이나 프록시 설정을 확인해 주세요 |
| 그 밖 | `ERROR: ` 를 뗀 원문 앞 160자 |

재생목록의 한 항목이 실패해도 그 카드만 실패로 표시하고 나머지는 계속 받는다.

## 테스트

**자동 테스트 (Qt 화면·네트워크 없이)**

- `options`: 기본값의 인자 / 영상 화질·코덱·컨테이너 조합의 `-S`·`--merge-output-format` / 음원 형식·음질 / 자막 embed·file·음원일 때 / 썸네일·메타데이터·챕터·나누기 / 구간 / 재생목록 single·all·범위·archive / 쿠키 browser·file / 프록시·속도·재시도 / 이름 규칙 5가지 / 덮어쓰기 / 재생목록 접두어(폴더·번호·`%` 이스케이프) / `to_ydl_opts` 결과에 앱 고정값이 들어가는지 / 고급 옵션이 같은 옵션을 이기는지 / `split_extra` 따옴표·백슬래시 / 검사 함수들의 성공·실패 / `from_dict` 형식 되돌리기와 잘못된 값 / 옛 `format` 이전 / 프리셋 적용과 일치 판정 / `describe`.
- `job_queue`: 한도만큼만 시작, 끝나면 다음 시작, 순서 유지, 한도 늘리기·줄이기.
- `site`: `domain_of`, `display_name`(표·`:` 붙은 키·Generic·모르는 키), `icon_url`, `letter_of`.
- `downloader`: `friendly_error` 각 줄, 재생목록 결과에서 항목 뽑기(url 없는 항목 무시, index 대체).
- 화면: 오프스크린 모드에서 `MainWindow`, 패널, 설정 창을 만들고 패널 값 ↔ `DownloadOptions`, 설정 창 저장 시 검사가 막는지.

**직접 받아 보기 (손으로)**

유튜브 영상 1개(최고 화질), 짧은 재생목록(번호·폴더 켜고 3개 범위), 유튜브 외 사이트 1곳, 자막 넣기, MP3 + 앨범아트, 챕터 넣기, 구간 받기, 동시 2개, 사이트 아이콘 표시, 고급 옵션 `--limit-rate 1M`. 브라우저 쿠키는 이 PC 에 있는 브라우저로만 확인한다.

## 문서

README 의 기능 목록을 새 기능에 맞게 고친다.
