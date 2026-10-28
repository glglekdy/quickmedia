# QuickMedia 안드로이드 앱 구현 계획

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 링크·공유로 영상/음성을 받아 `다운로드/QuickMedia` 에 저장하고, GitHub 릴리스의 APK 로 스스로 업데이트하는 안드로이드 앱을 v1.1.0 으로 내보낸다.

**Architecture:** `android/` 의 단일 모듈 Gradle 프로젝트. 순수 Kotlin(옵션→인자, 오류 문구, 링크 뽑기, 버전 비교, 이름 번호, 작업 상태)은 JVM 단위 테스트로 검사하고, 안드로이드 쪽(포그라운드 서비스, MediaStore, PackageInstaller, Compose 화면)은 얇게 두고 에뮬레이터에서 확인한다. 다운로드는 youtubedl-android 가 앱 캐시에 받고 MediaStore 로 옮긴다.

**Tech Stack:** Kotlin 2.4.21, AGP 9.5.0, Gradle 9.8.2, Compose BOM 2026.09.00 (Material 3), activity-compose 1.13.0, lifecycle 2.11.0, `io.github.junkfood02.youtubedl-android:{library,ffmpeg}:0.18.1`, JUnit 4.13.2, `org.json:json:20260814`(테스트 전용).

**Spec:** `docs/superpowers/specs/2026-10-29-quickmedia-android-design.md`

## Global Constraints

- 패키지 `com.glglekdy.quickmedia`, 앱 이름 `QuickMedia`, minSdk 29, targetSdk/compileSdk 36.
- 릴리스 APK ABI `arm64-v8a` 만. 디버그는 `arm64-v8a` + `x86_64`(로컬 에뮬레이터용).
- 매니페스트 `android:extractNativeLibs="true"`, `packaging.jniLibs.useLegacyPackaging = true` (youtubedl-android 요구).
- `versionName` = `-PappVersion` 값(없으면 `0.0.0-dev`), `versionCode` = 주*10000 + 부*100 + 수(없으면 1). 1.1.0 → 10100.
- 저장 위치 `Download/QuickMedia/`, 같은 이름이면 `제목 (1).확장자`.
- 릴리스 저장소 `glglekdy/quickmedia`, APK 자산 이름 `QuickMedia-<버전>.apk`.
- 화면 문구는 한국어, 데스크톱 문구와 같은 말투("~해요"). 문구는 스펙과 이 계획에 적힌 그대로 쓴다.
- 로컬 빌드: `JAVA_HOME="/c/Program Files/Android/Android Studio/jbr"`, `ANDROID_HOME="$LOCALAPPDATA/Android/Sdk"`. 명령은 `android/` 에서 `./gradlew …`.
- 키스토어·비밀번호는 커밋하지 않는다.

## Review Focus

1. **긴 한국어 제목** — 255바이트 파일 이름 한도(한글 3바이트)를 넘으면 yt-dlp 가 "File name too long" 으로 실패한다. 인자에 `--trim-filenames 80` 을 늘 넣는다 (Task 2 테스트로 고정).
2. **재생목록 결과가 여러 파일** — 캐시 폴더의 파일을 모두 옮기고, 카드를 누르면 폴더를 연다. 부분 파일(`.part`, `.ytdl`, `.temp`)은 옮기지 않는다 (Task 5 `resultFiles` 테스트).
3. **앱이 받는 중에 강제 종료됨** — 다음 실행 때 `cache/dl/` 에 남은 폴더를 지운다(서비스가 돌고 있지 않을 때만) (Task 6).
4. **공유 글에 링크가 없거나 여러 개, `youtu.be/...?si=` 처럼 꼬리가 붙음** — 첫 `http(s)` 링크를 공백·닫는 괄호 전까지 그대로 쓴다 (Task 3 테스트).
5. **업데이트 받다 끊김 / 해시 불일치** — 부분 파일을 지우고 문구를 보인다. 시작할 때의 확인 실패는 조용히 넘긴다 (Task 4 `verify` 테스트, Task 8 수동 확인).

---

### Task 1: 프로젝트 뼈대와 버전 번호

**Files:**
- Create: `android/settings.gradle.kts`, `android/build.gradle.kts`, `android/gradle.properties`, `android/gradle/libs.versions.toml`, `android/gradlew`, `android/gradlew.bat`, `android/gradle/wrapper/*`
- Create: `android/app/build.gradle.kts`, `android/app/src/main/AndroidManifest.xml`, `android/app/src/main/res/values/strings.xml`, `android/app/src/main/res/mipmap-*/ic_launcher*.png` (데스크톱 `ytdl/assets/icon.png` 에서 만든다)
- Create: `android/app/src/main/java/com/glglekdy/quickmedia/App.kt`, `MainActivity.kt`
- Modify: `.gitignore` (`android/.gradle/`, `android/build/`, `android/app/build/`, `android/local.properties`, `*.jks`, `*.keystore`)

**Interfaces:**
- Produces: `App : Application` — `onCreate` 에서 `YoutubeDL.getInstance().init(this)`, `FFmpeg.getInstance().init(this)`; 실패하면 `App.initError: String?` 에 남긴다(화면이 띄운다). `MainActivity : ComponentActivity` (Task 7 이 내용을 채운다). `BuildConfig.VERSION_NAME`.

- [ ] **Step 1: 래퍼 만들기** — `https://services.gradle.org/distributions/gradle-9.8.2-bin.zip` 를 스크래치 폴더에 풀고 `android/` 에서 `gradle wrapper --gradle-version 9.8.2` 를 실행한다.
- [ ] **Step 2: Gradle 파일 작성** — 버전 카탈로그에 Tech Stack 의 버전을 넣는다. AGP 9 는 Kotlin 을 내장하므로 `org.jetbrains.kotlin.android` 플러그인은 쓰지 않고 `org.jetbrains.kotlin.plugin.compose` 만 붙인다(AGP 가 거부하면 그 오류 문구대로 맞춘다). `app/build.gradle.kts` 에 Global Constraints 의 SDK·ABI·packaging·버전 규칙과 `buildFeatures { compose = true; buildConfig = true }`. 버전 계산:

```kotlin
val appVersion = (findProperty("appVersion") as String?) ?: "0.0.0-dev"
val versionParts = Regex("""^(\d+)\.(\d+)\.(\d+)""").find(appVersion)?.destructured
val appVersionCode = versionParts?.let { (a, b, c) -> a.toInt() * 10000 + b.toInt() * 100 + c.toInt() } ?: 1
```

릴리스 서명은 Task 9 에서 붙인다.
- [ ] **Step 3: 매니페스트·App·MainActivity** — 매니페스트에 `INTERNET`, `extractNativeLibs`, `android:name=".App"`, 런처 액티비티. MainActivity 는 우선 `Text("QuickMedia")` 만.
- [ ] **Step 4: 빌드 확인**

Run: `./gradlew assembleDebug -PappVersion=1.1.0` 후 `"$ANDROID_HOME/build-tools/37.0.0/aapt2" dump badging app/build/outputs/apk/debug/app-debug.apk | head -1`
Expected: `BUILD SUCCESSFUL`, `package: name='com.glglekdy.quickmedia' versionCode='10100' versionName='1.1.0'`

- [ ] **Step 5: 에뮬레이터에서 켜지는지** — `emulator -avd Pixel_10_Pro_XL` 을 띄우고 `adb install -r app-debug.apk`, `adb shell am start -n com.glglekdy.quickmedia/.MainActivity`. 화면에 QuickMedia 가 보이고 `adb logcat -d | grep -i youtubedl` 에 초기화 오류가 없어야 한다.
- [ ] **Step 6: Commit** — `안드로이드 프로젝트 뼈대`

### Task 2: 옵션과 yt-dlp 인자

**Files:**
- Create: `android/app/src/main/java/com/glglekdy/quickmedia/options/DownloadOptions.kt`, `options/YtdlpArgs.kt`
- Test: `android/app/src/test/java/com/glglekdy/quickmedia/options/YtdlpArgsTest.kt`

**Interfaces:**
- Produces:
  - `enum class Kind { VIDEO, AUDIO }`, `enum class PlaylistMode { SINGLE, ALL, ITEMS }`
  - `data class DownloadOptions(kind: Kind = VIDEO, quality: String = "best" /* best|1080|720|480 */, audioFormat: String = "mp3" /* mp3|m4a */, playlist: PlaylistMode = SINGLE, playlistItems: String = "", embedMeta: Boolean = false, sectionStart: String = "", sectionEnd: String = "")`
  - `object YtdlpArgs { fun build(o: DownloadOptions, outDir: String): List<String>; fun validate(o: DownloadOptions): Map<String, String>; fun parseTime(text: String): Double? }`
  - `validate` 키: `"playlistItems"`, `"sectionStart"`, `"sectionEnd"`. 문구: `"1-5,8 처럼 번호와 범위로 적어 주세요"`, `"1:23 이나 83 처럼 적어 주세요"`, `"끝이 시작보다 뒤여야 해요"`(키 `sectionEnd`).

- [ ] **Step 1: 실패하는 테스트 작성**

```kotlin
private fun args(o: DownloadOptions) = YtdlpArgs.build(o, "/c/dl")

@Test fun alwaysArgs() {
    val a = args(DownloadOptions())
    assertTrue(a.containsAll(listOf("--no-mtime", "--trim-filenames", "80")))
    assertEquals(listOf("-P", "/c/dl", "-o", "%(title)s.%(ext)s"), a.windowAfter("-P", 4))
}
@Test fun videoBest() { val a = args(DownloadOptions()); assertEquals("ext:mp4:m4a", a.after("-S")); assertEquals("mp4", a.after("--merge-output-format")) }
@Test fun video720() = assertEquals("res:720,ext:mp4:m4a", args(DownloadOptions(quality = "720")).after("-S"))
@Test fun audio() {
    val a = args(DownloadOptions(kind = Kind.AUDIO, audioFormat = "m4a"))
    assertTrue(a.containsAll(listOf("-x"))); assertEquals("m4a", a.after("--audio-format")); assertEquals("0", a.after("--audio-quality"))
    assertFalse("-S" in a)
}
@Test fun playlistModes() {
    assertTrue("--no-playlist" in args(DownloadOptions()))
    assertTrue("--yes-playlist" in args(DownloadOptions(playlist = PlaylistMode.ALL)))
    val items = args(DownloadOptions(playlist = PlaylistMode.ITEMS, playlistItems = "1-5, 8"))
    assertTrue("--yes-playlist" in items); assertEquals("1-5,8", items.after("-I"))
}
@Test fun embedMeta() = assertTrue(args(DownloadOptions(embedMeta = true)).containsAll(listOf("--embed-thumbnail", "--embed-metadata")))
@Test fun section() {
    assertEquals("*1:23-inf", args(DownloadOptions(sectionStart = "1:23")).after("--download-sections"))
    assertEquals("*0-90", args(DownloadOptions(sectionEnd = "90")).after("--download-sections"))
    assertTrue("--force-keyframes-at-cuts" in args(DownloadOptions(sectionEnd = "90")))
    assertFalse("--download-sections" in args(DownloadOptions()))
}
@Test fun parseTime() {
    assertEquals(83.0, YtdlpArgs.parseTime("1:23")); assertEquals(3723.0, YtdlpArgs.parseTime("1:02:03"))
    assertEquals(83.0, YtdlpArgs.parseTime(" 83 ")); assertNull(YtdlpArgs.parseTime("1:2:3:4")); assertNull(YtdlpArgs.parseTime("abc"))
}
@Test fun validate() {
    assertEquals(emptyMap<String, String>(), YtdlpArgs.validate(DownloadOptions()))
    assertTrue("playlistItems" in YtdlpArgs.validate(DownloadOptions(playlist = PlaylistMode.ITEMS, playlistItems = "a-b")))
    assertTrue("playlistItems" in YtdlpArgs.validate(DownloadOptions(playlist = PlaylistMode.ITEMS, playlistItems = "")))
    assertEquals(emptyMap<String, String>(), YtdlpArgs.validate(DownloadOptions(playlist = PlaylistMode.ITEMS, playlistItems = "1-5, 8")))
    assertTrue("sectionStart" in YtdlpArgs.validate(DownloadOptions(sectionStart = "x")))
    assertEquals("끝이 시작보다 뒤여야 해요", YtdlpArgs.validate(DownloadOptions(sectionStart = "2:00", sectionEnd = "1:00"))["sectionEnd"])
}
// 테스트 파일 안의 도우미: List<String>.after(flag) = this[indexOf(flag) + 1], windowAfter(flag, n) = subList(indexOf(flag), indexOf(flag) + n)
```

- [ ] **Step 2: 실패 확인** — Run: `./gradlew testDebugUnitTest --tests '*YtdlpArgsTest'` Expected: 컴파일 실패(`YtdlpArgs` 없음).
- [ ] **Step 3: 구현** — 인자 표는 스펙의 "옵션 → yt-dlp 인자" 그대로. 인자 순서: `-P dir -o tmpl --no-mtime --trim-filenames 80`, 재생목록, 형식, 메타, 구간. 일부 항목 검사는 `^\d+(-\d+)?(,\d+(-\d+)?)*$` (공백 제거 후). `parseTime` 은 `:` 로 나눈 1~3개 숫자 조각.
- [ ] **Step 4: 통과 확인** — 같은 명령, Expected: PASS
- [ ] **Step 5: Commit** — `안드로이드: 옵션을 yt-dlp 인자로`

### Task 3: 오류 문구와 공유 링크

**Files:**
- Create: `.../download/FriendlyError.kt`, `.../download/SharedLink.kt`
- Test: `.../download/FriendlyErrorTest.kt`, `.../download/SharedLinkTest.kt`

**Interfaces:**
- Produces: `fun friendlyError(message: String): String`, `fun extractLink(text: String?): String?`

- [ ] **Step 1: 실패하는 테스트 작성** — 데스크톱 `tests/test_downloader.py::test_friendly_error` 의 표를 옮기되 쿠키 두 줄은 빼고 로그인 문구만 바꾼다:

```kotlin
@Test fun table() = mapOf(
    "ERROR: Unsupported URL: https://x" to "지원하지 않는 링크예요",
    "ERROR: Sign in to confirm your age" to "로그인이 필요한 영상이라 받을 수 없어요",
    "ERROR: Video unavailable" to "삭제되었거나 볼 수 없는 영상이에요",
    "ERROR: Requested format is not available" to "고른 화질·형식이 없는 영상이에요. 옵션을 바꿔 보세요",
    "ERROR: [Errno 7] No address associated with hostname" to "인터넷 연결을 확인해 주세요",
    "ERROR: something odd" to "something odd",
).forEach { (m, want) -> assertEquals(want, friendlyError(m)) }
@Test fun longMessageCut() = assertEquals(160, friendlyError("ERROR: " + "a".repeat(300)).length)
@Test fun links() {
    assertEquals("https://youtu.be/abc?si=x1", extractLink("이 영상 봐 https://youtu.be/abc?si=x1 재밌음"))
    assertEquals("https://a.com/1", extractLink("(https://a.com/1) and https://b.com/2"))
    assertNull(extractLink("링크 없음")); assertNull(extractLink(null))
}
```

연결 오류 단어는 데스크톱 목록에 안드로이드 문구를 더한다: `getaddrinfo`, `timed out`, `unable to connect`, `no address associated`, `network is unreachable`. 오류는 여러 줄일 수 있으니 `ERROR:` 로 시작하는 마지막 줄을 기준으로 한다.

- [ ] **Step 2: 실패 확인** — `./gradlew testDebugUnitTest --tests '*FriendlyErrorTest' --tests '*SharedLinkTest'` → 컴파일 실패.
- [ ] **Step 3: 구현** — 링크 정규식 `https?://[^\s)\]>"']+`.
- [ ] **Step 4: 통과 확인**
- [ ] **Step 5: Commit** — `안드로이드: 오류 문구와 공유 링크 뽑기`

### Task 4: 릴리스 고르기와 버전 비교

**Files:**
- Create: `.../update/Releases.kt`
- Test: `.../update/ReleasesTest.kt`
- Modify: `android/app/build.gradle.kts` (`testImplementation(libs.json)` — 안드로이드 `org.json` 은 JVM 테스트에서 비어 있다)

**Interfaces:**
- Produces:
  - `fun parseVersion(text: String): List<Int>` — 데스크톱 `parse_version` 과 같다(`v` 무시, 숫자 부분만, 끝의 0 제거).
  - `fun isNewer(latest: String, current: String): Boolean`
  - `data class Release(val version: String, val notes: String, val apkUrl: String, val apkSize: Long, val sha256: String /* 소문자 hex, 없으면 "" */)`
  - `fun parseRelease(json: String, current: String): Release?` — 더 새 버전이고 `^QuickMedia-.+\.apk$` 자산이 있을 때만.
  - `fun verify(size: Long, sha256Hex: String, release: Release): String?` — 맞으면 null, 아니면 문구 `"받은 파일 크기가 맞지 않아요. 다시 시도해 주세요."` / `"받은 파일이 손상되었어요. (SHA-256 불일치)"`. 기대값이 비어 있으면 그 검사는 건너뛴다.
  - `const val LATEST_URL = "https://api.github.com/repos/glglekdy/quickmedia/releases/latest"`

- [ ] **Step 1: 실패하는 테스트 작성**

```kotlin
@Test fun versions() {
    assertEquals(listOf(1, 1), parseVersion("v1.1.0")); assertEquals(listOf(1, 2), parseVersion("1.2.0-beta"))
    assertTrue(isNewer("1.1.0", "1.0.1")); assertFalse(isNewer("v1.1", "1.1.0")); assertFalse(isNewer("1.0.9", "1.1.0"))
    assertTrue(isNewer("1.1.0", "0.0.0-dev"))
}
private val json = """{"tag_name":"v1.1.0","body":"notes","assets":[
  {"name":"QuickMedia-Setup-1.1.0.exe","browser_download_url":"https://x/exe","size":1,"digest":"sha256:aa"},
  {"name":"QuickMedia-1.1.0.apk","browser_download_url":"https://x/apk","size":42,"digest":"sha256:ABCD"}]}"""
@Test fun picksApk() = assertEquals(Release("1.1.0", "notes", "https://x/apk", 42, "abcd"), parseRelease(json, "1.0.1"))
@Test fun notNewer() = assertNull(parseRelease(json, "1.1.0"))
@Test fun noApk() = assertNull(parseRelease(json.replace("QuickMedia-1.1.0.apk", "other.zip"), "1.0.0"))
@Test fun verify() {
    val r = Release("1.1.0", "", "u", 3, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad") // "abc"
    assertNull(verify(3, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", r))
    assertEquals("받은 파일 크기가 맞지 않아요. 다시 시도해 주세요.", verify(4, "x", r))
    assertEquals("받은 파일이 손상되었어요. (SHA-256 불일치)", verify(3, "00", r))
}
```

- [ ] **Step 2: 실패 확인** — `./gradlew testDebugUnitTest --tests '*ReleasesTest'`
- [ ] **Step 3: 구현** (`org.json.JSONObject` 사용)
- [ ] **Step 4: 통과 확인**
- [ ] **Step 5: Commit** — `안드로이드: 릴리스에서 APK 고르기와 버전 비교`

### Task 5: 결과 파일 저장 (MediaStore)

**Files:**
- Create: `.../download/MediaSaver.kt`
- Test: `.../download/MediaSaverTest.kt`

**Interfaces:**
- Produces:
  - `fun numberedName(name: String, taken: (String) -> Boolean): String` — `"Me.mp4"` → `"Me (1).mp4"` → `"Me (2).mp4"`… 확장자 없는 이름은 끝에 붙인다.
  - `fun mimeOf(name: String): String` — mp4 `video/mp4`, mkv `video/x-matroska`, webm `video/webm`, mp3 `audio/mpeg`, m4a `audio/mp4`, opus `audio/ogg`, jpg `image/jpeg`, 그 밖 `application/octet-stream`.
  - `fun resultFiles(dir: File): List<File>` — 일반 파일 중 `.part`, `.ytdl`, `.temp`, `.part-Frag*` 를 뺀 것, 이름순.
  - `class MediaSaver(context: Context) { fun save(file: File): Uri }` — `MediaStore.Downloads.EXTERNAL_CONTENT_URI` 에 `RELATIVE_PATH = "Download/QuickMedia/"`, `IS_PENDING` 1 → 복사 → 0. 이름 충돌은 같은 RELATIVE_PATH 에 같은 DISPLAY_NAME 이 있는지 쿼리하는 `taken` 으로 `numberedName` 을 부른다(MediaStore 의 자동 이름 바꾸기에 기대지 않는다).
  - `const val SAVE_DIR = "Download/QuickMedia"`

- [ ] **Step 1: 실패하는 테스트** — `numberedName("Me.mp4") { it in setOf("Me.mp4", "Me (1).mp4") } == "Me (2).mp4"`, 빈 집합이면 그대로, `"a.b.mp3"` → `"a.b (1).mp3"`, `"noext"` → `"noext (1)"`; `mimeOf("x.MP3") == "audio/mpeg"`; 임시 폴더에 `a.mp4`, `a.f137.mp4.part`, `b.mp3`, `c.ytdl` 을 만들고 `resultFiles` 가 `[a.mp4, b.mp3]`.
- [ ] **Step 2: 실패 확인** — `./gradlew testDebugUnitTest --tests '*MediaSaverTest'`
- [ ] **Step 3: 구현**
- [ ] **Step 4: 통과 확인**
- [ ] **Step 5: Commit** — `안드로이드: 결과 파일을 다운로드 폴더에 저장`

### Task 6: 작업 목록과 다운로드 서비스

**Files:**
- Create: `.../download/Jobs.kt`, `.../download/DownloadService.kt`
- Test: `.../download/JobsTest.kt`
- Modify: `AndroidManifest.xml` (`FOREGROUND_SERVICE`, `FOREGROUND_SERVICE_DATA_SYNC`, `POST_NOTIFICATIONS`, `<service android:name=".download.DownloadService" android:foregroundServiceType="dataSync" android:exported="false"/>`), `App.kt` (남은 `cache/dl` 정리)

**Interfaces:**
- Consumes: `YtdlpArgs.build`, `friendlyError`, `MediaSaver`, `resultFiles`.
- Produces:
  - `enum class JobState { QUEUED, RUNNING, DONE, FAILED, CANCELLED }`
  - `data class Job(val id: Long, val url: String, val options: DownloadOptions, val title: String = url, val state: JobState = QUEUED, val percent: Float = 0f, val status: String = "대기 중", val saved: List<Uri> = emptyList())` — `Uri` 를 피하려면 `saved: List<String>`(uri 문자열)로 둔다 → **`List<String>`** 로 정한다.
  - `object Jobs { val list: StateFlow<List<Job>>; fun add(url: String, o: DownloadOptions): Job; fun update(id: Long, f: (Job) -> Job); fun nextQueued(): Job?; fun cancel(id: Long): Boolean /* QUEUED·RUNNING 일 때만 CANCELLED 로 바꾸고 true */; fun clearFinished(); fun reset() /* 테스트용 */ }` — 새 작업은 목록 **맨 위**에.
  - `DownloadService` — `companion fun start(context)`, `companion fun cancel(context, id: Long)`. 대기열을 하나씩: 상태 문구 `"영상 정보 불러오는 중…"` → `"받는 중 n%"` → `"파일 정리 중…"` → `"완료"` / 실패 문구 / `"취소됨"`. 제목은 yt-dlp 출력 줄 `[download] Destination: .../<이름>` 에서 확장자 앞까지 뽑아 바꾼다. processId = 작업 id 문자열. 끝나면 `stopSelf()`. 알림 채널 `downloads`, 제목 "QuickMedia", 본문 "<제목> · n%".

- [ ] **Step 1: 실패하는 테스트** (`@Before Jobs.reset()`)

```kotlin
@Test fun newestFirstAndQueueOrder() {
    val a = Jobs.add("u1", DownloadOptions()); val b = Jobs.add("u2", DownloadOptions())
    assertEquals(listOf(b.id, a.id), Jobs.list.value.map { it.id })
    assertEquals(a.id, Jobs.nextQueued()?.id)          // 먼저 넣은 것부터 받는다
}
@Test fun cancelOnlyUnfinished() {
    val a = Jobs.add("u", DownloadOptions())
    Jobs.update(a.id) { it.copy(state = JobState.DONE) }
    assertFalse(Jobs.cancel(a.id))
    val b = Jobs.add("u", DownloadOptions()); assertTrue(Jobs.cancel(b.id)); assertNull(Jobs.nextQueued())
}
@Test fun clearFinishedKeepsActive() {
    val a = Jobs.add("u", DownloadOptions()); val b = Jobs.add("u", DownloadOptions())
    Jobs.update(a.id) { it.copy(state = JobState.FAILED) }; Jobs.clearFinished()
    assertEquals(listOf(b.id), Jobs.list.value.map { it.id })
}
@Test fun titleFromDestination() = assertEquals("Me at the zoo",
    titleFromLine("[download] Destination: /data/user/0/x/cache/dl/3/Me at the zoo.f137.mp4"))
```

`titleFromLine(line: String): String?` 도 `Jobs.kt` 에 둔다(포맷 꼬리 `.f137` 과 확장자를 뗀다, Destination 줄이 아니면 null).

- [ ] **Step 2: 실패 확인** — `./gradlew testDebugUnitTest --tests '*JobsTest'`
- [ ] **Step 3: Jobs 구현**, 테스트 통과 확인
- [ ] **Step 4: DownloadService 구현** — 코루틴 하나(`Dispatchers.IO`)로 `while (nextQueued() != null)`. 작업 폴더 `cacheDir/dl/<id>/`. `YoutubeDLRequest(url)` 에 `YtdlpArgs.build` 인자를 순서대로 `addOption`(값 있는 옵션은 두 개씩). 콜백 `(progress: Float, eta: Long, line: String)`. 성공하면 `resultFiles` 를 모두 `MediaSaver.save` → `saved`. 결과 파일이 없으면 실패 `"받은 파일이 없어요"`. 취소는 `YoutubeDL.getInstance().destroyProcessById(id)`; 취소된 작업에서 나는 예외는 실패로 바꾸지 않는다. 어떤 결과든 작업 폴더를 지운다.
- [ ] **Step 5: Commit** — `안드로이드: 다운로드 서비스와 작업 목록`

### Task 7: 화면, 공유 받기, 옵션 기억

**Files:**
- Create: `.../ui/MainScreen.kt`, `.../ui/SettingsScreen.kt`, `.../ui/Theme.kt`, `.../options/OptionsStore.kt`
- Modify: `MainActivity.kt`, `AndroidManifest.xml` (MainActivity 에 `ACTION_SEND` `text/plain` intent-filter, `launchMode="singleTop"`)

**Interfaces:**
- Consumes: `Jobs`, `DownloadService.start/cancel`, `YtdlpArgs.validate`, `extractLink`, `SAVE_DIR`, `App.initError`.
- Produces: `class OptionsStore(context) { fun load(): DownloadOptions; fun save(o: DownloadOptions) }` (SharedPreferences `options`, 키는 필드 이름). `@Composable fun MainScreen(onOpenSettings: () -> Unit)`, `@Composable fun SettingsScreen(onBack: () -> Unit)` (Task 8 이 버튼 동작을 채운다).

- [ ] **Step 1: MainScreen** — 스펙 "메인 화면" 1~6 그대로. 입력 오류는 `validate` 키에 맞는 칸 아래 빨간 글씨. 링크가 비면 `"먼저 링크를 넣어 주세요"`. 다운로드 누르면 옵션 저장 → `Jobs.add` → `DownloadService.start` → 링크 칸 비우기. 안드로이드 13+ 에서 첫 다운로드 때 `POST_NOTIFICATIONS` 요청(거절해도 진행). 카드: 제목, 상태, `LinearProgressIndicator`(RUNNING 일 때), 취소(QUEUED·RUNNING), 완료 카드를 누르면 `saved` 가 하나면 `ACTION_VIEW`(mime 포함), 여럿이면 `DownloadManager.ACTION_VIEW_DOWNLOADS`. 목록 위에 "끝난 작업 지우기".
- [ ] **Step 2: 공유 받기** — `onCreate`/`onNewIntent` 에서 `ACTION_SEND` 의 `EXTRA_TEXT` 를 `extractLink` → 링크 칸. 없으면 스낵바 `"링크를 찾지 못했어요"`.
- [ ] **Step 3: SettingsScreen 틀** — 앱 버전(`BuildConfig.VERSION_NAME`), `업데이트 확인` 버튼, yt-dlp 버전, `yt-dlp 갱신` 버튼(동작은 Task 8), `"저장 위치: 다운로드/QuickMedia"`.
- [ ] **Step 4: 에뮬레이터 확인** — `./gradlew installDebug` 후:
  - "Me at the zoo"(`https://www.youtube.com/watch?v=jNQXAC9IVRw`) 영상 받기 → 완료, `adb shell ls /sdcard/Download/QuickMedia/` 에 `Me at the zoo.mp4`. 같은 걸 또 받기 → `Me at the zoo (1).mp4`.
  - 음성 mp3 → `Me at the zoo.mp3`, 썸네일·메타 켜고 받기 성공.
  - 구간 `0:05`~`0:10` → 약 5초 파일.
  - `adb shell am start -a android.intent.action.SEND -t text/plain --es android.intent.extra.TEXT "봐 https://youtu.be/jNQXAC9IVRw?si=a" -n com.glglekdy.quickmedia/.MainActivity` → 링크 칸이 채워짐.
  - 받는 중 홈으로 나가도 알림 진행률이 올라가고 끝난다. 취소 → "취소됨", 캐시 폴더 비어 있음(`adb shell run-as com.glglekdy.quickmedia ls cache/dl`).
  - 앱을 껐다 켜도 고른 옵션이 남아 있다.
- [ ] **Step 5: Commit** — `안드로이드: 메인·설정 화면과 공유 받기`

### Task 8: 앱 업데이트와 yt-dlp 갱신

**Files:**
- Create: `.../update/Updater.kt`, `.../ui/UpdateDialog.kt`
- Modify: `AndroidManifest.xml` (`REQUEST_INSTALL_PACKAGES`, 설치 결과를 받을 `<receiver android:name=".update.InstallResultReceiver" android:exported="false"/>`), `MainActivity.kt`, `SettingsScreen.kt`

**Interfaces:**
- Consumes: `LATEST_URL`, `parseRelease`, `verify`, `Release`.
- Produces:
  - `object Updater { suspend fun check(): Release? /* 실패하면 UpdateException(문구) */; suspend fun download(context, r: Release, progress: (Long, Long) -> Unit): File; fun install(context, apk: File) }`
  - `class UpdateException(message: String) : Exception(message)` — 문구 `"GitHub 에 연결하지 못했어요."`, `"릴리즈 정보를 가져오지 못했어요. (HTTP <코드>)"`, `"업데이트 파일을 받다가 끊겼어요. 다시 시도해 주세요."`, 그리고 `verify` 문구.
  - `@Composable fun UpdateDialog(release: Release, onDismiss: () -> Unit)`

- [ ] **Step 1: Updater 구현** — `HttpURLConnection`, 헤더 `User-Agent: QuickMedia/<버전>`, `Accept: application/vnd.github+json`, 시간 제한 20초. 404 는 `null`(릴리스 없음). 받기는 `cacheDir/update/QuickMedia-<버전>.apk.part` 에 쓰며 SHA-256 을 같이 계산 → `verify` → 이름 바꾸기, 실패하면 `.part` 지움. 설치는 `PackageInstaller.createSession` → APK 를 쓰고 → `commit(PendingIntent(InstallResultReceiver))`. 리시버는 `STATUS_PENDING_USER_ACTION` 이면 `EXTRA_INTENT` 를 띄운다. 그 전에 `packageManager.canRequestPackageInstalls()` 가 false 면 `Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES`(`package:` uri)로 보내고, 화면으로 돌아오면(`ON_RESUME`) 설치를 이어 간다.
- [ ] **Step 2: 시작할 때 확인** — MainActivity 가 처음 켜질 때 한 번 `check()`; 새 버전이면 `UpdateDialog`, 실패는 조용히. 대화상자: 제목 `"새 버전 <버전>"`, 본문 릴리스 설명(스크롤), 버튼 `지금 업데이트` / `나중에`, 받는 동안 진행 막대, 실패 문구는 대화상자 안에.
- [ ] **Step 3: 설정 버튼** — `업데이트 확인`: 새 버전이면 같은 대화상자, 없으면 `"최신 버전이에요"`, 실패면 그 문구. `yt-dlp 갱신`: `YoutubeDL.getInstance().updateYoutubeDL(context, YoutubeDL.UpdateChannel._STABLE)`(0.18.1 의 실제 이름을 확인해 쓴다) → `ALREADY_UP_TO_DATE` 면 `"최신 버전이에요"`, `DONE` 이면 `"<버전> 으로 갱신했어요"`, 예외면 `"yt-dlp 를 갱신하지 못했어요"`. 버전 표시는 `YoutubeDL.getInstance().version(context)`.
- [ ] **Step 4: 에뮬레이터 확인** — 아직 APK 릴리스가 없으므로: 설정에서 `업데이트 확인` → `"최신 버전이에요"`(v1.0.1 에 APK 자산이 없음) 확인, `yt-dlp 갱신` 이 문구를 띄우고 버전이 바뀌는지 확인. 설치 흐름 자체는 Task 10 에서 실제 릴리스로 확인한다.
- [ ] **Step 5: Commit** — `안드로이드: 앱 업데이트와 yt-dlp 갱신`

### Task 9: 서명과 릴리스 워크플로

**Files:**
- Modify: `android/app/build.gradle.kts` (release `signingConfig` — 환경 변수 `ANDROID_KEYSTORE_FILE`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD` 가 있으면 사용, 없으면 서명 없이), `.github/workflows/release.yml`, `README.md`

**Interfaces:**
- Produces: 릴리스에 `QuickMedia-<버전>.apk`.

- [ ] **Step 1: 키스토어 만들기 (사용자와 함께)** — `keytool -genkeypair -v -keystore quickmedia-release.jks -alias quickmedia -keyalg RSA -keysize 4096 -validity 36500` (Android Studio jbr 의 keytool). 비밀번호는 사용자가 정해 직접 입력하도록 `! <명령>` 으로 안내한다. 파일은 저장소 밖(예: `C:\Users\kimji\Documents\keys\`)에 두고 백업을 당부한다.
- [ ] **Step 2: Secrets 등록 (사용자 확인 후)** — `gh secret set ANDROID_KEYSTORE_BASE64 < <(base64 -w0 quickmedia-release.jks)` 와 나머지 셋. 비밀번호는 사용자가 직접 입력.
- [ ] **Step 3: 로컬 릴리스 빌드 확인** — 환경 변수를 넣고 `./gradlew testDebugUnitTest assembleRelease -PappVersion=1.0.99` → `apksigner verify --print-certs app/build/outputs/apk/release/app-release.apk` 에 인증서가 나오고, `aapt2 dump badging` 의 `native-code` 가 `'arm64-v8a'` 하나.
- [ ] **Step 4: 워크플로** — `android` 작업 추가:

```yaml
  android:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-java@v4
        with: { distribution: temurin, java-version: "21" }
      - uses: gradle/actions/setup-gradle@v4
      - name: 키스토어
        run: echo "${{ secrets.ANDROID_KEYSTORE_BASE64 }}" | base64 -d > $RUNNER_TEMP/release.jks
      - name: 시험과 빌드
        working-directory: android
        env:
          ANDROID_KEYSTORE_FILE: ${{ runner.temp }}/release.jks
          ANDROID_KEYSTORE_PASSWORD: ${{ secrets.ANDROID_KEYSTORE_PASSWORD }}
          ANDROID_KEY_ALIAS: ${{ secrets.ANDROID_KEY_ALIAS }}
          ANDROID_KEY_PASSWORD: ${{ secrets.ANDROID_KEY_PASSWORD }}
        run: |
          v="${GITHUB_REF_NAME#v}"
          ./gradlew testDebugUnitTest assembleRelease -PappVersion="$v"
          mv app/build/outputs/apk/release/app-release.apk "QuickMedia-$v.apk"
      - uses: softprops/action-gh-release@v2
        with:
          files: android/QuickMedia-*.apk
          # 설명은 build 작업이 release-notes/<태그>.md 로 이미 채웠다. 여기서는 파일만 더한다.
```

파일 맨 위 주석에 APK 도 함께 올라간다고 한 줄 더한다. (스펙은 JDK 17 이라 했으나 AGP 9.5 는 JDK 17 이상이면 되고 로컬이 21 이므로 21 로 맞춘다.)
- [ ] **Step 5: README** — 스펙 "문서" 절 내용으로 "안드로이드" 절.
- [ ] **Step 6: Commit** — `안드로이드: 릴리스 서명과 APK 배포`

### Task 10: v1.1.0 릴리스와 업데이트 확인

- [ ] **Step 1: 전체 확인** — 데스크톱 `python -m pytest -q` 통과, `./gradlew testDebugUnitTest assembleDebug` 통과, Task 7 Step 4 목록을 한 번 더 훑는다.
- [ ] **Step 2: 업데이트 시험 준비** — 릴리스 키로 서명한 `-PappVersion=1.0.99` APK 를 에뮬레이터에 설치(디버그 앱은 먼저 지운다). 에뮬레이터는 x86_64 라 arm64 전용 릴리스 APK 가 설치되지 않으면, 이 시험용 빌드에만 `-PextraAbi=x86_64` 로 x86_64 를 더한다(`app/build.gradle.kts` 에서 속성이 있을 때만 abiFilters 에 추가).
- [ ] **Step 3: 업데이트 내용 쓰기** — `release-notes/v1.1.0.md` 를 직접 쓴다. 자동 생성("Full Changelog", 커밋 목록) 금지. 사용자 눈높이의 한국어로 `## 새 기능` / `## 고친 점` 아래 짧은 항목들: 안드로이드 앱 첫 버전(받는 곳, 할 수 있는 것, 64비트 안드로이드 10 이상, 처음 설치 방법은 README 안내), 그 밖에 v1.0.1 이후 데스크톱 변경이 있으면 그것. 앱 업데이트 창에 그대로 보이므로 짧게. 커밋.
- [ ] **Step 4: 병합과 태그 (사용자 확인 후)** — `android-app` 을 `main` 에 병합해 push, `git tag v1.1.0 && git push origin v1.1.0`. `gh run watch` 로 두 작업 성공, `gh release view v1.1.0` 에 `QuickMedia-Setup-1.1.0.exe` 와 `QuickMedia-1.1.0.apk`.
- [ ] **Step 5: 실제 업데이트** — 에뮬레이터의 1.0.99 앱을 켜면 "새 버전 1.1.0" → 지금 업데이트 → 출처 허용 → 설치 → 설정 화면 버전 1.1.0. (x86_64 에뮬레이터에서는 arm64 APK 가 "호환되지 않음"으로 실패할 수 있다. 그 경우 받기·검증·설치 요청까지 확인하고, 실제 덮어 설치는 사용자 폰에서 확인해 달라고 한다.)
- [ ] **Step 6: 데스크톱 업데이트 영향 없음** — 데스크톱 1.0.1 이 1.1.0 설치 파일을 고르는지 `python -c "from ytdl.updater import pick_setup_asset; ..."` 로 v1.1.0 자산 목록에 대해 확인.
