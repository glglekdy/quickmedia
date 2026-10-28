# 왼쪽 메뉴와 설정 화면 구현 계획

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 설정 창을 없애고 메인 창 왼쪽 메뉴(다운로드 / 설정 ▸ 로그인·네트워크·파일·고급) 안의 화면으로 바꾼다. 바꾸면 바로 저장한다.

**Architecture:** 순수 함수 `update_settings`가 검사를 맡고, `SettingsModel`이 저장된 설정 하나를 네 화면에 나눠 준다. `Sidebar`는 `page_changed(str)`만 내보내고, `MainWindow`가 `QStackedWidget`으로 화면을 바꾼다.

**Tech Stack:** Python 3.14, PySide6 6.12 (Qt Widgets), pytest (offscreen `qapp` 픽스처)

**Spec:** `docs/superpowers/specs/2026-10-29-quickmedia-sidebar-settings-design.md` (시안: 같은 폴더 `...-mockup.html`)

## Global Constraints

- 화면 문구, 크기, 동작은 스펙의 값을 글자 그대로 쓴다(설명·안내 문구, 112px/160px 이름 칸, 168px 메뉴, 200ms OutCubic, 1.5초 저장 표시, 창 800×680 / 최소 690×480).
- 색은 `theme.py` 토큰만 쓴다. 새 토큰은 `nav_hover` 하나(LIGHT `#f0f0f0`, DARK `#232323`).
- `ytdl/update_dialog.py`, 다운로드 화면 내용(⚙ 버튼 제외)은 건드리지 않는다.
- 커밋 메시지는 한국어 한 줄 + 끝에 `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` / `Claude-Session: https://claude.ai/code/session_01SwAsdehfynj5X9tzWpurp8`.
- 테스트: `python -m pytest -q` 전체가 통과해야 각 작업을 커밋한다.

## Review Focus

- 글자를 쓴 채로 바로 메뉴를 누름 → 맞는 값은 저장(Task 4 `test_switching_page_saves_pending_text`).
- 저장된 쿠키 파일이 지워진 상태에서 다른 칸을 바꿈 → 다른 칸은 저장(Task 1 `test_stale_cookie_file_does_not_block_proxy`).
- 틀린 값을 쓴 뒤 저장된 값과 같은 값으로 되돌림 → 오류 표시가 지워짐(Task 3 `test_error_clears_when_reverted_to_saved`).
- 'file' 쿠키를 골랐다가 파일 없이 다른 화면으로 감 → 쿠키 방식이 전 값으로 돌아감(Task 3 `test_leave_restores_unsaved_cookie_mode`).
- 설정을 보던 중 접고 다시 펼침 → 화면 그대로, 신호 없음(Task 2 `test_reexpand_keeps_settings_page`).

---

### Task 1: `update_settings`

**Files:** Modify `ytdl/options.py` (`validate_settings` 아래), Test `tests/test_options_validate.py`

**Interfaces:** Produces `update_settings(s: AppSettings, **fields) -> tuple[AppSettings, dict[str, str]]`.

- [ ] **Step 1: 실패하는 테스트** — `test_update_settings_saves_valid`(`proxy="http://p:1"` → 새 값, `{}`), `test_update_settings_rejects_bad_rate`(`rate_limit="fast"` → `s` 그대로, `{"rate_limit": "속도 제한은 5M, 500K 처럼 써 주세요"}`), `test_stale_cookie_file_does_not_block_proxy`(`AppSettings(cookies="file", cookie_file="없는.txt")`에 `proxy=` → 저장, 오류 없음), `test_cookie_mode_file_checks_path`(`cookies="file", cookie_file="없는.txt"` → `cookie_file` 오류, 그대로), `test_custom_template_checked_with_name`(`name_template="custom", custom_template=" "` → `custom_template` 오류), `test_bad_extra_args`(`extra_args="--nope"` → `extra_args` 오류).
- [ ] **Step 2:** `python -m pytest tests/test_options_validate.py -q` → ImportError 로 실패.
- [ ] **Step 3:** 구현. 원인 표 `_ERROR_CAUSES = {"rate_limit": {"rate_limit"}, "cookie_file": {"cookies", "cookie_file"}, "custom_template": {"name_template", "custom_template"}, "extra_args": {"extra_args"}}`로 `validate_settings(replace(s, **fields))` 오류를 거른다.
- [ ] **Step 4:** 전체 테스트 통과 → 커밋 `설정 칸 하나씩 검사해 저장하는 update_settings`.

### Task 2: 왼쪽 메뉴 `Sidebar`

**Files:** Create `ytdl/sidebar.py`, `ytdl/assets/download.svg`; Modify `ytdl/theme.py`; Test `tests/test_sidebar.py`

**Interfaces:** Produces `SETTINGS_KEYS = ("login", "network", "file", "advanced")`; `class Sidebar(QWidget)` — 신호 `page_changed = Signal(str)`, 속성 `current: str`, `expanded: bool`, 메서드 `select(key: str)`, 위젯 `nav: dict[str, NavItem]`(키 `download`, `settings`, 네 설정 키), `anim`(펼침 `QParallelAnimationGroup`).

- [ ] **Step 1: 실패하는 테스트** (`nav[...].click()`, 신호는 리스트에 모음): `test_starts_on_download`(current `download`, 접힘, `nav["download"].isChecked()`), `test_settings_expands_to_login`(신호 `["login"]`, expanded), `test_settings_again_collapses`(두 번째 클릭에 신호 추가 없음, 접힘, `nav["settings"].isChecked()`), `test_category_and_download`(`network` 클릭 → `"network"`; `download` 클릭 → `"download"`, 접힘), `test_reexpand_keeps_settings_page`(file 선택 → 설정 접기 → 다시 펼치기: current `file`, 신호 추가 없음), `test_same_item_no_signal`.
- [ ] **Step 2:** 실행 → 실패 확인.
- [ ] **Step 3:** 구현. `NavItem(QAbstractButton)`이 `paintEvent`로 바탕(checked `btn_bg`, hover `nav_hover`), 빨간 막대(3px, 위아래 9px, `accent`), 아이콘 16px, 글자(13px, checked 600)를 그린다; 하위 항목은 아이콘 없이 왼쪽 36px 들여쓰기. '설정' 항목은 화살표(`muted` 선 두 개)를 `angle` 속성만큼 돌려 그린다. 하위 항목 상자의 `maximumHeight`와 화살표 `angle`을 200ms OutCubic 으로 함께 움직인다(상태는 즉시). `theme.py`: LIGHT/DARK 에 `nav_hover`, QSS `#sidebar { background: $surface; border-right: 1px solid $border; }`. `download.svg`는 Feather download, `gear.svg`와 같은 속성(`stroke="#8c8c8c"`).
- [ ] **Step 4:** 전체 테스트 통과 → 커밋 `왼쪽 메뉴`.

### Task 3: 설정 화면 4개

**Files:** Create `ytdl/settings_pages.py`, Test `tests/test_settings_pages.py`; Modify `ytdl/theme.py`, `tests/test_option_widgets.py`(SettingsDialog 테스트 3개 삭제)

**Interfaces:** Consumes `update_settings`. Produces `class SettingsModel(QObject)` — `value: AppSettings`, `changed = Signal(object)`, `update(**fields) -> dict[str, str]`; `class SettingsPage(QWidget)` — `load(s)`, `leave()`, `errors: dict[str, QLabel]`, `saved_badge: QLabel`; `LoginPage/NetworkPage/FilePage/AdvancedPage(model)`; `PAGES: dict[str, type[SettingsPage]]`(키는 `SETTINGS_KEYS`). 칸 이름은 옛 `SettingsDialog`와 같다(`cookie_none/cookie_browser/cookie_file`, `browser`, `cookie_path`, `browse_btn`, `proxy`, `rate_limit`, `retries`, `concurrency`, `template`, `custom_template`, `overwrite`, `extra_args`).

- [ ] **Step 1: 실패하는 테스트**: `test_round_trip`(옛 round-trip 값으로 모델을 만들고 네 화면을 load → 각 칸 값이 같다), `test_checkbox_saves_immediately`(`overwrite.click()` → `model.value.overwrite`, 신호 1번, `not saved_badge.isHidden()`), `test_bad_rate_shows_error`(`rate_limit.setText("fast"); editingFinished.emit()` → 오류 문구, `property("invalid")`, 저장 안 됨; `"5M"` → 저장, 오류 숨김), `test_error_clears_when_reverted_to_saved`, `test_cookie_file_without_file`(`cookie_file.click()` → 오류, `model.value.cookies == "none"`), `test_leave_saves_valid_pending_text`(`proxy.setText("http://x:1")` 후 `leave()` → 저장), `test_leave_reverts_invalid`(틀린 rate → `leave()` → 칸은 저장값, 오류 숨김), `test_leave_restores_unsaved_cookie_mode`.
- [ ] **Step 2:** 실행 → 실패 확인.
- [ ] **Step 3:** 구현. 스펙 "설정 화면 공통·화면별 내용·저장"을 그대로. 제목 `#crumb`, 설명 `#viewHint`, 안내 `#hint`, 오류 `#fieldError`, 카드 `QFrame#settingsCard`, 저장 표시 `#savedBadge`(QTimer 1500ms singleShot 재시작), 스크롤 바탕 `#settingsHolder`. 글자 칸은 `editingFinished`, 나머지는 바뀜 신호에 저장; 'file' 쿠키·'직접 입력'은 묶인 칸을 같이 넘긴다; `load()` 중에는 저장하지 않는다. `leave()`는 저장값과 다른 글자 칸을 먼저 넘기고 `load(model.value)` + 오류 지우기. QSS 추가: `#settingsCard { background: $card_bg; border: 1px solid $border; border-radius: 10px; }`, `#savedBadge { color: $success; font-size: 12px; }`, `QLineEdit[invalid="true"] { border-color: $danger; }`, 스크롤 규칙에 `#settingsHolder`.
- [ ] **Step 4:** 전체 테스트 통과 → 커밋 `설정 화면 4개와 바로 저장`.

### Task 4: 메인 창에 붙이기

**Files:** Modify `ytdl/main_window.py`, `tests/test_main_window.py`; Delete `ytdl/settings_dialog.py`

**Interfaces:** Consumes `Sidebar`, `SettingsModel`, `PAGES`. Produces `MainWindow.sidebar`, `MainWindow.stack: QStackedWidget`, `MainWindow.pages: dict[str, QWidget]`(download 포함), `MainWindow.settings_model`.

- [ ] **Step 1: 실패하는 테스트**: `test_settings_change_applies`(`win.settings_model.update(concurrency=4, proxy="http://p:1")` → `win.settings.value("settings/concurrency")` 가 4, `win.jobs.limit == 4`, `win.icons.proxy == "http://p:1"`, `win.app_settings.concurrency == 4`), `test_menu_switches_page`(`win.sidebar.nav["settings"].click()` → `win.stack.currentWidget() is win.pages["login"]`), `test_switching_page_saves_pending_text`(network 화면 proxy 에 글자 → download 클릭 → 저장), `test_window_size`(`minimumWidth() == 690`).
- [ ] **Step 2:** 실행 → 실패 확인.
- [ ] **Step 3:** 구현. 지금 `_build_ui`의 `main` 위젯을 다운로드 화면으로 `stack`에 넣고, 중앙 위젯은 `[sidebar | stack]`. ⚙ 버튼·`open_settings`·`SettingsDialog` import 삭제, `settings_dialog.py` 삭제. `resize(800, 680)`, `setMinimumSize(690, 480)`. `page_changed` → 이전 화면이 `SettingsPage`면 `leave()` 후 전환. `closeEvent` 첫머리에서 보고 있는 설정 화면 `leave()`.
- [ ] **Step 4:** 전체 테스트 통과 → 커밋 `메인 창을 왼쪽 메뉴 + 화면 묶음으로`.

### Task 5: 눈으로 확인

- [ ] 각 화면을 라이트·다크로 `grab()` 해 시안과 비교, 어긋나면 고치고 테스트 재실행 후 커밋 `메뉴·설정 화면 모양 맞추기`.
