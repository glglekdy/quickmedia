# QuickMedia: 업데이트 화면 설계

작성일: 2026-10-30
범위: 데스크톱 화면 개편의 두 번째 작업. ① 왼쪽 메뉴 + 설정 화면(끝남, `2026-10-29-quickmedia-sidebar-settings-design.md`) → **② 업데이트 화면(이 문서)** → ③ 다운로드 화면 다듬기.
시안: [`2026-10-30-quickmedia-update-page-mockup.html`](2026-10-30-quickmedia-update-page-mockup.html) (메뉴 위치는 B. 브라우저로 열면 상태를 바꿔 볼 수 있다)

## 목표

- 업데이트 창을 새 창으로 띄우지 않고 메인 창 왼쪽 메뉴의 **업데이트** 화면에서 보여 준다.
- 앱을 켤 때 새 버전을 찾으면 하던 일을 끊지 않고 메뉴 '업데이트' 옆에 **빨간 점**만 켠다.
- 새 버전이 없어도 이 화면에서 지금 버전을 보고 직접 확인할 수 있다.
- 영상을 받는 중에 업데이트하면 받던 영상이 끊기므로 한 번 묻는다.

## 하지 않는 것

- 업데이트 방식(`ytdl/updater.py`) 바꾸기. 확인·받기·적용은 지금 함수를 그대로 쓴다.
- 업데이트 받기를 미뤄 두었다가 앱을 끌 때 적용하기, 자동 업데이트 켜고 끄기 설정, 베타 채널.
- 주기적인 재확인. 확인은 앱을 켤 때 한 번과 사용자가 누를 때만.
- 다운로드 화면 다듬기(③), 버전 올리기와 릴리스.

## 화면

### 왼쪽 메뉴

```
 ⤓  다운로드
 ⚙  설정        ›
      …
                      ← 남는 공간
 ⟳  업데이트    ●     ← 맨 아래. 새 버전이 있을 때만 빨간 점
    버전 1.1.0        ← 지금 버전. 11px muted
```

- '업데이트'는 메뉴 맨 아래에 붙는다. 아이콘은 `update.svg`(Feather `rotate-cw`, `gear.svg`와 같은 `#8c8c8c` 선 그림).
- 빨간 점: 지름 7px, `accent` 색, 항목 오른쪽 끝에서 16px 안쪽, 세로 가운데. 새 버전이 있을 때만(아래 상태 표의 '점' 칸).
- 그 아래 `버전 <지금 버전>` 글자(11px, `muted`). 왼쪽은 메뉴 아이콘과 같은 자리(항목 왼쪽에서 12px), 위로 6px 띄운다.
- '업데이트'를 누르면 '다운로드'처럼 설정 묶음이 저절로 접힌다. 나머지 메뉴 규칙은 ①과 같다.

### 업데이트 화면

설정 화면과 같은 틀이다: 제목 `업데이트`(#crumb), 설명, 카드 하나(`#settingsCard`). 저장 표시는 없다.

- 설명: `지금 버전 <버전> · 앱을 켤 때 새 버전을 확인해요.`
- 카드 안 맨 위에 굵은 한 줄(15px 600, `heading`)과 그 아래 작은 한 줄(12px `muted`), 그다음 상태에 따른 내용과 단추.

| 상태 | 굵은 줄 | 작은 줄 | 그 밖 | 단추 | 점 |
|---|---|---|---|---|---|
| 아직 확인 안 함 | `아직 확인하지 않았어요` | — | — | [업데이트 확인] | 없음 |
| 확인 중 | `확인하는 중…` | `최신 버전을 알아보고 있어요` | — | [업데이트 확인] 꺼짐 | 없음 |
| 최신 | `최신 버전이에요` | `<HH:MM> 에 확인했어요` | — | [업데이트 확인] | 없음 |
| 확인 실패 | `확인하지 못했어요` | 오류 문구(`danger` 색) | — | [다시 확인] | 없음 |
| 새 버전 | `새 버전 <버전> 이 나왔어요` | — | 업데이트 내용, 안내 줄 | [지금 업데이트] (주 단추) | 켬 |
| 받는 중 | (새 버전과 같음) | — | 업데이트 내용, 진행률 막대, `받는 중… 42%` | [지금 업데이트] 꺼짐 | 켬 |
| 받기 실패 | (새 버전과 같음) | — | 업데이트 내용, 오류 문구(`danger`) | [다시 시도] (주 단추) | 켬 |
| 적용 중 | (새 버전과 같음) | — | `곧 다시 시작해요…` | 꺼짐 | 켬 |

- 업데이트 내용은 릴리스 설명(마크다운)을 `QTextBrowser`로 보여 준다. 위에 1px `border` 선, 최대 높이 220px, 넘치면 그 안에서 스크롤. 설명이 비면 숨긴다.
- 새 버전 상태의 안내 줄: `받고 나면 QuickMedia 가 잠깐 꺼졌다 다시 켜져요.`
- 진행률 막대는 크기를 모를 때는 왔다 갔다 하는 막대(범위 0,0), 알면 퍼센트. 상태 줄은 `받는 중…` / `받는 중… 42%`.
- 소스(`python app.py`)로 실행 중이면 새 버전 상태에서 [지금 업데이트]를 끄고 안내 줄 대신 `소스로 실행 중이라 자동 업데이트는 exe 에서만 돼요.`
- 오류 문구는 `updater.UpdateError`의 문구 그대로(예: `GitHub 에 연결하지 못했어요.`, `업데이트 파일을 받다가 끊겼어요. 다시 시도해 주세요.`).
- 다른 메뉴에 다녀와도 상태는 그대로 남는다(받는 중이면 계속 받는다).

## 동작

- **앱을 켤 때**: 창을 띄운 뒤 `check()`로 뒤에서 확인한다. 화면은 바뀌지 않는다. 새 버전이면 점만 켜고, 실패하면 조용히(점 없음) 업데이트 화면에만 실패를 보인다.
- **[업데이트 확인] / [다시 확인]**: 다시 확인한다. 확인 중에는 단추를 꺼서 두 번 돌지 않는다.
- **[지금 업데이트] / [다시 시도]**:
  1. 받고 있거나 기다리는 영상이 있으면(`JobQueue`의 running + pending, N개) 묻는다: 제목 `업데이트`, 내용 `받는 중인 영상 <N>개가 멈춰요. 그래도 업데이트할까요?`, [예] [아니요]. 아니요면 아무것도 하지 않는다.
  2. 설치 파일을 받는다(받는 중). 실패하면 받기 실패.
  3. 다 받으면 `updater.launch_apply()` 후 `QApplication.quit()`(적용 중). `launch_apply`가 실패하면 받기 실패와 같이 오류를 보인다.
- **앱을 닫을 때** 업데이트 받기 스레드가 돌고 있으면 영상 받기처럼 멈추고(`terminate()` + `wait(2000)`) 닫는다.

## 구조

| 파일 | 역할 |
|---|---|
| `ytdl/update_page.py` (신규, `update_dialog.py` 대신) | `CheckJob`, `SetupDownloadJob`(스레드), `UpdatePage` |
| `ytdl/sidebar.py` | 맨 아래 '업데이트' 항목과 버전 글자, `NavItem`의 빨간 점, `Sidebar.set_dot(key, on)` |
| `ytdl/main_window.py` | 화면 묶음에 `update` 화면, 점 잇기, 묻는 창, 닫을 때 멈추기 |
| `app.py` | `check_on_startup(win)` 대신 `win.update_page.check()` |
| `ytdl/theme.py` | `#navVersion`, `#updateTitle`(굵은 줄), `#updateNotes`(업데이트 내용 테두리) 스타일 |
| `ytdl/assets/update.svg` (신규) | 메뉴 아이콘 |
| `ytdl/update_dialog.py` | 삭제 |

### `CheckJob(QThread)`

- 신호 `found(object)`(Release), `latest()`, `failed(str)`. `updater.fetch_latest()`가 Release 면 found, None 이면 latest, `UpdateError`면 failed.

### `SetupDownloadJob(QThread)`

- 지금 `update_dialog.DownloadJob`과 같다(`progress(int, int)`, `done(str)`, `failed(str)`). 영상 받기의 `downloader.DownloadJob`과 이름이 겹치지 않게 바꾼다.

### `UpdatePage(QWidget)`

- `UpdatePage(confirm: Callable[[], bool] | None = None)`. `confirm`은 [지금 업데이트]를 눌렀을 때 진행해도 되는지 묻는 함수(메인 창이 넘긴다). 없으면 묻지 않는다.
- 신호 `available_changed(bool)`: 점이 켜져야 하는지가 바뀔 때만.
- `state: str` — `idle` / `checking` / `latest` / `check_failed` / `available` / `downloading` / `download_failed` / `applying`.
- `check()`, `stop()`(닫을 때 받기 스레드 멈춤).
- 신호 연결은 ①의 교훈대로 `self`를 붙잡는 lambda 없이 bound method 로 한다.

### 메인 창

- `self.update_page = UpdatePage(confirm=self._confirm_update)`, 화면 키 `update`.
- `update_page.available_changed` → `sidebar.set_dot("update", on)`.
- `_confirm_update()`: running + pending 이 0이면 True, 아니면 `QMessageBox.question`.
- `closeEvent`에서 `update_page.stop()`.

## 테스트

모두 인터넷 없이, `updater.fetch_latest`·`updater.download`·`updater.launch_apply`·`updater.can_self_update`를 가짜로 바꾼다. 스레드는 `job.wait()` 뒤 `qapp.processEvents()`로 신호를 받는다.

- `tests/test_update_page.py` (신규)
  - 처음은 `idle`, 점 신호 없음.
  - 새 버전 → `available`, 굵은 줄 `새 버전 1.2.0 이 나왔어요`, 업데이트 내용 보임, `available_changed(True)`.
  - 최신 → `latest`, 점 없음. 확인 실패 → `check_failed`, 오류 문구, [다시 확인].
  - 확인 중에는 [업데이트 확인]이 꺼져 있다.
  - 소스로 실행 중 → [지금 업데이트] 꺼짐, 안내 문구.
  - `confirm`이 False 면 받지 않는다(`download` 안 불림).
  - 받기 실패 → `download_failed`, 오류 문구, [다시 시도], 점은 그대로.
  - 받기 성공 → `launch_apply` 불리고 `applying`(`QApplication.quit`은 가짜로).
  - 화면이 GC 된다.
- `tests/test_sidebar.py`
  - '업데이트'가 맨 아래(다른 항목보다 아래)에 있고, 누르면 `update` 신호 + 설정 묶음 접힘.
  - `set_dot("update", True/False)`로 점이 켜지고 꺼진다. 버전 글자가 `버전 <__version__>`.
- `tests/test_main_window.py`
  - 메뉴 '업데이트' → 업데이트 화면.
  - `available_changed(True)` → 메뉴 점이 켜진다.
  - 받는 중인 영상이 없으면 `_confirm_update()`는 묻지 않고 True. 있으면 `QMessageBox.question`을 부르고(가짜) 그 답을 돌려준다.
- 전체 `python -m pytest`, 종료 코드 0(①에서 본 종료 크래시가 없는지).
- 눈으로 확인: 상태별로 라이트·다크를 찍어 시안과 비교.
