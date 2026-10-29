# QuickMedia 안드로이드 UI 개편 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 안드로이드 앱 화면을 시안 A(다크 미디어) + 떠 있는 반투명 메뉴의 세 탭(받기·목록·설정)으로 바꾸고, 미리보기·목록 썸네일·다시 받기를 더한다.

**Architecture:** 순수 로직(미리보기 JSON 읽기, 숫자 표시, 미리보기 기억, 목록 필터, 옵션 요약, 다시 받기)을 먼저 TDD 로 만든다. 그 위에 테마와 공통 부품, 세 탭 화면을 따로 만든다. 마지막에 `AppShell` 이 탭·메뉴·스낵바·공유 링크를 묶고, 옛 `MainScreen`/`SettingsScreen` 을 지운다. 다운로드·업데이트 코드는 건드리지 않는다.

**Tech Stack:** Kotlin, Jetpack Compose(Material 3 BOM 2026.09.00), Haze 2.0.1, Coil 3.6.3, Pretendard 1.3.9, youtubedl-android 0.18.1, JUnit 4, kotlinx-coroutines-test 1.11.0.

**Spec:** `docs/superpowers/specs/2026-10-29-quickmedia-android-ui-design.md`

## Global Constraints

- 패키지 `com.glglekdy.quickmedia`, 코드는 `android/app/src/main/java/com/glglekdy/quickmedia/`, 테스트는 `android/app/src/test/java/com/glglekdy/quickmedia/`.
- minSdk 29. 흐림은 안드로이드 12+ 만, 10~11 은 흐림 없는 반투명(Haze 가 자동 대체). 10~11 에서 죽으면 안 된다.
- 새 의존성은 정확히: `dev.chrisbanes.haze:haze:2.0.1`, `io.coil-kt.coil3:coil-compose:3.6.3`, `io.coil-kt.coil3:coil-network-okhttp:3.6.3`, 테스트용 `org.jetbrains.kotlinx:kotlinx-coroutines-test:1.11.0`. `gradle/libs.versions.toml` 에 등록해서 쓴다.
- 글꼴: Pretendard 1.3.9 `PretendardVariable.ttf` → `res/font/pretendard_variable.ttf`. 라이선스는 `android/third_party/pretendard/LICENSE.txt`.
- 색은 스펙의 "색" 표 값을 그대로 쓴다. 모서리: 카드 20dp, 입력칸 16dp, 버튼 18dp, 칩·메뉴는 알약.
- 화면 문구는 스펙에 적힌 그대로(존댓말 "~요"). 코드 주석은 기존처럼 한국어로, 왜 그런지만 짧게.
- 다운로드(`DownloadService`, `YtdlpArgs`)·업데이트(`Updater`, `UpdateDialog` 의 동작)·서명·워크플로는 바꾸지 않는다. `Jobs` 에는 `retry` 만 더한다.
- 커밋 메시지는 한국어 한 줄 제목 + 빈 줄 + 아래 두 줄:
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`
  `Claude-Session: https://claude.ai/code/session_01SwAsdgEacjHBe8VSrqB25G`
- 빌드·테스트는 `android/` 에서 `JAVA_HOME="/c/Program Files/Android/Android Studio/jbr"`. 계획 워크스페이스의 `ut.sh <gradle 인자>` (옛 워크스페이스의 `ut.sh` 를 복사하고 로그 경로만 이 계획 워크스페이스로 바꾼다)가 마지막 줄에 `BUILD SUCCESSFUL - unit tests N/N passed` 를 찍는다.

## Review Focus

1. 링크를 빠르게 바꿨을 때(A 붙여넣고 곧바로 B) 받기 카드에 A 의 미리보기가 B 처럼 보이면 안 된다 → Task 2 `focusChangeCancelsUnpinned`, Task 7 은 지금 링크의 상태만 그린다.
2. 영상이 아닌 링크·막힌 영상·네트워크 없음 → 미리보기는 "영상 정보를 가져오지 못했어요" 카드, 다운로드는 그대로 된다 → Task 1 `garbageAndNoTitle`, Task 2 `failedNotCached`, Task 8 에뮬레이터 확인.
3. 이모지·한글이 긴 제목과 채널 이름 → 카드·목록에서 말줄임으로 끝나고 레이아웃이 밀리지 않는다 → Task 8 에뮬레이터 확인 목록.
4. 떠 있는 메뉴가 목록 마지막 항목·설정 마지막 줄·다운로드 버튼을 가리면 안 되고, 키보드가 올라오면 숨는다 → Task 8 에뮬레이터 확인 목록.
5. 다운로드를 누른 링크의 미리보기는 링크 칸을 비워도 취소되지 않고 끝까지 받아 목록에 채워진다 → Task 2 `pinnedSurvivesFocusChange`.

---

### Task 1: 미리보기 정보 읽기와 숫자 표시

**Files:**
- Create: `preview/Preview.kt`, `preview/Format.kt`
- Test: `preview/PreviewTest.kt`, `preview/FormatTest.kt`

**Interfaces:**
- Produces:
  - `data class Preview(val title: String, val channel: String?, val thumbnail: String?, val duration: Long?, val views: Long?, val isPlaylist: Boolean, val count: Int?)`
  - `Preview.parse(json: String): Preview?` (`companion object` 안)
  - `fun formatDuration(seconds: Long?): String?`, `fun formatViews(count: Long?): String?`

- [ ] **Step 1: 실패하는 테스트 작성** — `PreviewTest`:

```kotlin
@Test fun video() {
    val p = Preview.parse("""{"title":"Never Gonna","uploader":"Rick Astley","duration":213.0,"view_count":1500000000,"thumbnail":"https://i.ytimg.com/vi/x/maxres.jpg"}""")!!
    assertEquals(Preview("Never Gonna", "Rick Astley", "https://i.ytimg.com/vi/x/maxres.jpg", 213, 1_500_000_000, false, null), p)
}
@Test fun channelWhenNoUploader() = assertEquals("Ch", Preview.parse("""{"title":"t","channel":"Ch"}""")!!.channel)
@Test fun thumbnailsLastWhenNoThumbnail() = assertEquals("b",
    Preview.parse("""{"title":"t","thumbnails":[{"url":"a"},{"url":"b"}]}""")!!.thumbnail)
@Test fun playlistCount() {
    val p = Preview.parse("""{"_type":"playlist","title":"여름","uploader":"u","playlist_count":12,"entries":[]}""")!!
    assertTrue(p.isPlaylist); assertEquals(12, p.count)
}
@Test fun playlistCountFromEntriesAndFirstEntryThumbnail() {
    val p = Preview.parse("""{"_type":"playlist","title":"여름","entries":[{"thumbnails":[{"url":"e1"},{"url":"e2"}]},{}]}""")!!
    assertEquals(2, p.count); assertEquals("e2", p.thumbnail)
}
@Test fun garbageAndNoTitle() {
    assertNull(Preview.parse("not json"))
    assertNull(Preview.parse("""{"uploader":"u"}"""))
    assertNull(Preview.parse("""{"title":""}"""))
}
```

`FormatTest`:

```kotlin
@Test fun duration() {
    assertNull(formatDuration(null)); assertNull(formatDuration(0)); assertNull(formatDuration(-5))
    assertEquals("0:59", formatDuration(59)); assertEquals("3:33", formatDuration(213)); assertEquals("1:02:03", formatDuration(3723))
}
@Test fun views() {
    assertNull(formatViews(null))
    assertEquals("0회", formatViews(0)); assertEquals("1,234회", formatViews(1234)); assertEquals("9,999회", formatViews(9999))
    assertEquals("1만 회", formatViews(10_000)); assertEquals("1.2만 회", formatViews(12_345)); assertEquals("9.9만 회", formatViews(99_999))
    assertEquals("10만 회", formatViews(100_000)); assertEquals("12만 회", formatViews(123_456)); assertEquals("9,999만 회", formatViews(99_999_999))
    assertEquals("1억 회", formatViews(100_000_000)); assertEquals("1.5억 회", formatViews(150_000_000)); assertEquals("15억 회", formatViews(1_500_000_000))
}
```

- [ ] **Step 2: 실패 확인** — Run: `ut.sh testDebugUnitTest --tests "*.preview.*"` Expected: 컴파일 실패(`Unresolved reference: Preview`).

- [ ] **Step 3: 구현** — `Preview.parse` 는 `org.json.JSONObject`, 깨진 JSON 은 null. 문자열 값은 비었으면 없는 것으로 본다(`optString` 빈 문자열 → null). `duration` 은 소수 버림. 재생목록 썸네일: 자기 `thumbnails` 마지막 → 첫 항목 `thumbnails` 마지막 → null. `formatViews` 는 스펙 "숫자 표시" 규칙(소수 버림, `.0` 빼기, 정수 부분은 세 자리 쉼표).

- [ ] **Step 4: 통과 확인** — 같은 명령. Expected: `BUILD SUCCESSFUL`, preview 테스트 전부 통과.

- [ ] **Step 5: 커밋** — `안드로이드: 미리보기 정보 읽기와 숫자 표시`

---

### Task 2: 미리보기 기억과 불러오기

**Files:**
- Create: `preview/PreviewStore.kt`, `preview/Previews.kt`
- Modify: `android/gradle/libs.versions.toml`, `android/app/build.gradle.kts` (테스트 의존성 `kotlinx-coroutines-test`)
- Test: `preview/PreviewStoreTest.kt`

**Interfaces:**
- Consumes: `Preview` (Task 1)
- Produces:
  - `sealed interface PreviewState { data object Loading; data class Ready(val preview: Preview); data object Failed }`
  - `class PreviewStore(scope: CoroutineScope, fetch: suspend (url: String) -> Preview?)` with
    `val states: StateFlow<Map<String, PreviewState>>`,
    `fun load(url: String)` — Ready·Loading 이면 아무것도 안 함, 아니면 Loading 으로 두고 fetch,
    `fun focus(url: String?)` — 받기 칸이 지금 보는 링크. 전에 보던 링크가 Loading 이고 pin 되지 않았으면 그 fetch 를 취소하고 상태를 지운다,
    `fun pin(url: String)` — 다운로드에 쓰인 링크. 취소하지 않고, 없거나 Failed 면 다시 load.
  - `object Previews` — 앱 전체 하나: `val store = PreviewStore(App.scope, ::fetchWithYtdlp)` 와 `states`/`load`/`focus`/`pin` 을 그대로 넘긴다.

- [ ] **Step 1: 실패하는 테스트 작성** — `runTest` 와 `backgroundScope` 를 store 의 scope 로, fetch 는 `CompletableDeferred` 로 막아 둔 가짜. 이름과 확인할 것:
  - `loadsOnceAndCachesReady`: `load(a)` 두 번 → fetch 1회, 끝나면 `states.value[a] == Ready(p)`. 다시 `load(a)` → fetch 여전히 1회.
  - `failedNotCached`: fetch 가 null → `Failed`. 다시 `load(a)` → fetch 2회.
  - `focusChangeCancelsUnpinned`: `focus(a); load(a)` 중에 `focus(b)` → a 의 fetch 코루틴이 취소됨(`isCancelled`), `states.value` 에 a 없음.
  - `pinnedSurvivesFocusChange`: `focus(a); load(a); pin(a); focus(null)` → a 는 Loading 그대로, fetch 가 끝나면 Ready.
  - `pinReloadsFailed`: a 가 Failed 일 때 `pin(a)` → fetch 다시, 끝나면 Ready.
  - `fetchExceptionIsFailed`: fetch 가 예외(취소 말고) → `Failed`.

- [ ] **Step 2: 실패 확인** — Run: `ut.sh testDebugUnitTest --tests "*.PreviewStoreTest"` Expected: 컴파일 실패(`PreviewStore` 없음).

- [ ] **Step 3: `PreviewStore` 구현** — 링크별 `kotlinx.coroutines.Job` 맵과 pin 집합을 둔다. 맵은 `MutableStateFlow.update` 로만 바꾼다.

- [ ] **Step 4: 통과 확인** — 같은 명령. Expected: 6/6 통과.

- [ ] **Step 5: `Previews.fetchWithYtdlp(url): Preview?` 구현** — `App.tools.first { it !is Tools.Loading }` 가 `Ready` 가 아니면 null. `Dispatchers.IO` 에서 `YoutubeDLRequest(url)` + `-J --flat-playlist --no-playlist --no-warnings`, 프로세스 id `"preview-<증가 번호>"`. 코루틴이 취소되면 `YoutubeDL.getInstance().destroyProcessById(id)` (예: `currentCoroutineContext().job.invokeOnCompletion { if (it is CancellationException) destroy… }`). 결과 `out` 을 `Preview.parse`. yt-dlp 예외는 null.

- [ ] **Step 6: 전체 테스트** — Run: `ut.sh testDebugUnitTest` Expected: `BUILD SUCCESSFUL`, 전부 통과.

- [ ] **Step 7: 커밋** — `안드로이드: 미리보기 기억과 불러오기`

---

### Task 3: 목록 필터 · 다시 받기 · 옵션 요약

**Files:**
- Create: `ui/list/JobFilter.kt`
- Modify: `download/Jobs.kt` (`retry` 추가), `options/Summary.kt` (`changedSummary` → `optionsSummary`)
- Test: `ui/list/JobFilterTest.kt`, `download/JobsTest.kt`, `options/SummaryTest.kt` (`changedSummary` 테스트 3개를 아래로 바꾼다)

**Interfaces:**
- Consumes: `Job`, `JobState`, `Jobs`, `DownloadOptions`
- Produces:
  - `enum class JobFilter(val label: String) { ALL("전체"), ACTIVE("받는 중"), DONE("완료"), FAILED("실패"); fun matches(job: Job): Boolean }` — ACTIVE = QUEUED·RUNNING, FAILED = FAILED·CANCELLED
  - `fun jobCounts(jobs: List<Job>): Map<JobFilter, Int>` (네 키 모두)
  - `data class JobGroup(val title: String?, val jobs: List<Job>)`, `fun jobGroups(jobs: List<Job>, filter: JobFilter): List<JobGroup>` — ALL 은 `"받는 중"`/`"끝남"` 중 비지 않은 것만, 나머지는 걸러진 게 있으면 `JobGroup(null, …)` 하나, 없으면 빈 목록. 순서는 입력 순서 유지.
  - `fun Jobs.retry(id: Long): Boolean`
  - `fun optionsSummary(o: DownloadOptions): String?`

- [ ] **Step 1: 실패하는 테스트 작성**

`JobFilterTest` (작업은 `Job(id, "u$id", DownloadOptions(), state = …)` 로 만든다):
  - `countsByFilter`: QUEUED, RUNNING, DONE, FAILED, CANCELLED 하나씩 → ALL 5, ACTIVE 2, DONE 1, FAILED 2.
  - `allSplitsIntoTwoGroups`: 위 목록 → `[JobGroup("받는 중", [Q,R]), JobGroup("끝남", [D,F,C])]`.
  - `emptyGroupHidden`: DONE 만 → `[JobGroup("끝남", [D])]`.
  - `filteredSingleUntitledGroup`: FAILED 필터 → `[JobGroup(null, [F,C])]`; 맞는 게 없으면 `emptyList()`.

`JobsTest` 에 추가:
  - `retryFailedOrCancelled`: 실패 작업(percent 40, saved 1개) → `retry` true, 상태 QUEUED·percent 0·status `"대기 중"`·saved 빈 목록, 목록 위치 그대로. 취소 작업도 true.
  - `retryIgnoresActiveAndDone`: RUNNING·DONE → false, 그대로.

`SummaryTest` 의 `changedSummary` 테스트 대신:
  - `summaryNullByDefault`: `optionsSummary(DownloadOptions())` == null. `PlaylistMode.ITEMS` + 빈 항목도 `"재생목록 일부"` (null 아님).
  - `summaryEach`: ALL → `"전체 재생목록"`, ITEMS `" 1-5,8 "` → `"재생목록 1-5,8"`, embedMeta → `"메타데이터"`, 시작 `"1:23"` → `"1:23부터"`, 끝 `"1:23"` → `"1:23까지"`, 둘 다 `"1:23"`/`"2:00"` → `"1:23~2:00"`.
  - `summaryJoined`: ALL + embedMeta + 시작 `"1:23"` → `"전체 재생목록 · 메타데이터 · 1:23부터"`.

- [ ] **Step 2: 실패 확인** — Run: `ut.sh testDebugUnitTest` Expected: 컴파일 실패(`JobFilter`, `retry`, `optionsSummary` 없음).

- [ ] **Step 3: 구현** — 위 시그니처대로. `changedSummary` 는 지운다(옛 `MainScreen` 이 쓰고 있으니 그 호출을 `optionsSummary(options)` 로 바꿔 컴파일을 지킨다. 옛 화면은 Task 8 에서 지운다).

- [ ] **Step 4: 통과 확인** — 같은 명령. Expected: `BUILD SUCCESSFUL`, 전부 통과.

- [ ] **Step 5: 커밋** — `안드로이드: 목록 필터와 다시 받기, 옵션 요약`

---

### Task 4: 테마 · 글꼴 · 공통 부품

**Files:**
- Create: `ui/theme/Colors.kt`, `ui/theme/Type.kt`, `ui/theme/Theme.kt`, `ui/Tab.kt`, `ui/components/{Dock,Chip,SegmentToggle,GradientButton,Skeleton,Thumbnail,SectionCard}.kt`, `res/font/pretendard_variable.ttf`, `res/drawable/{ic_download,ic_list,ic_play,ic_retry,ic_video,ic_audio}.xml`, `android/third_party/pretendard/{LICENSE.txt,README.md}`
- Delete: `ui/Theme.kt` (옛 `QuickMediaTheme` 을 쓰는 곳은 새 패키지 `ui.theme.QuickMediaTheme` 로 import 를 바꾼다)
- Modify: `gradle/libs.versions.toml`, `app/build.gradle.kts` (haze, coil-compose, coil-network-okhttp), `res/values/themes.xml`·`res/values-night/themes.xml` (창 바탕을 `bg` 색으로 맞춰 켤 때 흰 번쩍임이 없게)

**Interfaces:**
- Produces:
  - `data class QmColors(val bg: Color, val surface: Color, val field: Color, val fieldOn: Color, val text: Color, val textDim: Color, val accent: Color, val accentText: Color, val ok: Color, val bad: Color, val dock: Color, val isDark: Boolean)`, `val DarkColors`, `val LightColors` (스펙 표 값), `val AccentGradient: Brush` (`#FF3B3B → #FF6A3B` 가로)
  - `object Qm { val colors: QmColors @Composable @ReadOnlyComposable get() }` (`LocalQmColors`)
  - `val Pretendard: FontFamily` — 400·500·600·700·800 굵기, 한 파일에서 `FontVariation.Settings(FontVariation.weight(w))`
  - `@Composable fun QuickMediaTheme(content: @Composable () -> Unit)` — `QmColors` 를 내려 주고, Material `colorScheme`(primary=accent, background/surface=bg·surface, onSurface=text, onSurfaceVariant=textDim, surfaceContainer*=surface/field, error=bad)와 `Typography`(모두 Pretendard)를 같은 값으로.
  - `enum class Tab(val label: String, @DrawableRes val icon: Int) { RECEIVE("받기", R.drawable.ic_download), LIST("목록", R.drawable.ic_list), SETTINGS("설정", R.drawable.ic_settings) }`
  - `@Composable fun Dock(selected: Tab, onSelect: (Tab) -> Unit, badge: Int, hazeState: HazeState, modifier: Modifier = Modifier)` — 알약, 바탕 `dock` 색 위에 Haze 흐림(`hazeEffect`), 얇은 흰 테두리 10%, 그림자. 고른 탭은 `accent` 18% 바탕 + `accentText`. `badge > 0` 이면 목록 아이콘 위에 빨강 숫자.
  - `@Composable fun Chip(text: String, selected: Boolean, onClick: () -> Unit, inverse: Boolean = false, modifier: Modifier = Modifier)` — 고르면 `accent` 바탕 흰 글자, `inverse` 면 `text` 바탕 `bg` 글자.
  - `@Composable fun <T> SegmentToggle(values: List<T>, selected: T, label: (T) -> String, onSelect: (T) -> Unit, modifier: Modifier = Modifier)` — `field` 바탕 안에서 고른 칸만 `fieldOn`.
  - `@Composable fun GradientButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier)` — 높이 52dp, 모서리 18dp, `AccentGradient`, 빨강 그림자.
  - `@Composable fun Skeleton(modifier: Modifier, shape: Shape = RoundedCornerShape(12.dp))` — `field` 색이 은은하게 밝아졌다 어두워지는 반짝임.
  - `@Composable fun Thumbnail(url: String?, @DrawableRes fallbackIcon: Int, modifier: Modifier, shape: Shape)` — Coil `AsyncImage`(가득 채워 자르기). url 이 없거나 실패하면 `field` 칸 가운데 `fallbackIcon`.
  - `@Composable fun SectionCard(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit)` — `surface`, 모서리 20dp.

- [ ] **Step 1: 의존성 · 글꼴 · 아이콘 넣기** — 글꼴은 `https://cdn.jsdelivr.net/npm/pretendard@1.3.9/dist/public/variable/PretendardVariable.ttf`, 라이선스는 같은 패키지의 `LICENSE`. README 에 출처·버전·라이선스(SIL OFL 1.1)를 적는다. 아이콘은 24dp 벡터, 선 굵기 2.2 둥근 끝(시안 SVG 와 같은 모양): 내려받기 화살표, 세 줄 목록, 재생 삼각형, 다시(둥근 화살표), 영상(필름/재생 사각), 음성(음표).

- [ ] **Step 2: 테마와 부품 구현** — 위 시그니처대로. Haze 2.0.1 의 정확한 함수 이름(`rememberHazeState`, `Modifier.hazeSource`, `Modifier.hazeEffect`)은 받은 라이브러리 소스로 확인하고, 다르면 2.0.1 이름을 쓴다. 옛 화면들은 새 `QuickMediaTheme` 로만 바꿔서 그대로 돌아가야 한다.

- [ ] **Step 3: 빌드와 테스트** — Run: `ut.sh testDebugUnitTest assembleDebug` Expected: `BUILD SUCCESSFUL`, 전부 통과. 경고에 Haze/Coil 관련 deprecation 이 없을 것.

- [ ] **Step 4: 커밋** — `안드로이드: 새 테마와 글꼴, 공통 부품`

---

### Task 5: 목록 탭

**Files:**
- Create: `ui/list/ListScreen.kt`, `ui/list/JobRow.kt`
- Modify: (없음 — 아직 연결하지 않는다. Task 8 이 붙인다)

**Interfaces:**
- Consumes: `JobFilter`, `jobCounts`, `jobGroups`, `Jobs.retry` (Task 3), `Previews.states`/`Previews.load` (Task 2), 부품들 (Task 4), `describe(options)`, `DownloadService.start/cancel`
- Produces: `@Composable fun ListScreen(onGoReceive: () -> Unit, say: (String) -> Unit, contentPadding: PaddingValues)`
  - 파일 열기 함수 `openJob(context, job): String?` 는 옛 `MainScreen.open` 을 이 파일로 옮긴다(문구 `"이 파일을 열 수 있는 앱이 없어요"` 그대로).

- [ ] **Step 1: 구현** — 스펙 "목록 탭" 그대로:
  - 머리 `목록` + `끝난 항목 지우기`(끝난 게 없으면 흐리고 못 누름).
  - 필터 칩 `전체 N`·`받는 중 N`·`완료 N`·`실패 N`(`inverse = true`), 고른 필터는 `rememberSaveable`.
  - `LazyColumn` 에 `jobGroups` 순서로 머리글(작은 흐린 굵은 글자)과 `JobRow`, `key = job.id`.
  - `JobRow`: 썸네일 96×56 모서리 12dp(`Previews.states[job.url]` 이 Ready 면 그 썸네일, 아니면 `ic_video`/`ic_audio`), 제목 1줄 말줄임(미리보기 제목 → `job.title`), 상태 줄(스펙의 세 모양, 점은 `ok`/`bad` 색), 받는 중이면 그라데이션 진행 막대(0% 면 왔다 갔다), 오른쪽 원형 버튼 ✕/▶/↻. 완료는 줄 전체도 누르면 열린다.
  - ↻: `if (Jobs.retry(id)) { DownloadService.start(context); Previews.load(job.url) }` (`load` 는 이미 Ready 면 아무것도 안 한다).
  - 비었을 때: 아이콘 + `아직 받은 게 없어요`(필터가 전체가 아니면 `해당하는 항목이 없어요`), 전체일 때만 `받기로 가기`(테두리 알약 버튼, `onGoReceive`).
  - 아래 여백은 `contentPadding` 을 그대로 쓴다(메뉴 높이를 Shell 이 넣어 준다).

- [ ] **Step 2: 빌드** — Run: `ut.sh testDebugUnitTest assembleDebug` Expected: `BUILD SUCCESSFUL`, 전부 통과.

- [ ] **Step 3: 커밋** — `안드로이드: 새 목록 탭`

---

### Task 6: 설정 탭

**Files:**
- Create: `ui/settings/SettingsScreen.kt`
- Modify: (없음 — Task 8 이 붙이고 옛 `ui/SettingsScreen.kt` 를 지운다. 이름이 같으니 패키지 `ui.settings` 로 구분한다)

**Interfaces:**
- Consumes: `Updater.check`, `UpdateException`, `Release`, 옛 `ui/SettingsScreen.kt` 의 `updateYtdlp(context)`/`ytdlpVersion(context)` (이 파일로 옮겨 쓴다), 부품들 (Task 4)
- Produces: `@Composable fun SettingsScreen(onNewVersion: (Release) -> Unit, say: (String) -> Unit, contentPadding: PaddingValues)`

- [ ] **Step 1: 구현** — 스펙 "설정 탭" 그대로:
  - 위 카드: `accent` 기운 그라데이션 바탕(다크 `#2A0F10 → surface`, 라이트 `#FFE4E2 → surface`, 135°), `Quick`+`Media`(빨강) + 버전, 상태 줄, `업데이트 확인` 빨강 알약.
  - 상태는 `enum class UpdateStatus { IDLE, CHECKING, LATEST, NEW, FAILED }` + 새 `Release?` + 실패 문구를 `rememberSaveable` 가능한 값으로. 문구: `업데이트를 확인해 보세요` / `확인하는 중…` / `최신 버전이에요` / `새 버전 ${release.version} 이 있어요`(이 줄을 누르면 `onNewVersion`) / `UpdateException.message`. 새 버전을 찾으면 바로 `onNewVersion` 도 부른다(지금과 같다).
  - 묶음 1: `yt-dlp` + 버전(준비 중 `확인 중…`, 도구 실패면 사유) + `갱신`(지금과 같은 조건·문구: 받는 중이면 `받는 중인 작업이 끝난 뒤에 갱신해 주세요`, 버튼 글자 `갱신 중…`), `저장 위치` + `다운로드/QuickMedia` + `열기`(`DownloadManager.ACTION_VIEW_DOWNLOADS`, 없으면 `say("이 파일을 열 수 있는 앱이 없어요")`).
  - 묶음 2: `GitHub에서 보기` › → `Intent.ACTION_VIEW` `https://github.com/glglekdy/quickmedia`.

- [ ] **Step 2: 빌드** — Run: `ut.sh testDebugUnitTest assembleDebug` Expected: `BUILD SUCCESSFUL`, 전부 통과.

- [ ] **Step 3: 커밋** — `안드로이드: 새 설정 탭`

---

### Task 7: 받기 탭

**Files:**
- Create: `ui/receive/ReceiveScreen.kt`, `ui/receive/PreviewCard.kt`, `ui/receive/OptionsSheet.kt`

**Interfaces:**
- Consumes: `Previews` (Task 2), `Preview`, `formatDuration`, `formatViews` (Task 1), `optionsSummary` (Task 3), 부품들 (Task 4), `extractLink`, `OptionsStore`, `YtdlpArgs.validate`, `Jobs.add`, `DownloadService.start`, `App.tools`, `QUALITIES`, `AUDIO_FORMATS`, `qualityLabel`
- Produces: `@Composable fun ReceiveScreen(link: String, instant: Boolean, onLinkChange: (text: String, instant: Boolean) -> Unit, say: (String) -> Unit, onQueued: () -> Unit, contentPadding: PaddingValues)`
  - `instant` 은 링크가 붙여넣기·공유로 채워졌는지. 직접 칠 때는 `onLinkChange(it, false)`.
  - `onQueued` 는 작업을 넣은 뒤 Shell 이 스낵바를 띄우게 부른다.

- [ ] **Step 1: 구현** — 스펙 "받기 탭" 그대로:
  - 순서: `Quick`+`Media` 제목(800 굵기 22sp, `Media` 빨강) → 도구 실패 띠(옛 `Banner` 와 같은 문구, `bad` 색 바탕 옅게) → 링크 칸(`field` 바탕 16dp, 테두리 없음, 비면 `붙여넣기`, 차면 ✕, 키보드 이동 = 다운로드, 빈 링크 오류 문구) → `PreviewCard` → `SegmentToggle(영상/음성)` → 칩 줄 → 옵션 줄 → `GradientButton("다운로드")`.
  - 미리보기: `val url = extractLink(link)`. `LaunchedEffect(url) { Previews.focus(url); if (url != null) { if (!instant) delay(600); Previews.load(url) } }`. 화면이 사라질 때(`DisposableEffect`) `Previews.focus(null)`. 도구가 `Failed` 면 카드를 그리지 않는다. 그릴 상태는 `states[url]` 하나만: 없음·Loading → 뼈대(도구 Loading 일 때도), Ready → 카드, Failed → 작은 실패 카드 `영상 정보를 가져오지 못했어요`.
  - `PreviewCard(preview: Preview)`: 16:9 썸네일(`Thumbnail`, 위 모서리 20dp), 오른쪽 아래 검은 반투명 꼬리표(영상은 `formatDuration`, 없으면 꼬리표 없음; 재생목록은 `재생목록 · N개`), 제목 2줄 말줄임 700 굵기, `채널 · 조회수`(없는 쪽은 빼고, 둘 다 없으면 줄 없음).
  - 옵션 줄: `optionsSummary(options)` 가 있으면 그 글자(`text`), 없으면 `재생목록 · 메타데이터 · 구간`(`textDim`), 오른쪽 `옵션 ›`. 누르면 `OptionsSheet`.
  - `OptionsSheet(options, errors, onChange, onDismiss)`: `ModalBottomSheet`(바탕 `surface`), 옛 `MoreOptions` 의 칸과 문구 그대로(재생목록 `SegmentToggle` 이 영상만·전체·일부 항목, 일부면 `받을 항목` 칸, `썸네일·메타데이터 넣기` 스위치, `구간 자르기` 시작·끝 칸과 도움말). 칸은 링크 칸과 같은 모양.
  - 다운로드: 옛 `download()` 순서 그대로, 검사에 걸리면 시트를 연다. 작업을 넣은 뒤 `Previews.pin(url)`, 링크 비우기(`onLinkChange("", false)`), 포커스 해제, `onQueued()`. 알림 권한 한 번 묻기(`askNotificationsOnce`)와 붙여넣기(`paste`, 문구 `복사한 글에 링크가 없어요`)는 옛 `MainScreen` 에서 이 파일로 옮긴다.

- [ ] **Step 2: 빌드** — Run: `ut.sh testDebugUnitTest assembleDebug` Expected: `BUILD SUCCESSFUL`, 전부 통과.

- [ ] **Step 3: 커밋** — `안드로이드: 새 받기 탭과 미리보기`

---

### Task 8: 탭 묶기 · 옛 화면 지우기 · 에뮬레이터 확인

**Files:**
- Create: `ui/AppShell.kt`
- Modify: `MainActivity.kt`, `ui/UpdateDialog.kt` (모양만)
- Delete: `ui/MainScreen.kt`, `ui/SettingsScreen.kt`, 안 쓰게 된 drawable(`ic_back`, `ic_expand` — 남은 사용처가 없을 때만)

**Interfaces:**
- Consumes: `Tab`, `Dock`, `QuickMediaTheme` (Task 4), `ReceiveScreen` (Task 7), `ListScreen` (Task 5), `SettingsScreen` (Task 6), `jobCounts` (Task 3), `UpdateDialog`, `Updater.check`
- Produces: `@Composable fun AppShell(shared: ReceiveChannel<String?>, launched: Boolean)`

- [ ] **Step 1: `AppShell` 구현** — 스펙 "화면 구조와 이동" 그대로:
  - 상태(`rememberSaveable`): `tab`, `link`, `instant`, `update: Release?`. 켤 때 한 번 업데이트 확인과 공유 채널 받기는 지금 `MainActivity` 의 두 `LaunchedEffect` 를 옮긴다. 공유를 받으면 `tab = RECEIVE`, `link = found`, `instant = true`.
  - 바탕 `bg` 를 꽉 채운 `Box`. 탭 내용 영역에 `hazeSource(hazeState)`, 탭 전환은 `Crossfade`(짧게). 아래에 바탕색으로 서서히 가리는 그라데이션(높이 110dp), 그 위에 `Dock`(시스템 내비게이션 바 위, 아래 여백 18dp). `WindowInsets.isImeVisible` 이면 `Dock` 과 그라데이션을 숨긴다.
  - 각 탭에 넘길 `contentPadding`: 위 = 상태 바 + 8dp, 아래 = 내비게이션 바 + 메뉴 높이 + 32dp(키보드가 떠 있으면 키보드 높이).
  - `BackHandler(enabled = tab != RECEIVE) { tab = RECEIVE }`.
  - 스낵바: 아래 `Dock` 위에 뜨게. `onQueued` → `showSnackbar("목록에 넣었어요", actionLabel = "보기")` 가 `ActionPerformed` 면 `tab = LIST`. 스낵바는 `surface` 바탕, `text` 글자, 동작 글자 `accentText`, 모서리 16dp.
  - 배지 = `jobCounts(jobs)[JobFilter.ACTIVE]`.
  - `update?.let { UpdateDialog(it) { update = null } }`.

- [ ] **Step 2: `MainActivity` 를 줄인다** — `setContent { QuickMediaTheme { AppShell(shared, launched) } }`. 공유 받기(`receiveShare`)와 채널은 그대로.

- [ ] **Step 3: `UpdateDialog` 모양** — 동작·문구는 그대로, 바탕 `surface`·모서리 24dp·주 버튼 `GradientButton` 이나 `accent` 글자, Pretendard 가 Material typography 로 이미 적용되는지만 확인.

- [ ] **Step 4: 옛 화면 지우기와 빌드** — `ui/MainScreen.kt`, `ui/SettingsScreen.kt` 삭제. Run: `ut.sh testDebugUnitTest assembleDebug` Expected: `BUILD SUCCESSFUL`, 전부 통과, `MainScreen`/`changedSummary` 참조 0(`grep -r` 로 확인).

- [ ] **Step 5: 에뮬레이터 확인** — `Pixel_10_Pro_XL`(API 37) 에 디버그 APK 설치. 아래를 하나씩 하고 스크린샷을 워크스페이스 `shots/` 에 남긴다:
  - 다크·라이트(`adb shell cmd uimode night yes|no`) 각각 세 탭.
  - `adb shell am start -a android.intent.action.SEND -t text/plain --es android.intent.extra.TEXT "https://youtu.be/jNQXAC9IVRw"` → 받기 탭, 링크 채움, 뼈대 → 미리보기 카드("Me at the zoo", `0:19`).
  - 링크를 빠르게 바꾸기(붙여넣기 뒤 곧바로 다른 링크 입력) → 카드는 마지막 링크 것만.
  - 다운로드 → 스낵바 `목록에 넣었어요 · 보기` → 보기 → 목록에 썸네일·진행 막대 → 완료 → ▶ 로 열림.
  - `https://www.youtube.com/watch?v=xxxxxxxxxxx`(없는 영상) → 미리보기 실패 카드, 다운로드 → 목록 실패 → ↻ 로 다시 대기열.
  - 받는 중 ✕ → 취소, ↻ 가능.
  - 긴 제목(이모지·한글 많은 영상)이 카드·목록에서 말줄임으로 끝나는지.
  - 목록 끝·설정 끝·다운로드 버튼이 메뉴에 가리지 않는지, 링크 칸을 누르면 메뉴가 숨는지.
  - 화면 돌리기 → 탭·링크 그대로, 죽지 않음. 목록·설정에서 뒤로가기 → 받기.
  - 옵션 시트: 구간 `x` 로 다운로드 → 시트가 열리고 칸에 문구.

- [ ] **Step 6: 커밋** — `안드로이드: 세 탭과 떠 있는 메뉴로 화면 바꾸기`

---

### Task 9: 안드로이드 10 확인 · 릴리스 노트

**Files:**
- Modify: `release-notes/v1.1.0.md`

- [ ] **Step 1: 안드로이드 10 확인** — API 29 x86_64 시스템 이미지(`system-images;android-29;google_apis;x86_64`)로 AVD 를 만들어(없으면 sdkmanager 로 받는다) 디버그 APK 를 깐다. 메뉴가 흐림 없는 반투명으로 보이고, 세 탭과 미리보기·다운로드가 죽지 않는지 확인하고 스크린샷을 남긴다. 확인 뒤 AVD 는 지운다.

- [ ] **Step 2: 릴리스 노트** — `## 새 기능` 의 두 번째 줄 뒤에 한 줄을 더한다: `- 링크를 넣으면 받기 전에 썸네일과 제목을 미리 보여 줘요. 실패한 항목은 목록에서 다시 받을 수 있어요.`

- [ ] **Step 3: APK 크기 확인** — Run: `ut.sh assembleRelease` (서명 환경 변수 없이, unsigned 로) 후 `ls -l app/build/outputs/apk/release/`. Expected: 개편 전 80.3MB 보다 약 3~5MB 큼. 크게 벗어나면(10MB 넘게) 원인을 ledger 에 적는다.

- [ ] **Step 4: 커밋** — `v1.1.0 업데이트 내용에 미리보기·다시 받기 추가`
