use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::lyrics::LyricsDoc;
use crate::parse::Track;

/// 队列只用于恢复现场，不保存播放地址；超长队列截断，避免配置文件无限膨胀。
const QUEUE_CAP: usize = 500;
const LYRIC_TTL_SECS: i64 = 90 * 24 * 3600;
const PLAY_MODES: [&str; 4] = ["order", "repeat-all", "repeat-one", "shuffle"];
const QUALITIES: [&str; 2] = ["standard", "lossless"];

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct PersistedState {
    pub volume: f64,
    pub mode: String,
    pub quality: String,
    pub queue_open: bool,
    /// 关闭窗口时是否隐藏到托盘而不是退出。
    pub close_to_tray: bool,
    pub cursor: i64,
    pub queue: Vec<Track>,
}

impl Default for PersistedState {
    fn default() -> Self {
        Self {
            volume: 0.8,
            mode: "repeat-all".into(),
            quality: "standard".into(),
            queue_open: false,
            close_to_tray: true,
            cursor: -1,
            queue: Vec::new(),
        }
    }
}

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("state.json"))
}

/// 读不到、解析失败、字段越界都退回默认值，不让一份坏配置把应用卡在启动阶段。
pub fn sanitize(mut state: PersistedState) -> PersistedState {
    state.volume = if state.volume.is_finite() {
        state.volume.clamp(0.0, 1.0)
    } else {
        0.8
    };
    if !PLAY_MODES.contains(&state.mode.as_str()) {
        state.mode = "repeat-all".into();
    }
    if !QUALITIES.contains(&state.quality.as_str()) {
        state.quality = "standard".into();
    }
    state.queue.retain(|track| !track.id.trim().is_empty() && !track.title.trim().is_empty());
    state.queue.truncate(QUEUE_CAP);
    if state.cursor >= state.queue.len() as i64 {
        state.cursor = state.queue.len() as i64 - 1;
    }
    state
}

pub fn load_from(path: &std::path::Path) -> PersistedState {
    let Ok(raw) = fs::read_to_string(path) else {
        return PersistedState::default();
    };
    sanitize(serde_json::from_str(&raw).unwrap_or_default())
}

/// 先写临时文件再改名，写到一半断电也不会把原配置截断成半截 JSON。
pub fn save_to(path: &std::path::Path, state: &PersistedState) -> Result<(), String> {
    let mut data = state.clone();
    data.queue.truncate(QUEUE_CAP);
    let payload = serde_json::to_vec(&data).map_err(|e| e.to_string())?;
    let temp = path.with_extension("json.tmp");
    fs::write(&temp, payload).map_err(|e| e.to_string())?;
    fs::rename(&temp, path).map_err(|e| e.to_string())
}

pub fn load_state(app: &AppHandle) -> PersistedState {
    match config_path(app) {
        Ok(path) => load_from(&path),
        Err(_) => PersistedState::default(),
    }
}

pub fn save_state(app: &AppHandle, state: &PersistedState) -> Result<(), String> {
    save_to(&config_path(app)?, state)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LyricCacheEntry {
    saved_at: i64,
    doc: LyricsDoc,
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn lyric_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_cache_dir().map_err(|e| e.to_string())?.join("lyrics");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// 曲目 ID 是不透明字符串，直接当文件名会踩到非法字符和超长路径，先哈希。
fn lyric_file(dir: &PathBuf, track_id: &str) -> PathBuf {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in track_id.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    dir.join(format!("{hash:016x}.json"))
}

pub fn read_lyrics(app: &AppHandle, track_id: &str) -> Option<LyricsDoc> {
    let dir = lyric_dir(app).ok()?;
    let raw = fs::read_to_string(lyric_file(&dir, track_id)).ok()?;
    let entry: LyricCacheEntry = serde_json::from_str(&raw).ok()?;
    if now_secs() - entry.saved_at > LYRIC_TTL_SECS {
        let _ = fs::remove_file(lyric_file(&dir, track_id));
        return None;
    }
    Some(entry.doc)
}

pub fn write_lyrics(app: &AppHandle, track_id: &str, doc: &LyricsDoc) {
    let Ok(dir) = lyric_dir(app) else { return };
    let entry = LyricCacheEntry { saved_at: now_secs(), doc: doc.clone() };
    if let Ok(payload) = serde_json::to_vec(&entry) {
        let _ = fs::write(lyric_file(&dir, track_id), payload);
    }
}

pub fn lyrics_cache_size(app: &AppHandle) -> u64 {
    let Ok(dir) = lyric_dir(app) else { return 0 };
    let Ok(entries) = fs::read_dir(dir) else { return 0 };
    entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .map(|meta| meta.len())
        .sum()
}

pub fn clear_lyrics_cache(app: &AppHandle) -> u64 {
    let freed = lyrics_cache_size(app);
    if let Ok(dir) = lyric_dir(app) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
    freed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::Artist;

    fn track(id: &str) -> Track {
        Track {
            id: id.into(),
            title: format!("歌 {id}"),
            artists: vec![Artist { id: Some("a1".into()), name: "艺人".into() }],
            album: None,
            cover_url: None,
            duration_seconds: Some(200.0),
            available: true,
            favorite: false,
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("music-state-{name}-{nanos}.json"))
    }

    #[test]
    fn round_trip_keeps_queue_and_cursor() {
        let path = temp_path("roundtrip");
        let saved = PersistedState {
            volume: 0.35,
            mode: "shuffle".into(),
            quality: "lossless".into(),
            queue_open: true,
            close_to_tray: false,
            cursor: 1,
            queue: vec![track("1"), track("2"), track("3")],
        };
        save_to(&path, &saved).expect("save");
        let loaded = load_from(&path);

        assert_eq!(loaded.queue.len(), 3);
        assert_eq!(loaded.queue[1].title, "歌 2");
        assert_eq!(loaded.queue[1].artists[0].name, "艺人");
        assert_eq!(loaded.cursor, 1);
        assert_eq!(loaded.mode, "shuffle");
        assert_eq!(loaded.quality, "lossless");
        assert!(loaded.queue_open);
        assert!(!loaded.close_to_tray);
        assert!((loaded.volume - 0.35).abs() < 1e-9);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn missing_file_falls_back_to_defaults() {
        let loaded = load_from(&temp_path("missing"));
        assert_eq!(loaded.cursor, -1);
        assert!(loaded.queue.is_empty());
        assert_eq!(loaded.mode, "repeat-all");
    }

    #[test]
    fn broken_json_falls_back_to_defaults() {
        let path = temp_path("broken");
        fs::write(&path, b"{ this is not json").unwrap();
        let loaded = load_from(&path);
        assert!(loaded.queue.is_empty());
        assert_eq!(loaded.quality, "standard");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn out_of_range_values_are_clamped() {
        let path = temp_path("clamp");
        let saved = PersistedState {
            volume: 4.2,
            mode: "bogus".into(),
            quality: "hires".into(),
            queue_open: false,
            close_to_tray: true,
            cursor: 99,
            queue: vec![track("1"), track("2")],
        };
        save_to(&path, &saved).expect("save");
        let loaded = load_from(&path);

        assert!((loaded.volume - 1.0).abs() < 1e-9);
        assert_eq!(loaded.mode, "repeat-all");
        assert_eq!(loaded.quality, "standard");
        assert_eq!(loaded.cursor, 1);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn empty_queue_resets_cursor() {
        let path = temp_path("empty");
        let saved = PersistedState { cursor: 3, queue: Vec::new(), ..Default::default() };
        save_to(&path, &saved).expect("save");
        assert_eq!(load_from(&path).cursor, -1);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn tracks_without_id_or_title_are_dropped() {
        let path = temp_path("blank");
        let mut blank = track("x");
        blank.title = "  ".into();
        let saved = PersistedState { cursor: 0, queue: vec![track("1"), blank], ..Default::default() };
        save_to(&path, &saved).expect("save");
        let loaded = load_from(&path);
        assert_eq!(loaded.queue.len(), 1);
        assert_eq!(loaded.queue[0].id, "1");
        let _ = fs::remove_file(&path);
    }
}
