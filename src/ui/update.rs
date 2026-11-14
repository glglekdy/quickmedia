//! 업데이트 화면이 무엇을 보일지. 앱 업데이트와 yt-dlp 갱신 두 영역.

use crate::updater::{note_lines, NoteLine, Release};

/// 새 버전을 찾은 뒤의 단계.
#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    /// "지금 업데이트" 를 기다린다.
    Ready,
    /// 설치 파일을 받는 중. 퍼센트를 알면 `Some`.
    Downloading(Option<u8>),
    /// 받기나 적용에 실패했다. 화면에 보일 이유.
    Failed(String),
    /// 도우미를 띄웠고 곧 앱이 꺼진다.
    Restarting,
}

/// 앱 업데이트 영역의 상태.
#[derive(Clone, Debug, PartialEq)]
pub enum AppUpdate {
    Idle,
    Checking,
    /// 최신이다. 확인한 시각(`21:05`).
    UpToDate { at: String },
    CheckFailed(String),
    Available { release: Release, phase: Phase },
}

/// 버튼을 누르면 할 일.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Check,
    Download,
    Nothing,
}

impl AppUpdate {
    /// 메뉴 '업데이트' 옆 빨간 점: 새 버전이 기다리는 동안.
    pub fn has_dot(&self) -> bool {
        matches!(self, AppUpdate::Available { .. })
    }

    pub fn action(&self) -> Action {
        match self {
            AppUpdate::Idle | AppUpdate::UpToDate { .. } | AppUpdate::CheckFailed(_) => Action::Check,
            AppUpdate::Available { phase: Phase::Ready | Phase::Failed(_), .. } => Action::Download,
            _ => Action::Nothing,
        }
    }
}

/// 업데이트하면 앱이 꺼지므로 받고 있거나 기다리는 영상이 있으면 묻는다.
pub fn confirm_question(active_jobs: usize) -> Option<String> {
    (active_jobs > 0).then(|| crate::trf!("다운로드 {active_jobs}개가 중단됩니다. 업데이트할까요?", "{active_jobs} downloads will stop. Update now?"))
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UpdateView {
    pub title: String,
    /// 제목 아래 한 줄. 없으면 빈 글.
    pub sub: String,
    pub sub_error: bool,
    pub button: String,
    pub enabled: bool,
    /// 강조(빨간) 버튼.
    pub primary: bool,
    /// 버튼 옆 안내.
    pub status: String,
    pub status_error: bool,
    /// 받는 중이면 0~1. 아니면 막대를 숨긴다.
    pub progress: Option<f32>,
    /// 릴리즈 설명. 비어 있으면 숨긴다.
    pub notes: Vec<NoteLine>,
}

pub fn app_view(state: &AppUpdate, can_self_update: bool) -> UpdateView {
    let base = |title: &str, sub: &str, button: &str| UpdateView {
        title: title.to_string(),
        sub: sub.to_string(),
        button: button.to_string(),
        enabled: true,
        ..Default::default()
    };
    match state {
        AppUpdate::Idle => base(crate::lang::tr("확인 전", "Not checked yet"), "", crate::lang::tr("업데이트 확인", "Check for updates")),
        AppUpdate::Checking => UpdateView { enabled: false, ..base(crate::lang::tr("확인하는 중…", "Checking…"), crate::lang::tr("최신 버전 확인 중", "Looking for the latest version"), crate::lang::tr("업데이트 확인", "Check for updates")) },
        AppUpdate::UpToDate { at } => base(crate::lang::tr("최신 버전", "Up to date"), &crate::trf!("{at} 확인", "Checked {at}"), crate::lang::tr("업데이트 확인", "Check for updates")),
        AppUpdate::CheckFailed(message) => UpdateView { sub_error: true, ..base(crate::lang::tr("확인 실패", "Check failed"), message, crate::lang::tr("다시 확인", "Check again")) },
        AppUpdate::Available { release, phase } => {
            let mut view = UpdateView {
                primary: true,
                notes: if release.notes.trim().is_empty() { Vec::new() } else { note_lines(&release.notes) },
                ..base(&crate::trf!("새 버전 {}", "New version {}", release.version), "", crate::lang::tr("지금 업데이트", "Update now"))
            };
            match phase {
                Phase::Ready if can_self_update => {
                    view.status = crate::lang::tr("설치 후 QuickMedia가 다시 시작됨", "QuickMedia restarts after installing").to_string();
                }
                Phase::Ready => {
                    view.enabled = false;
                    view.status = crate::lang::tr("개발용 빌드에서는 자동 업데이트 불가", "Dev builds can't update themselves").to_string();
                }
                Phase::Downloading(percent) => {
                    view.enabled = false;
                    view.status = percent.map_or(crate::lang::tr("받는 중…", "Downloading…").to_string(), |p| crate::trf!("받는 중… {p}%", "Downloading… {p}%"));
                    view.progress = Some(percent.map_or(0.0, |p| f32::from(p) / 100.0));
                }
                Phase::Failed(message) => {
                    view.button = crate::lang::tr("다시 시도", "Try again").to_string();
                    view.status = message.clone();
                    view.status_error = true;
                }
                Phase::Restarting => {
                    view.enabled = false;
                    view.status = crate::lang::tr("다시 시작하는 중…", "Restarting…").to_string();
                }
            }
            view
        }
    }
}

/// yt-dlp 갱신 영역의 상태.
#[derive(Clone, Debug, PartialEq)]
pub enum YtdlpUpdate {
    Idle,
    Checking,
    /// 받는 중(퍼센트).
    Downloading(u8),
    UpToDate,
    /// 이 버전으로 갱신했다.
    Updated(String),
    Failed(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct YtdlpView {
    /// 지금 쓰는 버전.
    pub version: String,
    pub status: String,
    pub status_error: bool,
    /// "갱신" 버튼을 누를 수 있는지.
    pub enabled: bool,
}

pub fn ytdlp_view(current: Option<&str>, state: &YtdlpUpdate) -> YtdlpView {
    let status = match state {
        YtdlpUpdate::Idle => String::new(),
        YtdlpUpdate::Checking => crate::lang::tr("확인하는 중…", "Checking…").to_string(),
        YtdlpUpdate::Downloading(percent) => crate::trf!("받는 중… {percent}%", "Downloading… {percent}%"),
        YtdlpUpdate::UpToDate => crate::lang::tr("최신", "Up to date").to_string(),
        YtdlpUpdate::Updated(version) => crate::trf!("{version} 갱신 완료", "Updated to {version}"),
        YtdlpUpdate::Failed(message) => message.clone(),
    };
    YtdlpView {
        version: current.unwrap_or(crate::lang::tr("찾을 수 없음", "Not found")).to_string(),
        status,
        status_error: matches!(state, YtdlpUpdate::Failed(_)),
        enabled: !matches!(state, YtdlpUpdate::Checking | YtdlpUpdate::Downloading(_)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(notes: &str) -> Release {
        Release { version: "2.1.0".into(), notes: notes.into(), asset_url: "u".into(), asset_size: 1, sha256: String::new() }
    }

    fn available(phase: Phase) -> AppUpdate {
        AppUpdate::Available { release: release("## 고친 점\n- 하나"), phase }
    }

    #[test]
    fn idle_checking_latest_failed() {
        let v = app_view(&AppUpdate::Idle, true);
        assert_eq!((v.title.as_str(), v.sub.as_str(), v.button.as_str(), v.enabled, v.primary), ("확인 전", "", "업데이트 확인", true, false));
        assert!(v.status.is_empty() && v.progress.is_none() && v.notes.is_empty() && !v.sub_error);

        let v = app_view(&AppUpdate::Checking, true);
        assert_eq!((v.title.as_str(), v.sub.as_str(), v.button.as_str(), v.enabled), ("확인하는 중…", "최신 버전 확인 중", "업데이트 확인", false));

        let v = app_view(&AppUpdate::UpToDate { at: "21:05".into() }, true);
        assert_eq!((v.title.as_str(), v.sub.as_str(), v.button.as_str(), v.enabled), ("최신 버전", "21:05 확인", "업데이트 확인", true));

        let v = app_view(&AppUpdate::CheckFailed("GitHub 연결 실패".into()), true);
        assert_eq!((v.title.as_str(), v.sub.as_str(), v.button.as_str(), v.enabled, v.sub_error),
                   ("확인 실패", "GitHub 연결 실패", "다시 확인", true, true));
    }

    #[test]
    fn available_offers_update_with_notes() {
        let v = app_view(&available(Phase::Ready), true);
        assert_eq!((v.title.as_str(), v.sub.as_str(), v.button.as_str(), v.enabled, v.primary), ("새 버전 2.1.0", "", "지금 업데이트", true, true));
        assert_eq!(v.status, "설치 후 QuickMedia가 다시 시작됨");
        assert!(!v.status_error && v.progress.is_none());
        assert_eq!(v.notes, [NoteLine { text: "고친 점".into(), heading: true }, NoteLine { text: "• 하나".into(), heading: false }]);
    }

    #[test]
    fn dev_build_cannot_update_itself() {
        let v = app_view(&available(Phase::Ready), false);
        assert_eq!((v.button.as_str(), v.enabled, v.primary), ("지금 업데이트", false, true));
        assert_eq!(v.status, "개발용 빌드에서는 자동 업데이트 불가");
    }

    #[test]
    fn downloading_failed_restarting() {
        let v = app_view(&available(Phase::Downloading(None)), true);
        assert_eq!((v.status.as_str(), v.enabled, v.progress), ("받는 중…", false, Some(0.0)));
        let v = app_view(&available(Phase::Downloading(Some(40))), true);
        assert_eq!((v.status.as_str(), v.enabled, v.progress), ("받는 중… 40%", false, Some(0.4)));

        let v = app_view(&available(Phase::Failed("파일 손상 (SHA-256 불일치)".into())), true);
        assert_eq!((v.button.as_str(), v.enabled, v.primary, v.status_error), ("다시 시도", true, true, true));
        assert_eq!(v.status, "파일 손상 (SHA-256 불일치)");

        let v = app_view(&available(Phase::Restarting), true);
        assert_eq!((v.button.as_str(), v.enabled, v.status.as_str()), ("지금 업데이트", false, "다시 시작하는 중…"));
        assert!(!v.notes.is_empty());
    }

    #[test]
    fn blank_notes_are_hidden() {
        let v = app_view(&AppUpdate::Available { release: release("  \n"), phase: Phase::Ready }, true);
        assert!(v.notes.is_empty());
    }

    #[test]
    fn dot_only_while_a_new_version_waits() {
        for state in [AppUpdate::Idle, AppUpdate::Checking, AppUpdate::UpToDate { at: "1".into() }, AppUpdate::CheckFailed("x".into())] {
            assert!(!state.has_dot());
        }
        for phase in [Phase::Ready, Phase::Downloading(None), Phase::Failed("x".into()), Phase::Restarting] {
            assert!(available(phase).has_dot());
        }
    }

    #[test]
    fn action_depends_on_state() {
        assert_eq!(AppUpdate::Idle.action(), Action::Check);
        assert_eq!(AppUpdate::UpToDate { at: "1".into() }.action(), Action::Check);
        assert_eq!(AppUpdate::CheckFailed("x".into()).action(), Action::Check);
        assert_eq!(AppUpdate::Checking.action(), Action::Nothing);
        assert_eq!(available(Phase::Ready).action(), Action::Download);
        assert_eq!(available(Phase::Failed("x".into())).action(), Action::Download);
        assert_eq!(available(Phase::Downloading(Some(3))).action(), Action::Nothing);
        assert_eq!(available(Phase::Restarting).action(), Action::Nothing);
    }

    #[test]
    fn confirm_text_counts_jobs() {
        assert_eq!(confirm_question(0), None);
        assert_eq!(confirm_question(3).as_deref(), Some("다운로드 3개가 중단됩니다. 업데이트할까요?"));
    }

    #[test]
    fn ytdlp_views() {
        assert_eq!(ytdlp_view(Some("2026.10.01"), &YtdlpUpdate::Idle).version, "2026.10.01");
        assert_eq!(ytdlp_view(None, &YtdlpUpdate::Idle).version, "찾을 수 없음");
        assert_eq!(ytdlp_view(Some("x"), &YtdlpUpdate::Idle).status, "");
        assert_eq!(ytdlp_view(Some("x"), &YtdlpUpdate::Checking).status, "확인하는 중…");
        assert_eq!(ytdlp_view(Some("x"), &YtdlpUpdate::Downloading(40)).status, "받는 중… 40%");
        assert_eq!(ytdlp_view(Some("x"), &YtdlpUpdate::UpToDate).status, "최신");
        assert_eq!(ytdlp_view(Some("x"), &YtdlpUpdate::Updated("2026.11.05".into())).status, "2026.11.05 갱신 완료");
        assert!(!ytdlp_view(Some("x"), &YtdlpUpdate::Downloading(1)).enabled);
        assert!(!ytdlp_view(Some("x"), &YtdlpUpdate::Checking).enabled);
        assert!(ytdlp_view(Some("x"), &YtdlpUpdate::UpToDate).enabled && ytdlp_view(None, &YtdlpUpdate::Idle).enabled);
        let failed = ytdlp_view(Some("x"), &YtdlpUpdate::Failed("GitHub 연결 실패".into()));
        assert!(failed.status_error && failed.enabled);
        assert_eq!(failed.status, "GitHub 연결 실패");
        assert!(!ytdlp_view(Some("x"), &YtdlpUpdate::UpToDate).status_error);
    }
}
