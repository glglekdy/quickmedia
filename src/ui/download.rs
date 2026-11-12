//! 다운로드 화면의 동작: 링크를 대기열에 넣고, 작업이 알려 오는 것을 카드에 반영한다.
//!
//! 화면(Slint)도 프로세스도 모른다. 작업을 실제로 시작하는 함수와 물어보는 함수를 밖에서 받는다.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use super::cards::{apply_event, is_finished, Card, CardState};
use crate::options::{describe, validate_options, AppSettings, DownloadOptions, PlaylistContext};
use crate::queue::JobQueue;
use crate::ytdlp::{Event, JobSpec, PlaylistEntry};

/// 이보다 많은 영상을 한 번에 넣기 전에는 물어본다.
pub const CONFIRM_OVER: usize = 200;

/// 카드 목록이 어떻게 바뀌었는지. 화면은 이 순서대로 따라 한다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Change {
    Insert(usize),
    Update(usize),
    Remove(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub enum AddError {
    EmptyLink,
    /// 세부 옵션이 틀렸다. 화면에 보일 문구.
    Options(String),
    /// 앱을 닫는 중.
    Closing,
}

pub struct Controller {
    opts: DownloadOptions,
    settings: AppSettings,
    folder: String,
    archive: PathBuf,
    cards: Vec<Card>,
    /// 아직 끝나지 않은 작업의 값. 재생목록 항목은 부모의 값을 물려받는다.
    specs: HashMap<u32, JobSpec>,
    queue: JobQueue<u32>,
    /// 대기열이 "시작하라"고 한 작업. 대기열을 건드린 뒤 곧바로 비운다.
    to_start: Rc<RefCell<Vec<u32>>>,
    start: Box<dyn FnMut(u32, JobSpec)>,
    confirm_many: Box<dyn FnMut(usize) -> bool>,
    changes: Vec<Change>,
    next_id: u32,
    closing: bool,
}

impl Controller {
    /// `start` 는 작업을 실제로 돌리고, `confirm_many` 는 "영상 N개를 모두 받을까요?" 를 묻는다.
    pub fn new(
        opts: DownloadOptions,
        settings: AppSettings,
        folder: String,
        archive: PathBuf,
        start: Box<dyn FnMut(u32, JobSpec)>,
        confirm_many: Box<dyn FnMut(usize) -> bool>,
    ) -> Self {
        let to_start: Rc<RefCell<Vec<u32>>> = Rc::default();
        let sink = to_start.clone();
        let queue = JobQueue::new(settings.concurrency as usize, Box::new(move |id| sink.borrow_mut().push(id)));
        Self {
            opts,
            settings,
            folder,
            archive,
            cards: Vec::new(),
            specs: HashMap::new(),
            queue,
            to_start,
            start,
            confirm_many,
            changes: Vec::new(),
            next_id: 1,
            closing: false,
        }
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn options(&self) -> &DownloadOptions {
        &self.opts
    }

    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    pub fn folder(&self) -> &str {
        &self.folder
    }

    pub fn set_options(&mut self, opts: DownloadOptions) {
        self.opts = opts;
    }

    pub fn set_folder(&mut self, folder: String) {
        self.folder = folder;
    }

    pub fn set_settings(&mut self, settings: AppSettings) {
        self.set_concurrency(settings.concurrency as usize);
        self.settings = settings;
    }

    pub fn set_concurrency(&mut self, n: usize) {
        self.queue.set_limit(n);
        self.flush_starts();
    }

    /// 받는 중이거나 기다리는 작업 수.
    pub fn active_count(&self) -> usize {
        self.queue.running().len() + self.queue.pending().len()
    }

    /// 마지막으로 가져간 뒤에 카드 목록에 생긴 변화.
    pub fn take_changes(&mut self) -> Vec<Change> {
        std::mem::take(&mut self.changes)
    }

    /// 링크를 지금 옵션·설정·폴더로 대기열에 넣는다. 새 카드는 맨 위에 온다.
    pub fn add_url(&mut self, url: &str) -> Result<(), AddError> {
        if self.closing {
            return Err(AddError::Closing);
        }
        let url = url.trim();
        if url.is_empty() {
            return Err(AddError::EmptyLink);
        }
        if let Some(message) = validate_options(&self.opts).into_values().next() {
            return Err(AddError::Options(message));
        }
        let spec = JobSpec {
            url: url.to_string(),
            opts: self.opts.clone(),
            settings: self.settings.clone(),
            folder: self.folder.clone(),
            archive: self.archive.clone(),
            playlist: None,
        };
        self.insert(0, spec);
        self.flush_starts();
        Ok(())
    }

    fn insert(&mut self, at: usize, spec: JobSpec) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.cards.insert(at, Card::new(id, &spec.url, &describe(&spec.opts)));
        self.changes.push(Change::Insert(at));
        self.specs.insert(id, spec);
        self.queue.add(id);
        id
    }

    fn index_of(&self, id: u32) -> Option<usize> {
        self.cards.iter().position(|c| c.id == id)
    }

    fn flush_starts(&mut self) {
        let ids = std::mem::take(&mut *self.to_start.borrow_mut());
        for id in ids {
            let (Some(index), Some(spec)) = (self.index_of(id), self.specs.get(&id).cloned()) else { continue };
            self.cards[index].set_started();
            self.changes.push(Change::Update(index));
            (self.start)(id, spec);
        }
    }

    /// 작업이 알려 온 것. 끝났다는 소식이면 다음 작업을 시작한다.
    pub fn on_event(&mut self, id: u32, event: Event) {
        // 이미 끝난(또는 모르는) 작업의 소식은 버린다
        if !self.specs.contains_key(&id) {
            return;
        }
        let Some(index) = self.index_of(id) else { return };
        let ended = matches!(event, Event::Done(_) | Event::Skipped | Event::Failed(_) | Event::Expanded { .. });
        match event {
            Event::Expanded { title, entries, skipped } => self.expand(id, index, &title, entries, skipped),
            other => {
                apply_event(&mut self.cards[index], &other);
                self.changes.push(Change::Update(index));
            }
        }
        if ended {
            self.specs.remove(&id);
            if !self.closing {
                self.queue.done(&id);
                self.flush_starts();
            }
        }
    }

    fn expand(&mut self, id: u32, index: usize, title: &str, entries: Vec<PlaylistEntry>, skipped: u32) {
        if entries.len() > CONFIRM_OVER && !(self.confirm_many)(entries.len()) {
            self.cards[index].set_cancelled();
            self.changes.push(Change::Update(index));
            return;
        }
        self.cards[index].set_playlist(title, entries.len(), skipped);
        self.changes.push(Change::Update(index));
        let Some(parent) = self.specs.get(&id).cloned() else { return };
        let site = self.cards[index].site.clone();
        let count = entries.len() as u32;
        for (i, entry) in entries.into_iter().enumerate() {
            let spec = JobSpec {
                url: entry.url,
                playlist: Some(PlaylistContext { title: title.to_string(), index: entry.index, count }),
                ..parent.clone()
            };
            let at = index + 1 + i;
            self.insert(at, spec);
            self.cards[at].title = entry.title;
            self.cards[at].site = site.clone();
        }
    }

    pub fn clear_finished(&mut self) {
        for index in (0..self.cards.len()).rev() {
            if is_finished(self.cards[index].state) {
                self.cards.remove(index);
                self.changes.push(Change::Remove(index));
            }
        }
    }

    /// 앱을 닫는다. 기다리던 작업은 버리고, 돌던 작업이 끝나도 다음을 시작하지 않는다.
    pub fn shutdown(&mut self) {
        self.closing = true;
        self.queue.clear_pending();
    }

    /// 카드의 지금 상태(시험·화면용).
    pub fn state_of(&self, id: u32) -> Option<CardState> {
        self.index_of(id).map(|i| self.cards[i].state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    type Started = Rc<RefCell<Vec<(u32, JobSpec)>>>;

    fn make(settings: AppSettings, answer: bool) -> (Controller, Started, Rc<RefCell<Vec<usize>>>) {
        let started: Started = Rc::default();
        let asked: Rc<RefCell<Vec<usize>>> = Rc::default();
        let (sink, asks) = (started.clone(), asked.clone());
        let c = Controller::new(
            DownloadOptions::default(),
            settings,
            "D:/v".into(),
            "A:/archive.txt".into(),
            Box::new(move |id, spec| sink.borrow_mut().push((id, spec))),
            Box::new(move |n| {
                asks.borrow_mut().push(n);
                answer
            }),
        );
        (c, started, asked)
    }

    fn controller() -> (Controller, Started) {
        let (c, started, _) = make(AppSettings::default(), true);
        (c, started)
    }

    fn entries(n: usize) -> Vec<PlaylistEntry> {
        (1..=n).map(|i| PlaylistEntry { url: format!("https://y/{i}"), title: format!("영상 {i}"), index: i as u32 }).collect()
    }

    fn titles(c: &Controller) -> Vec<&str> {
        c.cards().iter().map(|card| card.title.as_str()).collect()
    }

    #[test]
    fn empty_link_is_refused() {
        let (mut c, started) = controller();
        assert_eq!(c.add_url("   "), Err(AddError::EmptyLink));
        assert!(c.cards().is_empty() && started.borrow().is_empty());
    }

    #[test]
    fn invalid_options_block_add() {
        let (mut c, started) = controller();
        c.set_options(DownloadOptions { section_start: "abc".into(), ..Default::default() });
        assert_eq!(c.add_url("https://y/1"), Err(AddError::Options("시간은 1:30 처럼 써 주세요".into())));
        assert!(c.cards().is_empty() && started.borrow().is_empty());
    }

    #[test]
    fn add_snapshots_options_settings_and_folder() {
        let (mut c, started, _) = make(AppSettings { concurrency: 1, ..Default::default() }, true);
        c.add_url(" https://y/1 ").unwrap();
        c.add_url("https://y/2").unwrap();
        // 대기 중에 패널·설정·폴더를 바꿔도 이미 넣은 항목은 그대로 받는다
        c.set_options(DownloadOptions { kind: "audio".into(), ..Default::default() });
        c.set_settings(AppSettings { proxy: "http://p:1".into(), concurrency: 1, ..Default::default() });
        c.set_folder("E:/new".into());
        c.on_event(1, Event::Done("D:/v/1.mp4".into()));
        let started = started.borrow();
        assert_eq!(started.len(), 2);
        let (id, spec) = &started[1];
        assert_eq!((*id, spec.url.as_str(), spec.opts.kind.as_str(), spec.settings.proxy.as_str(), spec.folder.as_str()),
                   (2, "https://y/2", "video", "", "D:/v"));
        assert_eq!(spec.archive, std::path::PathBuf::from("A:/archive.txt"));
        assert!(spec.playlist.is_none());
        assert_eq!(c.cards()[0].format, "최고 화질 · MP4");
    }

    #[test]
    fn newest_card_goes_on_top_and_started_cards_load() {
        let (mut c, _, _) = make(AppSettings { concurrency: 1, ..Default::default() }, true);
        c.add_url("https://y/1").unwrap();
        c.add_url("https://y/2").unwrap();
        assert_eq!(titles(&c), ["https://y/2", "https://y/1"]);
        assert_eq!(c.cards()[1].state, CardState::Loading);
        assert_eq!(c.cards()[0].state, CardState::Waiting);
        assert_eq!(c.active_count(), 2);
    }

    #[test]
    fn concurrency_limit_and_change() {
        let (mut c, started) = controller(); // 기본 2개
        for i in 1..=3 {
            c.add_url(&format!("https://y/{i}")).unwrap();
        }
        assert_eq!(started.borrow().len(), 2);
        c.set_concurrency(3);
        assert_eq!(started.borrow().len(), 3);
    }

    #[test]
    fn failure_frees_the_slot() {
        let (mut c, started, _) = make(AppSettings { concurrency: 1, ..Default::default() }, true);
        c.add_url("https://y/1").unwrap();
        c.add_url("https://y/2").unwrap();
        c.on_event(1, Event::Failed("x".into()));
        assert_eq!(started.borrow().len(), 2);
        assert_eq!(c.active_count(), 1);
        // 이미 끝난 작업이 또 알려 와도 아무 일 없다
        c.on_event(1, Event::Done("p".into()));
        c.on_event(99, Event::Skipped);
        assert_eq!(c.cards()[1].state, CardState::Error);
    }

    #[test]
    fn playlist_expands_below_its_card() {
        let (mut c, started, _) = make(AppSettings { concurrency: 1, ..Default::default() }, true);
        c.add_url("https://y/before").unwrap(); // id 1, 맨 아래
        c.set_options(DownloadOptions { playlist: "all".into(), number_prefix: true, ..Default::default() });
        c.add_url("https://y/list").unwrap(); // id 2, 맨 위
        c.on_event(1, Event::Skipped); // 이제 2 가 시작된다
        c.on_event(2, Event::Site { key: "YoutubeTab".into(), url: "https://www.youtube.com/playlist?list=x".into() });
        let mut list = entries(3);
        list[2].index = 7; // 범위를 골라 받으면 순번이 건너뛴다
        c.on_event(2, Event::Expanded { title: "모음".into(), entries: list, skipped: 2 });

        assert_eq!(titles(&c), ["재생목록 · 모음", "영상 1", "영상 2", "영상 3", "https://y/before"]);
        let parent = &c.cards()[0];
        assert_eq!((parent.state, parent.status.as_str()), (CardState::Playlist, "영상 3개 추가 · 2개는 이미 받아서 건너뜀"));
        // 항목은 재생목록 카드의 사이트 이름을 물려받는다
        assert_eq!(c.cards()[1].site, "YouTube");
        assert_eq!(c.cards()[1].host, "y");

        // 첫 항목이 재생목록 자리를 이어받아 시작된다
        let started = started.borrow();
        let (id, spec) = started.last().unwrap();
        assert_eq!(*id, c.cards()[1].id);
        assert_eq!(spec.url, "https://y/1");
        assert_eq!(spec.playlist, Some(PlaylistContext { title: "모음".into(), index: 1, count: 3 }));
        assert!(spec.opts.number_prefix && spec.opts.playlist == "all");
        assert_eq!(c.cards()[1].state, CardState::Loading);
        assert_eq!(c.cards()[3].state, CardState::Waiting);
        assert_eq!(c.active_count(), 3);
    }

    #[test]
    fn last_playlist_item_keeps_its_original_index() {
        let (mut c, started, _) = make(AppSettings { concurrency: 5, ..Default::default() }, true);
        c.add_url("https://y/list").unwrap();
        let mut list = entries(2);
        list[1].index = 9;
        c.on_event(1, Event::Expanded { title: "p".into(), entries: list, skipped: 0 });
        let contexts: Vec<_> = started.borrow().iter().filter_map(|(_, s)| s.playlist.clone()).collect();
        assert_eq!(contexts, [
            PlaylistContext { title: "p".into(), index: 1, count: 2 },
            PlaylistContext { title: "p".into(), index: 9, count: 2 },
        ]);
    }

    #[test]
    fn many_entries_ask_first_and_can_cancel() {
        let (mut c, started, asked) = make(AppSettings::default(), false);
        c.add_url("https://y/list").unwrap();
        c.on_event(1, Event::Expanded { title: "큰 목록".into(), entries: entries(CONFIRM_OVER + 1), skipped: 0 });
        assert_eq!(*asked.borrow(), [CONFIRM_OVER + 1]);
        assert_eq!(c.cards().len(), 1);
        assert_eq!((c.cards()[0].state, c.cards()[0].status.as_str()), (CardState::Cancelled, "취소했어요"));
        assert_eq!(started.borrow().len(), 1);
        assert_eq!(c.active_count(), 0);

        // 딱 한도까지는 묻지 않는다
        let (mut c, _, asked) = make(AppSettings::default(), false);
        c.add_url("https://y/list").unwrap();
        c.on_event(1, Event::Expanded { title: "p".into(), entries: entries(CONFIRM_OVER), skipped: 0 });
        assert!(asked.borrow().is_empty());
        assert_eq!(c.cards().len(), CONFIRM_OVER + 1);
    }

    #[test]
    fn clear_finished_keeps_running_and_waiting() {
        let (mut c, _, _) = make(AppSettings { concurrency: 1, ..Default::default() }, true);
        for i in 1..=4 {
            c.add_url(&format!("https://y/{i}")).unwrap();
        }
        c.on_event(1, Event::Done("p".into()));
        c.on_event(2, Event::Failed("x".into()));
        c.take_changes();
        c.clear_finished();
        assert_eq!(titles(&c), ["https://y/4", "https://y/3"]);
        // 아래에서부터 지운 자리를 알려 준다
        assert_eq!(c.take_changes(), [Change::Remove(3), Change::Remove(2)]);
    }

    #[test]
    fn shutdown_does_not_start_waiting_jobs() {
        let (mut c, started, _) = make(AppSettings { concurrency: 1, ..Default::default() }, true);
        c.add_url("https://y/1").unwrap();
        c.add_url("https://y/2").unwrap();
        c.shutdown();
        // 닫는 중에 끝난 작업 자리에 대기 중인 작업을 시작하지 않는다
        c.on_event(1, Event::Failed("끊김".into()));
        assert_eq!(started.borrow().len(), 1);
        assert_eq!(c.add_url("https://y/3"), Err(AddError::Closing));
    }

    #[test]
    fn changes_follow_what_happened() {
        let (mut c, _) = controller();
        c.add_url("https://y/1").unwrap();
        // 넣고(맨 위) 곧바로 시작했다
        assert_eq!(c.take_changes(), [Change::Insert(0), Change::Update(0)]);
        assert!(c.take_changes().is_empty());
        c.on_event(1, Event::Title("제목".into()));
        assert_eq!(c.take_changes(), [Change::Update(0)]);
        c.on_event(1, Event::Expanded { title: "p".into(), entries: entries(2), skipped: 0 });
        let changes = c.take_changes();
        assert_eq!(changes[..3], [Change::Update(0), Change::Insert(1), Change::Insert(2)]);
    }
}
