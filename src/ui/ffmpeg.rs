//! 변환 도구(ffmpeg)를 준비하는 동안 화면 구석에 떠 있는 알림.

/// ffmpeg 준비 상태.
#[derive(Clone, Debug, PartialEq)]
pub enum FfmpegState {
    /// PC 에 있는지 찾는 중. 금방 끝나서 아무것도 보이지 않는다.
    Looking,
    /// 받는 중(퍼센트). 크기를 아직 모르면 `None`.
    Fetching(Option<u8>),
    Ready,
    Failed(String),
}

impl FfmpegState {
    /// 다운로드를 시작하지 않고 붙잡아 두어야 하는지.
    pub fn holds_jobs(&self) -> bool {
        matches!(self, FfmpegState::Looking | FfmpegState::Fetching(_))
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Banner {
    /// 빈 글이면 알림을 보이지 않는다.
    pub text: String,
    /// 제목 아래 작은 글.
    pub detail: String,
    /// "43%". 모르면 빈 글.
    pub percent: String,
    pub error: bool,
    /// 0~1. 채워지는 바탕을 보이지 않으면 `None`.
    pub progress: Option<f32>,
    /// "다시 시도" 버튼.
    pub retry: bool,
}

pub fn banner(state: &FfmpegState) -> Banner {
    match state {
        FfmpegState::Looking | FfmpegState::Ready => Banner::default(),
        FfmpegState::Fetching(percent) => Banner {
            text: "ffmpeg 세팅중...".to_string(),
            percent: percent.map(|p| format!("{}%", p.min(100))).unwrap_or_default(),
            progress: Some(percent.map_or(0.0, |p| f32::from(p.min(100)) / 100.0)),
            ..Banner::default()
        },
        FfmpegState::Failed(message) => {
            Banner { text: "ffmpeg 세팅 실패".to_string(), detail: message.clone(), error: true, retry: true, ..Banner::default() }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_shown_when_ready_or_still_looking() {
        assert_eq!(banner(&FfmpegState::Ready), Banner::default());
        assert_eq!(banner(&FfmpegState::Looking), Banner::default());
    }

    #[test]
    fn fetching_shows_percent_and_fill() {
        let b = banner(&FfmpegState::Fetching(Some(43)));
        assert_eq!((b.text.as_str(), b.percent.as_str(), b.detail.as_str()), ("ffmpeg 세팅중...", "43%", ""));
        assert_eq!((b.progress, b.error, b.retry), (Some(0.43), false, false));
        let b = banner(&FfmpegState::Fetching(None));
        assert_eq!((b.text.as_str(), b.percent.as_str(), b.progress), ("ffmpeg 세팅중...", "", Some(0.0)));
    }

    #[test]
    fn failure_offers_retry() {
        let b = banner(&FfmpegState::Failed("변환 도구를 받다가 끊겼어요. 다시 시도해 주세요.".into()));
        assert_eq!(b.text, "ffmpeg 세팅 실패");
        assert_eq!(b.detail, "변환 도구를 받다가 끊겼어요. 다시 시도해 주세요.");
        assert_eq!((b.error, b.retry, b.progress, b.percent.as_str()), (true, true, None, ""));
    }

    #[test]
    fn jobs_wait_only_while_ffmpeg_may_still_arrive() {
        assert!(FfmpegState::Looking.holds_jobs());
        assert!(FfmpegState::Fetching(None).holds_jobs());
        assert!(!FfmpegState::Ready.holds_jobs());
        // 받지 못했으면 그냥 시작한다. ffmpeg 가 필요 없는 형식은 받아진다.
        assert!(!FfmpegState::Failed("x".into()).holds_jobs());
    }
}
