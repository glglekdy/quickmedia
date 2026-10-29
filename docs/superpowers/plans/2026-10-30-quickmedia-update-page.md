# 업데이트 화면 구현 계획

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 업데이트 창을 없애고 왼쪽 메뉴 맨 아래 '업데이트' 화면으로 옮긴다. 새 버전은 빨간 점으로만 알린다.

**Architecture:** `UpdatePage`가 확인·받기 스레드와 상태(`idle`…`applying`)를 들고, 점이 바뀔 때 `available_changed(bool)`만 내보낸다. `Sidebar`는 맨 아래 항목과 `set_dot`을 더하고, `MainWindow`가 둘을 잇고 받는 중인 영상이 있으면 묻는다.

**Tech Stack:** Python 3.14, PySide6 6.12, pytest (offscreen `qapp`)

**Spec:** `docs/superpowers/specs/2026-10-30-quickmedia-update-page-design.md` (시안: 같은 폴더 `...-mockup.html`)

## Global Constraints

- 화면 문구는 스펙 상태 표·동작 절의 글자 그대로.
- 색은 `theme.py` 토큰만. 점 7px `accent`, 버전 글자 11px `muted`.
- `ytdl/updater.py`는 바꾸지 않는다.
- 신호는 `self`를 붙잡는 lambda 로 잇지 않는다(bound method). 새 위젯마다 GC 테스트.
- 테스트는 인터넷 없이. `python -m pytest -q` 통과 + 종료 코드 0 이어야 커밋.
- 커밋 메시지 한국어 한 줄 + `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` / `Claude-Session: https://claude.ai/code/session_01SwAsdehfynj5X9tzWpurp8`.

## Review Focus

- 확인 중에 [업데이트 확인]을 또 누름 → 두 번째 확인이 시작되지 않음(Task 2 `test_check_button_disabled_while_checking`).
- 받기 실패 뒤 [다시 시도] → 다시 받기 시작, 점은 계속 켜짐(Task 2 `test_download_failure_keeps_dot_and_retries`).
- `launch_apply`가 `UpdateError` → 받기 실패와 같이 오류 표시, 앱은 안 꺼짐(Task 2 `test_apply_failure_shows_error`).
- 묻는 창에서 [아니요] → `download` 가 불리지 않음(Task 2 `test_confirm_false_does_not_download`).
- 받는 중에 창을 닫음 → 받기 스레드가 멈추고 닫힘(Task 3 `test_close_stops_update_download`).

---

### Task 1: 메뉴 맨 아래 '업데이트'

**Files:** Modify `ytdl/sidebar.py`, `ytdl/theme.py`; Create `ytdl/assets/update.svg`; Test `tests/test_sidebar.py`

**Interfaces:** Produces `Sidebar.nav["update"]`, `Sidebar.set_dot(key: str, on: bool) -> None`, `NavItem.dot: bool`, `Sidebar.version_label: QLabel`(objectName `navVersion`).

- [ ] **Step 1: 실패하는 테스트**: `test_update_item_at_bottom`(`nav["update"].geometry().top() > nav["advanced"].geometry().top()` — 펼친 뒤 `show()`·`resize(168, 640)`), `test_update_collapses_settings`(설정 펼침 → update 클릭 → 신호 `"update"`, 접힘), `test_set_dot`(`set_dot("update", True)` → `nav["update"].dot`, False → 꺼짐), `test_version_label`(`version_label.text() == f"버전 {__version__}"`).
- [ ] **Step 2:** 실행 → 실패.
- [ ] **Step 3:** 구현. 레이아웃은 기존 항목들 → `addStretch(1)` → update 항목 → 6px → `version_label`(왼쪽 여백 12px). `NavItem.dot`이 참이면 `paintEvent`에서 오른쪽 끝 16px 안쪽에 지름 7px `accent` 원. `update.svg`는 Feather `rotate-cw`(`<polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>`), `gear.svg`와 같은 속성. QSS `#navVersion { color: $muted; font-size: 11px; }`.
- [ ] **Step 4:** 전체 통과 → 커밋 `메뉴 맨 아래 업데이트 항목과 빨간 점`.

### Task 2: `UpdatePage`

**Files:** Create `ytdl/update_page.py`, `tests/test_update_page.py`; Modify `ytdl/theme.py`

**Interfaces:** Produces `CheckJob(QThread)`(신호 `found(object)`, `latest()`, `failed(str)`), `SetupDownloadJob(QThread)`(`progress(int,int)`, `done(str)`, `failed(str)`), `UpdatePage(confirm: Callable[[], bool] | None = None)` — 속성 `state: str`, `title: QLabel`, `sub: QLabel`, `notes: QTextBrowser`, `status: QLabel`, `bar: QProgressBar`, `action: QPushButton`, `check_job`, `download_job`; 신호 `available_changed(bool)`; 메서드 `check()`, `stop()`.

- [ ] **Step 1: 실패하는 테스트** (픽스처가 `updater.fetch_latest`/`download`/`launch_apply`/`can_self_update`와 `QApplication.quit`을 monkeypatch; 헬퍼 `run(page)`는 돌고 있는 job 을 `wait()` 후 `qapp.processEvents()`): `test_starts_idle`(`state == "idle"`, `title.text() == "아직 확인하지 않았어요"`), `test_found_new_version`(Release("1.2.0", "## 새 기능\n- 하나", …) → `available`, 제목 `새 버전 1.2.0 이 나왔어요`, notes 보임, 점 신호 `[True]`, `action.text() == "지금 업데이트"`), `test_latest`(`latest`, 제목 `최신 버전이에요`, `sub` 가 `에 확인했어요`로 끝남, 점 신호 없음), `test_check_failed`(UpdateError("GitHub 에 연결하지 못했어요.") → `check_failed`, `sub.text()` 그 문구, `action.text() == "다시 확인"`), `test_check_button_disabled_while_checking`(fetch 가 Event 를 기다리게 해서 `check()` 직후 `not action.isEnabled()`, 두 번째 `check()`는 job 을 새로 만들지 않음), `test_source_run_cannot_update`(`can_self_update` False → `not action.isEnabled()`, `status.text() == "소스로 실행 중이라 자동 업데이트는 exe 에서만 돼요."`), `test_confirm_false_does_not_download`, `test_download_failure_keeps_dot_and_retries`(→ `download_failed`, 오류 문구, `action.text() == "다시 시도"`, 점 신호 `[True]`만; 다시 누르면 `download` 두 번째 호출), `test_download_success_applies`(→ `launch_apply` 호출, `quit` 호출, `applying`), `test_apply_failure_shows_error`, `test_page_is_garbage_collected`.
- [ ] **Step 2:** 실행 → 실패.
- [ ] **Step 3:** 구현. 틀은 설정 화면과 같다(스크롤 + 제목 #crumb `업데이트` + 설명 #viewHint `지금 버전 <__version__> · 앱을 켤 때 새 버전을 확인해요.` + `QFrame#settingsCard`). 카드: `title`(#updateTitle 15px 600 heading), `sub`(#hint, 실패 땐 #fieldError 색으로 objectName 바꾸고 repolish), `notes`(QTextBrowser#updateNotes, setMarkdown, 최대 220px, 비면 숨김), `bar`, 단추 줄 `[action] status`. 상태 하나당 `_set_state(state)`가 모든 위젯을 표대로 맞추고, 점 여부(`available`/`downloading`/`download_failed`/`applying`)가 바뀌었을 때만 `available_changed` 를 낸다. `action` 하나가 상태에 따라 확인/업데이트를 맡는다(`clicked` → `_on_action`). 최신 시각 `datetime.now():%H:%M`. QSS: `#updateTitle { font-size: 15px; font-weight: 600; color: $heading; }`, `QTextBrowser#updateNotes { background: transparent; border: none; border-top: 1px solid $border; padding-top: 8px; }`.
- [ ] **Step 4:** 전체 통과 → 커밋 `업데이트 화면`.

### Task 3: 메인 창에 붙이고 업데이트 창 지우기

**Files:** Modify `ytdl/main_window.py`, `app.py`, `tests/test_main_window.py`; Delete `ytdl/update_dialog.py`

**Interfaces:** Consumes Task 1·2. Produces `MainWindow.update_page`, `MainWindow._confirm_update() -> bool`, `pages["update"]`.

- [ ] **Step 1: 실패하는 테스트**: `test_menu_opens_update_page`, `test_update_dot_follows_page`(`win.update_page.available_changed.emit(True)` → `win.sidebar.nav["update"].dot`), `test_confirm_without_jobs`(running/pending 비면 True, `QMessageBox.question` 안 불림), `test_confirm_with_jobs`(작업 1개 넣고 `QMessageBox.question`을 가짜로 No → False, 문구에 `받는 중인 영상 1개가 멈춰요`), `test_close_stops_update_download`(`update_page.stop`이 불림).
- [ ] **Step 2:** 실행 → 실패.
- [ ] **Step 3:** 구현. 화면 묶음에 `update` 추가, `available_changed` → `self._update_dot`(bound) → `sidebar.set_dot("update", on)`. `closeEvent`에서 `update_page.stop()`. `app.py`는 `win.show()` 뒤 `win.update_page.check()`, `update_dialog` import 삭제. `update_dialog.py` 삭제.
- [ ] **Step 4:** 전체 통과 + 종료 코드 0 → 커밋 `업데이트 창을 메뉴 화면으로`.

### Task 4: 눈으로 확인

- [ ] 상태별(idle·available·downloading·latest·check_failed) 라이트·다크 `grab()` → 시안과 비교, 어긋나면 고치고 커밋 `업데이트 화면 모양 맞추기`.
