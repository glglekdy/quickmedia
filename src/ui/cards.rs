//! 다운로드 카드 한 장의 상태와 문구. 사이트 · 제목 · 상태 · 진행률.

use crate::site::{display_name, domain_of};
use crate::ytdlp::Event;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CardState {
    /// 대기열에서 차례를 기다린다.
    Waiting,
    /// 시작했고 영상 정보를 알아보는 중.
    Loading,
    Running,
    Done,
    Error,
    Skipped,
    Cancelled,
    /// 재생목록을 항목으로 펼치고 끝났다.
    Playlist,
}

/// "끝난 항목 지우기"로 지워도 되는 상태.
pub fn is_finished(state: CardState) -> bool {
    !matches!(state, CardState::Waiting | CardState::Loading | CardState::Running)
}

pub fn playlist_status(added: usize, skipped: u32) -> String {
    let mut text = format!("영상 {added}개 추가");
    if skipped > 0 {
        text.push_str(&format!(" · {skipped}개는 이미 받아서 건너뜀"));
    }
    text
}

#[derive(Clone, Debug, PartialEq)]
pub struct Card {
    pub id: u32,
    pub url: String,
    /// 받는 형식 요약. 예: `1080p · MKV`
    pub format: String,
    /// 사이트 이름. 알기 전에는 호스트 이름.
    pub site: String,
    /// 아이콘을 찾을 호스트 이름.
    pub host: String,
    /// 알기 전에는 링크.
    pub title: String,
    pub state: CardState,
    pub status: String,
    pub percent: f32,
    /// 받은 파일 경로.
    pub path: String,
}

impl Card {
    pub fn new(id: u32, url: &str, format: &str) -> Self {
        let host = domain_of(url);
        Self {
            id,
            url: url.to_string(),
            format: format.to_string(),
            site: host.clone(),
            host,
            title: url.to_string(),
            state: CardState::Waiting,
            status: format!("{format} · 대기 중"),
            percent: 0.0,
            path: String::new(),
        }
    }

    /// 진행률 막대를 보일지. 받지 않고 끝난 카드는 숨긴다.
    pub fn show_bar(&self) -> bool {
        !matches!(self.state, CardState::Error | CardState::Skipped | CardState::Cancelled | CardState::Playlist)
    }

    /// 제목 옆 퍼센트. 받는 동안만 보인다.
    pub fn percent_text(&self) -> String {
        match self.state {
            CardState::Loading | CardState::Running => format!("{:.0}%", self.percent),
            _ => String::new(),
        }
    }

    pub fn set_started(&mut self) {
        self.state = CardState::Loading;
        self.percent = 0.0;
        self.status = format!("{} · 영상 정보 불러오는 중…", self.format);
    }

    pub fn set_cancelled(&mut self) {
        self.state = CardState::Cancelled;
        self.status = "취소했어요".to_string();
    }

    pub fn set_playlist(&mut self, title: &str, added: usize, skipped: u32) {
        self.title = format!("재생목록 · {title}");
        self.state = CardState::Playlist;
        self.status = playlist_status(added, skipped);
    }
}

/// 작업이 알려 온 것을 카드에 반영한다. 재생목록 펼치기는 항목 카드가 필요해 `Controller` 가 한다.
pub fn apply_event(card: &mut Card, event: &Event) {
    match event {
        Event::Site { key, url } => {
            let host = domain_of(url);
            if !host.is_empty() {
                card.host = host;
            }
            card.site = display_name(key, &card.host);
        }
        Event::Title(title) => card.title = title.clone(),
        Event::Progress(percent, text) => {
            card.state = CardState::Running;
            card.percent = *percent;
            card.status = format!("{} · {text}", card.format);
        }
        Event::Done(path) => {
            card.state = CardState::Done;
            card.percent = 100.0;
            card.path = path.clone();
            card.status = format!("{} · 완료", card.format);
        }
        Event::Failed(message) => {
            card.state = CardState::Error;
            card.status = format!("실패 · {message}");
        }
        Event::Skipped => {
            card.state = CardState::Skipped;
            card.status = "이미 받은 영상이라 건너뛰었어요".to_string();
        }
        Event::Expanded { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card() -> Card {
        Card::new(7, "https://www.youtube.com/watch?v=x", "최고 화질 · MP4")
    }

    #[test]
    fn new_card_waits_and_shows_the_link() {
        let c = card();
        assert_eq!((c.id, c.title.as_str(), c.host.as_str(), c.site.as_str()), (7, "https://www.youtube.com/watch?v=x", "youtube.com", "youtube.com"));
        assert_eq!((c.state, c.status.as_str(), c.percent), (CardState::Waiting, "최고 화질 · MP4 · 대기 중", 0.0));
        assert!(c.show_bar() && c.percent_text().is_empty() && !is_finished(c.state));
    }

    #[test]
    fn started_card_loads_info() {
        let mut c = card();
        c.set_started();
        assert_eq!((c.state, c.status.as_str()), (CardState::Loading, "최고 화질 · MP4 · 영상 정보 불러오는 중…"));
        assert_eq!(c.percent_text(), "0%");
    }

    #[test]
    fn site_and_title_events() {
        let mut c = card();
        apply_event(&mut c, &Event::Site { key: "Youtube".into(), url: "https://m.youtube.com/watch?v=x".into() });
        assert_eq!((c.site.as_str(), c.host.as_str()), ("YouTube", "youtube.com"));
        // 주소가 다른 곳으로 넘어갔으면 아이콘도 그 사이트 것을 쓴다
        apply_event(&mut c, &Event::Site { key: "Generic".into(), url: "https://cdn.example.com/v".into() });
        assert_eq!((c.site.as_str(), c.host.as_str()), ("cdn.example.com", "cdn.example.com"));
        // 주소를 못 읽으면 호스트는 그대로
        apply_event(&mut c, &Event::Site { key: "Vimeo".into(), url: String::new() });
        assert_eq!((c.site.as_str(), c.host.as_str()), ("Vimeo", "cdn.example.com"));
        apply_event(&mut c, &Event::Title("제목".into()));
        assert_eq!(c.title, "제목");
    }

    #[test]
    fn progress_done_and_failure() {
        let mut c = card();
        apply_event(&mut c, &Event::Progress(41.6, "받는 중 · 3MiB/s".into()));
        assert_eq!((c.state, c.status.as_str(), c.percent_text().as_str()), (CardState::Running, "최고 화질 · MP4 · 받는 중 · 3MiB/s", "42%"));
        apply_event(&mut c, &Event::Done("D:\\v\\a.mp4".into()));
        assert_eq!((c.state, c.status.as_str(), c.percent, c.path.as_str()), (CardState::Done, "최고 화질 · MP4 · 완료", 100.0, "D:\\v\\a.mp4"));
        assert!(c.show_bar() && c.percent_text().is_empty() && is_finished(c.state));

        let mut c = card();
        apply_event(&mut c, &Event::Failed("지원하지 않는 링크예요".into()));
        assert_eq!((c.state, c.status.as_str()), (CardState::Error, "실패 · 지원하지 않는 링크예요"));
        assert!(!c.show_bar() && c.percent_text().is_empty() && is_finished(c.state));
    }

    #[test]
    fn quiet_endings() {
        let mut c = card();
        apply_event(&mut c, &Event::Skipped);
        assert_eq!((c.state, c.status.as_str(), c.show_bar()), (CardState::Skipped, "이미 받은 영상이라 건너뛰었어요", false));
        let mut c = card();
        c.set_cancelled();
        assert_eq!((c.state, c.status.as_str(), c.show_bar()), (CardState::Cancelled, "취소했어요", false));
        let mut c = card();
        c.set_playlist("모음", 12, 0);
        assert_eq!((c.state, c.title.as_str(), c.status.as_str(), c.show_bar()), (CardState::Playlist, "재생목록 · 모음", "영상 12개 추가", false));
        assert!(is_finished(CardState::Skipped) && is_finished(CardState::Cancelled) && is_finished(CardState::Playlist));
        assert!(!is_finished(CardState::Loading) && !is_finished(CardState::Running));
    }

    #[test]
    fn playlist_status_mentions_skipped() {
        assert_eq!(playlist_status(3, 0), "영상 3개 추가");
        assert_eq!(playlist_status(3, 2), "영상 3개 추가 · 2개는 이미 받아서 건너뜀");
        assert_eq!(playlist_status(0, 5), "영상 0개 추가 · 5개는 이미 받아서 건너뜀");
    }
}
