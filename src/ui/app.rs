//! 화면(Slint)과 로직을 잇는다. 여기 있는 것은 모두 UI 스레드에서만 돈다.
//! 작업 스레드는 `slint::invoke_from_event_loop` 로 결과만 넘긴다.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use slint::{ComponentHandle, Model, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel};

use super::cards::{Card, CardState};
use super::download::{AddError, Change, Controller};
use super::settings::{ExtraStep, SettingsForm, ERROR_KEYS};
use super::{nav, win, AppWindow, CardView, Download, Nav, Settings};
use crate::app_info::{self, VERSION};
use crate::options::{
    apply_preset, changed_summary, matching_preset, options_from_map, options_to_map, settings_from_map,
    settings_to_map, AppSettings, DownloadOptions, AUDIO_FORMATS, AUDIO_QUALITIES, BROWSERS, CODECS, CONTAINERS,
    COOKIE_MODES, NAME_TEMPLATE_KEYS, QUALITIES, SUB_FORMATS,
};
use crate::site::{letter_of, IconPixels, SiteIcons};
use crate::store::{read_group, write_group, KeyValue, Registry};
use crate::ytdlp::{check_extra_args, run_job, Event, JobSpec, KillGroup, Tools};
use crate::{migrate, ytdlp_update};

/// 자막 언어 상자의 값. 마지막은 "직접 입력".
const SUB_LANGS: [&str; 4] = ["ko", "en", "all", ""];

pub struct App {
    pub window: slint::Weak<AppWindow>,
    pub controller: RefCell<Controller>,
    pub form: RefCell<SettingsForm>,
    /// "✓ 저장했어요" 를 끄는 타이머.
    badge_timer: slint::Timer,
    cards: Rc<VecModel<CardView>>,
    pub store: RefCell<Registry>,
    pub tools: Arc<Mutex<Tools>>,
    pub group: Arc<KillGroup>,
    pub icons: Arc<SiteIcons>,
    /// 호스트 → 아이콘. `None` 은 받는 중이거나 없는 곳.
    icon_cache: RefCell<HashMap<String, Option<slint::Image>>>,
}

thread_local! {
    static APP: RefCell<Option<Rc<App>>> = const { RefCell::new(None) };
}

/// UI 스레드에서 앱 상태로 일한다. 창이 닫힌 뒤라면 아무 일도 하지 않는다.
pub fn with_app(f: impl FnOnce(&Rc<App>)) {
    let app = APP.with(|slot| slot.borrow().clone());
    if let Some(app) = app {
        f(&app);
    }
}

/// 개발용 빌드는 설치된 앱의 설정·데이터를 건드리지 않게 따로 둔다.
fn settings_key() -> String {
    if cfg!(debug_assertions) { format!("{}-dev", app_info::SETTINGS_KEY) } else { app_info::SETTINGS_KEY.to_string() }
}

pub fn data_dir() -> PathBuf {
    let dir = app_info::data_dir();
    if cfg!(debug_assertions) { dir.with_file_name(format!("{}-dev", app_info::NAME)) } else { dir }
}

fn install_dir() -> PathBuf {
    std::env::current_exe().ok().and_then(|p| p.parent().map(PathBuf::from)).unwrap_or_default()
}

/// 개발 중에는 `scripts/fetch-tools.ps1` 로 받아 둔 것을 쓴다.
fn dev_tools_dir() -> Option<PathBuf> {
    cfg!(debug_assertions).then(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("build").join("tools"))
}

fn initial_tools() -> Tools {
    let install = install_dir();
    let dev = dev_tools_dir();
    let pick = |name: &str| {
        let bundled = install.join(name);
        if bundled.is_file() {
            return Some(bundled);
        }
        dev.as_ref().map(|d| d.join(name)).filter(|p| p.is_file())
    };
    Tools {
        ytdlp: pick("yt-dlp.exe").unwrap_or_else(|| install.join("yt-dlp.exe")),
        ffmpeg_dir: pick("ffmpeg.exe").and_then(|p| p.parent().map(PathBuf::from)),
    }
}

fn index_of(list: &[&str], value: &str) -> i32 {
    list.iter().position(|v| *v == value).unwrap_or(0) as i32
}

fn value_at(list: &[&str], index: i32) -> String {
    list.get(index.max(0) as usize).or(list.first()).copied().unwrap_or("").to_string()
}

/// 패널 값을 화면에 채운다.
fn show_options(dl: &Download, o: &DownloadOptions) {
    dl.set_kind_audio(o.kind == "audio");
    dl.set_quality(index_of(QUALITIES, &o.quality));
    dl.set_container(index_of(CONTAINERS, &o.container));
    dl.set_codec(index_of(CODECS, &o.codec));
    dl.set_audio_format(index_of(AUDIO_FORMATS, &o.audio_format));
    dl.set_audio_quality(index_of(AUDIO_QUALITIES, &o.audio_quality));
    dl.set_subs(o.subs);
    let known = SUB_LANGS[..3].contains(&o.sub_langs.as_str());
    dl.set_sub_lang(if known { index_of(&SUB_LANGS, &o.sub_langs) } else { 3 });
    dl.set_sub_lang_custom(if known { "".into() } else { o.sub_langs.as_str().into() });
    dl.set_auto_subs(o.auto_subs);
    dl.set_sub_file(o.sub_mode == "file");
    dl.set_sub_format(index_of(SUB_FORMATS, &o.sub_format));
    dl.set_embed_thumbnail(o.embed_thumbnail);
    dl.set_save_thumbnail(o.save_thumbnail);
    dl.set_embed_metadata(o.embed_metadata);
    dl.set_embed_chapters(o.embed_chapters);
    dl.set_split_chapters(o.split_chapters);
    dl.set_section_start(o.section_start.as_str().into());
    dl.set_section_end(o.section_end.as_str().into());
    dl.set_playlist_all(o.playlist == "all");
    dl.set_playlist_items(o.playlist_items.as_str().into());
    dl.set_number_prefix(o.number_prefix);
    dl.set_playlist_folder(o.playlist_folder);
    dl.set_skip_archived(o.skip_archived);
}

/// 화면의 패널 값.
fn read_options(dl: &Download) -> DownloadOptions {
    let lang = match dl.get_sub_lang() {
        i @ 0..=2 => SUB_LANGS[i as usize].to_string(),
        _ => dl.get_sub_lang_custom().trim().to_string(),
    };
    DownloadOptions {
        kind: if dl.get_kind_audio() { "audio" } else { "video" }.into(),
        quality: value_at(QUALITIES, dl.get_quality()),
        container: value_at(CONTAINERS, dl.get_container()),
        codec: value_at(CODECS, dl.get_codec()),
        audio_format: value_at(AUDIO_FORMATS, dl.get_audio_format()),
        audio_quality: value_at(AUDIO_QUALITIES, dl.get_audio_quality()),
        subs: dl.get_subs(),
        sub_langs: if lang.is_empty() { "ko".into() } else { lang },
        auto_subs: dl.get_auto_subs(),
        sub_mode: if dl.get_sub_file() { "file" } else { "embed" }.into(),
        sub_format: value_at(SUB_FORMATS, dl.get_sub_format()),
        embed_thumbnail: dl.get_embed_thumbnail(),
        save_thumbnail: dl.get_save_thumbnail(),
        embed_metadata: dl.get_embed_metadata(),
        embed_chapters: dl.get_embed_chapters(),
        split_chapters: dl.get_split_chapters(),
        section_start: dl.get_section_start().trim().to_string(),
        section_end: dl.get_section_end().trim().to_string(),
        playlist: if dl.get_playlist_all() { "all" } else { "single" }.into(),
        playlist_items: dl.get_playlist_items().trim().to_string(),
        number_prefix: dl.get_number_prefix(),
        playlist_folder: dl.get_playlist_folder(),
        skip_archived: dl.get_skip_archived(),
    }
}

/// 저장된 설정으로 칸을 채운다.
fn show_settings(st: &Settings, s: &AppSettings) {
    st.set_cookies(index_of(COOKIE_MODES, &s.cookies));
    st.set_cookie_browser(index_of(BROWSERS, &s.cookie_browser));
    st.set_cookie_file(s.cookie_file.as_str().into());
    st.set_proxy(s.proxy.as_str().into());
    st.set_rate_limit(s.rate_limit.as_str().into());
    st.set_retries(s.retries as i32);
    st.set_concurrency(s.concurrency as i32);
    st.set_name_template(index_of(NAME_TEMPLATE_KEYS, &s.name_template));
    st.set_custom_template(s.custom_template.as_str().into());
    st.set_overwrite(s.overwrite);
    st.set_extra_args(s.extra_args.as_str().into());
}

/// 칸에 지금 들어 있는 값.
fn read_settings(st: &Settings) -> AppSettings {
    AppSettings {
        cookies: value_at(COOKIE_MODES, st.get_cookies()),
        cookie_browser: value_at(BROWSERS, st.get_cookie_browser()),
        cookie_file: st.get_cookie_file().to_string(),
        proxy: st.get_proxy().to_string(),
        rate_limit: st.get_rate_limit().to_string(),
        retries: st.get_retries().max(0) as u32,
        name_template: value_at(NAME_TEMPLATE_KEYS, st.get_name_template()),
        custom_template: st.get_custom_template().to_string(),
        overwrite: st.get_overwrite(),
        concurrency: st.get_concurrency().max(1) as u32,
        extra_args: st.get_extra_args().to_string(),
    }
}

fn to_image(pixels: &IconPixels) -> slint::Image {
    let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&pixels.rgba, pixels.width, pixels.height);
    slint::Image::from_rgba8(buffer)
}

impl App {
    fn window(&self) -> Option<AppWindow> {
        self.window.upgrade()
    }

    /// 프리셋 버튼 선택과 "세부 옵션" 버튼 글자를 지금 값에 맞춘다.
    fn sync_format_row(&self) {
        let Some(window) = self.window() else { return };
        let dl = window.global::<Download>();
        let controller = self.controller.borrow();
        let opts = controller.options();
        dl.set_preset(matching_preset(opts).unwrap_or("").into());
        let changed = changed_summary(opts);
        let text = if changed.is_empty() { "세부 옵션".to_string() } else { format!("세부 옵션 · {}", changed.join(", ")) };
        dl.set_more_text(text.into());
    }

    fn save_options(&self) {
        let values = options_to_map(self.controller.borrow().options());
        write_group(&mut *self.store.borrow_mut(), "options", &values);
    }

    fn view(&self, card: &Card) -> CardView {
        let icon = self.icon_for(&card.host);
        CardView {
            id: card.id as i32,
            site: card.site.as_str().into(),
            letter: letter_of(&card.host).into(),
            has_icon: icon.is_some(),
            icon: icon.unwrap_or_default(),
            title: card.title.as_str().into(),
            status: card.status.as_str().into(),
            tone: match card.state {
                CardState::Done => "done",
                CardState::Error => "error",
                _ => "",
            }
            .into(),
            percent_text: card.percent_text().into(),
            progress: card.percent / 100.0,
            show_bar: card.show_bar(),
            can_open: card.state == CardState::Done,
        }
    }

    /// 저장해 둔 아이콘. 처음 보는 곳이면 뒤에서 받아 오고, 오면 그 사이트의 카드를 다시 그린다.
    fn icon_for(&self, host: &str) -> Option<slint::Image> {
        if host.is_empty() {
            return None;
        }
        if let Some(known) = self.icon_cache.borrow().get(host) {
            return known.clone();
        }
        self.icon_cache.borrow_mut().insert(host.to_string(), None);
        let (icons, host) = (self.icons.clone(), host.to_string());
        std::thread::spawn(move || {
            if let Some(pixels) = icons.load(&host) {
                let _ = slint::invoke_from_event_loop(move || with_app(|app| app.icon_ready(&host, &pixels)));
            }
        });
        None
    }

    fn icon_ready(&self, host: &str, pixels: &IconPixels) {
        self.icon_cache.borrow_mut().insert(host.to_string(), Some(to_image(pixels)));
        let controller = self.controller.borrow();
        for (index, card) in controller.cards().iter().enumerate() {
            if card.host == host && index < self.cards.row_count() {
                self.cards.set_row_data(index, self.view(card));
            }
        }
    }

    /// 카드 목록에 생긴 변화를 화면에 옮긴다.
    pub fn sync(&self) {
        let changes = self.controller.borrow_mut().take_changes();
        let controller = self.controller.borrow();
        let cards = controller.cards();
        // 변화는 일어난 순서대로지만 내용은 지금 것을 쓴다. 넣은 자리의 카드가 그새 지워졌을 수는 없다.
        let mut shadow: Vec<Option<u32>> = (0..self.cards.row_count()).map(|i| self.cards.row_data(i).map(|v| v.id as u32)).collect();
        for change in &changes {
            match *change {
                Change::Insert(at) => shadow.insert(at.min(shadow.len()), None),
                Change::Remove(at) if at < shadow.len() => {
                    shadow.remove(at);
                }
                _ => {}
            }
        }
        // 길이가 달라졌거나 순서가 바뀌었으면 통째로 다시 채운다. 아니면 바뀐 줄만.
        let same_shape = shadow.len() == self.cards.row_count() && !changes.iter().any(|c| !matches!(c, Change::Update(_)));
        if same_shape {
            for change in changes {
                if let Change::Update(index) = change {
                    if let Some(card) = cards.get(index) {
                        self.cards.set_row_data(index, self.view(card));
                    }
                }
            }
        } else {
            self.cards.set_vec(cards.iter().map(|c| self.view(c)).collect::<Vec<_>>());
        }
    }

    pub fn on_event(&self, id: u32, event: Event) {
        self.controller.borrow_mut().on_event(id, event);
        self.sync();
    }

    fn add(&self) {
        let Some(window) = self.window() else { return };
        let dl = window.global::<Download>();
        let result = self.controller.borrow_mut().add_url(&dl.get_url());
        match result {
            Ok(()) => {
                self.save_options();
                dl.set_url("".into());
                dl.set_error("".into());
                self.sync();
            }
            Err(AddError::EmptyLink) => {
                dl.set_url("".into());
                dl.set_url_placeholder("먼저 링크를 넣어 주세요".into());
                window.invoke_focus_url();
            }
            Err(AddError::Options(message)) => dl.set_error(message.into()),
            Err(AddError::Closing) => {}
        }
    }

    /// 칸 아래 오류 문구를 화면에 맞춘다.
    fn show_setting_errors(&self) {
        let Some(window) = self.window() else { return };
        let st = window.global::<Settings>();
        let form = self.form.borrow();
        for key in ERROR_KEYS {
            let text: slint::SharedString = form.error(key).into();
            match key {
                "cookie_file" => st.set_error_cookie_file(text),
                "rate_limit" => st.set_error_rate_limit(text),
                "custom_template" => st.set_error_custom_template(text),
                _ => st.set_error_extra_args(text),
            }
        }
    }

    /// 저장된 설정이 바뀌었다: 레지스트리에 쓰고, 대기열·아이콘에 알리고, "저장했어요" 를 잠깐 보인다.
    fn settings_saved(&self) {
        let saved = self.form.borrow().saved().clone();
        write_group(&mut *self.store.borrow_mut(), "settings", &settings_to_map(&saved));
        self.icons.set_proxy(&saved.proxy);
        self.controller.borrow_mut().set_settings(saved);
        self.sync();
        let Some(window) = self.window() else { return };
        window.global::<Settings>().set_saved_badge(true);
        self.badge_timer.start(slint::TimerMode::SingleShot, std::time::Duration::from_millis(1500), || {
            with_app(|app| {
                if let Some(window) = app.window() {
                    window.global::<Settings>().set_saved_badge(false);
                }
            })
        });
    }

    /// 고급 옵션을 뒤에서 yt-dlp 로 확인하고, 결과가 오면 저장하거나 오류를 보인다.
    fn check_extra_later(&self, text: String) {
        let tools = self.tools.lock().unwrap().clone();
        let group = self.group.clone();
        std::thread::spawn(move || {
            let error = check_extra_args(&tools, &group, &text);
            let _ = slint::invoke_from_event_loop(move || {
                with_app(|app| {
                    let changed = app.form.borrow_mut().apply_extra(&text, error);
                    app.show_setting_errors();
                    if changed {
                        app.settings_saved();
                        // 그사이 화면을 떠나 되돌려진 칸은 새 값으로 채운다
                        if let Some(window) = app.window() {
                            let st = window.global::<Settings>();
                            st.set_extra_args(app.form.borrow().view().extra_args.as_str().into());
                        }
                    }
                })
            });
        });
    }

    /// 묶음의 칸을 저장한다. 틀리면 칸 아래 오류를 보인다.
    fn commit_settings(&self, group: &str) {
        let Some(window) = self.window() else { return };
        self.form.borrow_mut().set_view(read_settings(&window.global::<Settings>()));
        let changed = if group == "extra_args" {
            let step = self.form.borrow().extra_step();
            match step {
                ExtraStep::Ask(text) => {
                    self.form.borrow_mut().clear_error("extra_args");
                    self.check_extra_later(text);
                    false
                }
                // 저장된 값 그대로거나 yt-dlp 없이 알 수 있는 경우
                _ => self.form.borrow_mut().commit(group, &|_| None),
            }
        } else {
            self.form.borrow_mut().commit(group, &|_| None)
        };
        self.show_setting_errors();
        if changed {
            self.settings_saved();
        }
    }

    /// 설정 화면을 떠나거나 창을 닫을 때. 맞는 글자는 저장하고 나머지는 저장된 값으로 되돌린다.
    /// `closing` 이면 고급 옵션 확인을 기다렸다가 끝낸다.
    fn leave_settings(&self, closing: bool) {
        let Some(window) = self.window() else { return };
        let st = window.global::<Settings>();
        if !nav::is_settings_page(&window.global::<Nav>().get_page()) {
            return;
        }
        self.form.borrow_mut().set_view(read_settings(&st));
        let step = self.form.borrow().extra_step();
        let changed = match step {
            ExtraStep::Ask(text) if closing => {
                let tools = self.tools.lock().unwrap().clone();
                let error = check_extra_args(&tools, &self.group, &text);
                self.form.borrow_mut().leave(&|_| error.clone())
            }
            ExtraStep::Ask(text) => {
                self.check_extra_later(text);
                // 확인이 끝나기 전에는 고급 옵션을 저장하지 않는다
                self.form.borrow_mut().leave(&|_| Some(String::new()))
            }
            _ => self.form.borrow_mut().leave(&|_| None),
        };
        show_settings(&st, self.form.borrow().view());
        self.show_setting_errors();
        if changed {
            self.settings_saved();
        }
    }

    fn options_changed(&self, opts: DownloadOptions) {
        self.controller.borrow_mut().set_options(opts);
        self.save_options();
        self.sync_format_row();
        if let Some(window) = self.window() {
            window.global::<Download>().set_error("".into());
        }
    }
}

/// 작업을 실제로 돌린다: 스레드에서 yt-dlp 를 돌리고 소식을 UI 스레드로 넘긴다.
fn start_job(tools: &Arc<Mutex<Tools>>, group: &Arc<KillGroup>, id: u32, spec: JobSpec) {
    let tools = tools.lock().unwrap().clone();
    let group = group.clone();
    std::thread::spawn(move || {
        let emit = move |event: Event| {
            let _ = slint::invoke_from_event_loop(move || with_app(|app| app.on_event(id, event)));
        };
        let job = std::panic::AssertUnwindSafe(|| run_job(&spec, &tools, &group, &emit));
        if std::panic::catch_unwind(job).is_err() {
            emit(Event::Failed("알 수 없는 오류가 났어요".into()));
        }
    });
}

fn wire_nav(window: &AppWindow) {
    let nav_global = window.global::<Nav>();
    nav_global.set_version(format!("버전 {VERSION}").into());
    let weak = window.as_weak();
    nav_global.on_go(move |key| {
        let Some(window) = weak.upgrade() else { return };
        let nav_global = window.global::<Nav>();
        let (page, open) = nav::nav_after_click(&nav_global.get_page(), nav_global.get_settings_open(), &key);
        if page != nav_global.get_page().as_str() {
            with_app(|app| app.leave_settings(false));
        }
        nav_global.set_page(page.into());
        nav_global.set_settings_open(open);
    });
}

fn wire_download(window: &AppWindow) {
    let dl = window.global::<Download>();
    dl.on_add(|| with_app(|app| app.add()));
    dl.on_paste(|| {
        with_app(|app| {
            let Some(window) = app.window() else { return };
            if let Some(text) = win::clipboard_text().map(|t| t.trim().to_string()).filter(|t| !t.is_empty()) {
                window.global::<Download>().set_url(text.into());
                window.invoke_focus_url();
            }
        })
    });
    dl.on_preset_clicked(|key| {
        with_app(|app| {
            let Some(window) = app.window() else { return };
            let opts = apply_preset(app.controller.borrow().options(), &key);
            show_options(&window.global::<Download>(), &opts);
            app.options_changed(opts);
        })
    });
    dl.on_toggle_panel(|| {
        with_app(|app| {
            let Some(window) = app.window() else { return };
            let dl = window.global::<Download>();
            let open = !dl.get_panel_open();
            dl.set_panel_open(open);
            app.store.borrow_mut().set("ui/panel_open", if open { "true" } else { "false" });
        })
    });
    dl.on_options_edited(|| {
        with_app(|app| {
            let Some(window) = app.window() else { return };
            app.options_changed(read_options(&window.global::<Download>()));
        })
    });
    dl.on_clear_finished(|| {
        with_app(|app| {
            app.controller.borrow_mut().clear_finished();
            app.sync();
        })
    });
    dl.on_reveal(|id| {
        with_app(|app| {
            let path = app.controller.borrow().cards().iter().find(|c| c.id as i32 == id).map(|c| c.path.clone());
            if let Some(path) = path.filter(|p| !p.is_empty()) {
                win::reveal(&path);
            }
        })
    });
    dl.on_pick_folder(|| {
        with_app(|app| {
            let Some(window) = app.window() else { return };
            let start = app.controller.borrow().folder().to_string();
            if let Some(dir) = win::pick_folder("저장 위치 고르기", &start) {
                let dir = dir.to_string_lossy().into_owned();
                app.store.borrow_mut().set("folder", &dir);
                window.global::<Download>().set_folder(dir.as_str().into());
                app.controller.borrow_mut().set_folder(dir);
            }
        })
    });
    dl.on_open_folder(|| with_app(|app| win::open_folder(app.controller.borrow().folder())));
}

fn wire_settings(window: &AppWindow) {
    let st = window.global::<Settings>();
    st.on_commit(|group| with_app(|app| app.commit_settings(&group)));
    st.on_browse_cookie(|| {
        with_app(|app| {
            let Some(window) = app.window() else { return };
            if let Some(path) = win::pick_file("cookies.txt 고르기", "쿠키 파일", &["txt"]) {
                window.global::<Settings>().set_cookie_file(path.to_string_lossy().as_ref().into());
                app.commit_settings("cookies");
            }
        })
    });
}

/// 창을 만들고 저장된 값으로 채운다.
pub fn build() -> Result<AppWindow, slint::PlatformError> {
    let window = AppWindow::new()?;

    let mut store = Registry::open(&settings_key());
    // 옛 유튜브 다운로더에서 넘어온 설정과 파일 정리. 실패해도 앱은 켜진다.
    let _ = migrate::run(&mut store, &Registry::open(app_info::LEGACY_SETTINGS_KEY));

    let downloads = std::env::var_os("USERPROFILE").map(PathBuf::from).unwrap_or_default().join("Downloads");
    let folder = store.get("folder").filter(|f| !f.is_empty()).unwrap_or_else(|| downloads.to_string_lossy().into_owned());
    let opts = options_from_map(&read_group(&store, "options"), store.get("format").as_deref());
    let settings = settings_from_map(&read_group(&store, "settings"));
    let panel_open = store.get("ui/panel_open").is_some_and(|v| v.eq_ignore_ascii_case("true"));

    let data = data_dir();
    let _ = std::fs::create_dir_all(&data);
    let icons = Arc::new(SiteIcons::new(data.join("favicons")));
    icons.set_proxy(&settings.proxy);

    let group = Arc::new(KillGroup::new().map_err(|e| slint::PlatformError::Other(e.to_string()))?);
    let tools = Arc::new(Mutex::new(initial_tools()));

    let (job_tools, job_group) = (tools.clone(), group.clone());
    let controller = Controller::new(
        opts.clone(),
        settings.clone(),
        folder.clone(),
        data.join("archive.txt"),
        Box::new(move |id, spec| start_job(&job_tools, &job_group, id, spec)),
        Box::new(|n| win::confirm("재생목록", &format!("영상 {n}개를 모두 받을까요?"))),
    );

    let cards = Rc::new(VecModel::<CardView>::default());
    let dl = window.global::<Download>();
    dl.set_cards(ModelRc::from(cards.clone()));
    dl.set_folder(folder.as_str().into());
    dl.set_panel_open(panel_open);
    show_options(&dl, &opts);
    show_settings(&window.global::<Settings>(), &settings);

    let app = Rc::new(App {
        window: window.as_weak(),
        controller: RefCell::new(controller),
        form: RefCell::new(SettingsForm::new(settings.clone())),
        badge_timer: slint::Timer::default(),
        cards,
        store: RefCell::new(store),
        tools,
        group,
        icons,
        icon_cache: RefCell::default(),
    });
    APP.with(|slot| *slot.borrow_mut() = Some(app.clone()));
    app.sync_format_row();

    wire_nav(&window);
    wire_download(&window);
    wire_settings(&window);

    // 닫을 때: 기다리던 작업은 버린다. 돌던 yt-dlp·ffmpeg 는 앱이 끝나며 작업 개체와 함께 끝난다.
    window.window().on_close_requested(|| {
        with_app(|app| {
            app.leave_settings(true);
            app.controller.borrow_mut().shutdown();
        });
        slint::CloseRequestResponse::HideWindow
    });

    locate_ytdlp(&app);
    Ok(window)
}

/// 쓸 yt-dlp 를 뒤에서 고른다(설치 폴더 것과 갱신본 중 새것). 처음에는 버전을 알아보느라 1초쯤 걸린다.
fn locate_ytdlp(app: &Rc<App>) {
    let mut store = app.store.borrow().clone();
    let group = app.group.clone();
    let (install, bin) = (install_dir(), data_dir().join("bin"));
    std::thread::spawn(move || {
        if let Some(found) = ytdlp_update::locate(&install, &bin, &mut store, &group) {
            let _ = slint::invoke_from_event_loop(move || {
                with_app(|app| app.tools.lock().unwrap().ytdlp = found.path.clone());
            });
        }
    });
}

/// 창이 닫힌 뒤 정리.
pub fn teardown() {
    APP.with(|slot| slot.borrow_mut().take());
}
