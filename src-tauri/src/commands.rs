use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, State};

use crate::api::{AppError, Client};
use crate::auth::Session;
use crate::lyrics::{self, LyricsDoc};
use crate::parse::{self, AlbumMeta, ArtistMeta, Playlist, Track};
use crate::state::{self, PersistedState};
use crate::{login, store};

pub struct AppState {
    pub client: Client,
    pub session: Mutex<Option<Session>>,
    /// 关闭窗口时是否隐藏到托盘，由设置页写入；托盘与窗口事件直接读这里。
    pub close_to_tray: AtomicBool,
}

impl AppState {
    pub fn session(&self) -> Result<Session, AppError> {
        let guard = self.session.lock().map_err(|_| AppError::auth("会话状态异常"))?;
        guard.clone().ok_or_else(|| AppError::auth("尚未登录"))
    }

    fn set_session(&self, session: Option<Session>) {
        if let Ok(mut guard) = self.session.lock() {
            *guard = session;
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    authenticated: bool,
    account_label: Option<String>,
    music_expire_time: Option<String>,
}

fn status_of(session: Option<&Session>) -> AuthStatus {
    match session {
        Some(session) => AuthStatus {
            authenticated: true,
            account_label: session.account_label(),
            music_expire_time: session.expire_time.clone(),
        },
        None => AuthStatus { authenticated: false, account_label: None, music_expire_time: None },
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    items: Vec<Track>,
    next_start: i64,
    has_more: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Playback {
    url: String,
    actual_quality: String,
    content_type: Option<String>,
    duration_seconds: Option<f64>,
}

#[tauri::command]
pub async fn auth_status(state: State<'_, AppState>) -> Result<AuthStatus, AppError> {
    let session = state.session.lock().map_err(|_| AppError::auth("会话状态异常"))?.clone();
    Ok(status_of(session.as_ref()))
}

#[tauri::command]
pub async fn login(app: AppHandle, state: State<'_, AppState>) -> Result<AuthStatus, AppError> {
    let session = login::run_login(&app, &state.client).await?;
    // 系统密钥环不可用时仍保留本次会话，只是重启后需要重新登录。
    if let Err(message) = store::save_session(&session) {
        eprintln!("[login] {message}");
    }
    let status = status_of(Some(&session));
    state.set_session(Some(session));
    Ok(status)
}

#[tauri::command]
pub async fn set_close_to_tray(state: State<'_, AppState>, enabled: bool) -> Result<(), AppError> {
    state.close_to_tray.store(enabled, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), AppError> {
    store::clear_session();
    state.set_session(None);
    Ok(())
}

#[tauri::command]
pub async fn favorites(app: AppHandle, state: State<'_, AppState>, start: i64, limit: i64) -> Result<Page, AppError> {
    let session = state.session()?;
    let limit = limit.clamp(1, 100);
    let body = state.client.favorites(&session, start.max(0), limit).await?;
    let raw = parse::array_at(&body, "favoriteInfos");
    // 只在第一页打一次，确认时长与收藏标记在当前账号下是否解析成功。
    if start <= 0 {
        if let Some(first) = raw.as_array().and_then(|items| items.first()) {
            let keys: Vec<&str> = first
                .as_object()
                .map(|map| map.keys().map(|k| k.as_str()).collect())
                .unwrap_or_default();
            lyrics_debug(&app, &format!("[favorites] 首条字段={keys:?}"));
        }
    }
    let count = raw.as_array().map(|a| a.len()).unwrap_or(0) as i64;
    let items = parse::tracks_from(raw);
    if start <= 0 {
        if let Some(first) = items.first() {
            lyrics_debug(
                &app,
                &format!("[favorites] 首条解析 时长={:?} 收藏={}", first.duration_seconds, first.favorite),
            );
        }
    }
    Ok(Page {
        items,
        next_start: start.max(0) + limit,
        has_more: count == limit,
    })
}

#[tauri::command]
pub async fn search(
    state: State<'_, AppState>,
    query: String,
    start: i64,
    limit: i64,
) -> Result<Page, AppError> {
    let keyword = query.trim();
    if keyword.is_empty() {
        return Err(AppError::invalid("搜索词不能为空"));
    }
    let limit = limit.clamp(1, 50);
    let body = state
        .client
        .search(&keyword.chars().take(80).collect::<String>(), start.max(0), limit)
        .await?;
    let raw = parse::array_at(&body, "songSimpleInfos");
    let count = raw.as_array().map(|a| a.len()).unwrap_or(0) as i64;
    Ok(Page {
        items: parse::tracks_from(raw),
        next_start: start.max(0) + limit,
        has_more: body
            .get("hasNextPage")
            .and_then(|v| v.as_bool().or_else(|| v.as_str().map(|s| s == "1")))
            .unwrap_or(count == limit),
    })
}

#[tauri::command]
pub async fn playback(
    state: State<'_, AppState>,
    track_id: String,
    quality: String,
) -> Result<Playback, AppError> {
    let session = state.session()?;
    let body = state.client.playback(&session, &track_id, &quality).await?;
    let url = body
        .get("fileURL")
        .and_then(|v| v.as_str())
        .filter(|u| !u.is_empty())
        .ok_or_else(|| AppError::invalid("服务端未返回音频地址"))?;
    if !parse::is_safe_resource_url(url) {
        return Err(AppError::invalid("服务端返回了不在白名单内的音频地址"));
    }
    let actual_quality = match body.get("playQuality").and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string()))) {
        Some(q) if q == "1" => "standard",
        Some(q) if q == "3" => "lossless",
        _ => "unknown",
    };
    Ok(Playback {
        url: url.to_string(),
        actual_quality: actual_quality.to_string(),
        content_type: body.get("type").and_then(|v| v.as_str()).map(|s| s.to_string()),
        duration_seconds: body
            .get("songInfo")
            .and_then(|s| s.get("duration"))
            .and_then(|d| d.as_f64().or_else(|| d.as_str().and_then(|s| s.parse().ok()))),
    })
}

#[tauri::command]
pub async fn favorite_exist(state: State<'_, AppState>, track_id: String) -> Result<bool, AppError> {
    let session = state.session()?;
    state.client.favorite_exist(&session, &track_id).await
}

#[tauri::command]
pub async fn set_favorite(
    state: State<'_, AppState>,
    track_id: String,
    favorite: bool,
) -> Result<(), AppError> {
    let session = state.session()?;
    state.client.set_favorite(&session, &track_id, favorite).await
}

#[tauri::command]
pub async fn recent(state: State<'_, AppState>) -> Result<Vec<Track>, AppError> {
    let session = state.session()?;
    let body = state.client.recent(&session).await?;
    Ok(parse::tracks_from(parse::array_at(&body, "contentSimpleInfos")))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistPage {
    playlist: Option<Playlist>,
    items: Vec<Track>,
    next_start: i64,
    has_more: bool,
}

#[tauri::command]
pub async fn playlists(state: State<'_, AppState>) -> Result<Vec<Playlist>, AppError> {
    let session = state.session()?;
    // 发现接口把歌曲和歌单混在一页，翻到短页或没有新条目就停，最多 5 页。
    let mut collected: Vec<serde_json::Value> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    const PAGE: i64 = 200;
    for page in 0..5 {
        let body = state.client.library(&session, page * PAGE, PAGE).await?;
        let items = parse::array_at(&body, "favoriteInfos");
        let Some(entries) = items.as_array() else { break };
        let mut fresh = 0usize;
        for entry in entries {
            let key = format!(
                "{}:{}",
                parse::content_type_of(entry).unwrap_or_default(),
                entry.get("contentID").and_then(|v| v.as_str()).unwrap_or_default()
            );
            if seen.insert(key) {
                fresh += 1;
                collected.push(entry.clone());
            }
        }
        if (entries.len() as i64) < PAGE || fresh == 0 {
            break;
        }
    }
    Ok(parse::playlists_from_entries(&serde_json::Value::Array(collected)))
}

#[tauri::command]
pub async fn playlist_detail(
    state: State<'_, AppState>,
    playlist_id: String,
    kind: String,
    start: i64,
    limit: i64,
) -> Result<PlaylistPage, AppError> {
    let session = state.session()?;
    let limit = limit.clamp(1, 100);
    let start = start.max(0);
    let (playlist, items, item_count) = match kind.as_str() {
        "owned" => parse::owned_playlist_from(
            &state.client.playlist_detail(&session, &playlist_id, start, limit).await?,
        ),
        "collected" => parse::public_playlist_from(
            &state.client.public_playlist(&playlist_id, start, limit).await?,
        ),
        "chart" => return Err(AppError::invalid("暂不支持榜单详情")),
        _ => return Err(AppError::invalid("未知的歌单类型")),
    };
    if playlist.is_none() {
        return Err(AppError::invalid("歌单信息不完整，无法显示"));
    }
    Ok(PlaylistPage {
        playlist,
        items,
        next_start: start + limit,
        has_more: item_count as i64 == limit,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumPage {
    album: AlbumMeta,
    items: Vec<Track>,
    next_start: i64,
    has_more: bool,
}

#[tauri::command]
pub async fn album(
    state: State<'_, AppState>,
    album_id: String,
    start: i64,
    limit: i64,
) -> Result<AlbumPage, AppError> {
    let limit = limit.clamp(1, 100);
    let start = start.max(0);
    let (album, items, item_count) =
        parse::album_from(&state.client.album(&album_id, start, limit).await?);
    Ok(AlbumPage { album, items, next_start: start + limit, has_more: item_count as i64 == limit })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistPage {
    artist: ArtistMeta,
    items: Vec<Track>,
    albums: Vec<parse::AlbumCard>,
}

#[tauri::command]
pub async fn artist(state: State<'_, AppState>, artist_id: String) -> Result<ArtistPage, AppError> {
    // 歌手详情没有验证过分页，首版给有限列表，不做无限滚动。
    let (artist, items, albums) = parse::artist_from(&state.client.artist(&artist_id, 30, 20).await?);
    Ok(ArtistPage { artist, items, albums })
}

/// 仅 debug 构建：把歌词链路的判定过程写到缓存目录，方便定位"没歌词"到底卡在哪一步。
/// 只记录字段名、主机名和数量，不写任何响应内容或完整 URL。
#[cfg(debug_assertions)]
fn lyrics_debug(app: &AppHandle, message: &str) {
    use std::io::Write;
    use tauri::Manager;
    let Ok(dir) = app.path().app_cache_dir() else { return };
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("lyrics-debug.log"))
    {
        let _ = writeln!(file, "{message}");
    }
}

#[cfg(not(debug_assertions))]
fn lyrics_debug(_app: &AppHandle, _message: &str) {}

/// 只接受 contentID 完全一致的那一条，绝不按歌名模糊匹配，避免把别人的歌词贴上来。
fn lyrics_from_search(body: &serde_json::Value, track_id: &str) -> Option<(String, String)> {
    let items = body.get("songSimpleInfos")?.as_array()?;
    let hit = items
        .iter()
        .find(|item| item.get("contentID").and_then(|v| v.as_str()) == Some(track_id))?;
    Some((
        lyrics::lyric_address_from(hit).unwrap_or_default(),
        lyrics::inline_lyrics_from(hit).unwrap_or_default(),
    ))
}

/// 从一份响应里按 地址 → 内嵌文本 的顺序取歌词，取到就写缓存。
async fn resolve_lyrics(
    app: &AppHandle,
    state: &State<'_, AppState>,
    track_id: &str,
    body: &serde_json::Value,
) -> Option<LyricsDoc> {
    if let Some(address) = lyrics::lyric_address_from(body) {
        let host = url::Url::parse(&address)
            .ok()
            .and_then(|u| u.host_str().map(str::to_string))
            .unwrap_or_default();
        lyrics_debug(app, &format!("[{track_id}] 歌词地址 host={host}"));
        match state.client.fetch_text(&address, lyrics::max_bytes()).await {
            Ok(raw) => {
                let doc = lyrics::parse(&raw);
                lyrics_debug(
                    app,
                    &format!("[{track_id}] 下载 {} 字节，解析 {} 行，synced={}", raw.len(), doc.lines.len(), doc.synced),
                );
                if doc.synced || !doc.lines.is_empty() {
                    let doc = LyricsDoc { source: Some("huawei".into()), ..doc };
                    state::write_lyrics(app, track_id, &doc);
                    return Some(doc);
                }
            }
            Err(error) => lyrics_debug(app, &format!("[{track_id}] 下载失败 {:?}", error.message)),
        }
    }
    // 有的响应直接把歌词内嵌在 contentExInfo 里，没有资源地址。
    if let Some(inline) = lyrics::inline_lyrics_from(body) {
        let doc = lyrics::parse(&inline);
        lyrics_debug(app, &format!("[{track_id}] 内嵌歌词 {} 字符，解析 {} 行", inline.len(), doc.lines.len()));
        if doc.synced || !doc.lines.is_empty() {
            let doc = LyricsDoc { source: Some("huawei".into()), ..doc };
            state::write_lyrics(app, track_id, &doc);
            return Some(doc);
        }
    }
    None
}

#[tauri::command]
pub async fn lyrics(
    app: AppHandle,
    state: State<'_, AppState>,
    track_id: String,
    title: String,
    artist: String,
) -> Result<LyricsDoc, AppError> {
    // 歌词按曲目缓存 90 天，命中就不再走一遍取播放地址 + 下载。
    if let Some(cached) = state::read_lyrics(&app, &track_id) {
        lyrics_debug(&app, &format!("[{track_id}] 命中缓存"));
        return Ok(cached);
    }
    let doc = resolve_track_lyrics(&app, &state, &track_id, &title, &artist).await?;
    if doc.synced || !doc.lines.is_empty() {
        state::write_lyrics(&app, &track_id, &doc);
    }
    Ok(doc)
}

/// 歌词解析主流程，命令与 debug 自检共用。
async fn resolve_track_lyrics(
    app: &AppHandle,
    state: &State<'_, AppState>,
    track_id: &str,
    title: &str,
    artist: &str,
) -> Result<LyricsDoc, AppError> {
    let session = state.session()?;

    // 1) 播放响应。两种音质都试：有的账号只在其中一个音质下回 contentExInfo。
    for quality in ["standard", "lossless"] {
        let body = match state.client.playback(&session, track_id, quality).await {
            Ok(body) => body,
            Err(error) => {
                lyrics_debug(app, &format!("[{track_id}] {quality} 取播放响应失败：{}", error.message));
                continue;
            }
        };
        if let Some(doc) = resolve_lyrics(app, state, track_id, &body).await {
            return Ok(doc);
        }
    }

    // 2) 搜索兜底：列表项里的 contentExInfo 常常带着歌词，按 contentID 精确命中才用。
    let keyword = format!("{} {}", title.trim(), artist.trim()).trim().to_string();
    if !keyword.is_empty() {
        match state.client.search(&keyword, 0, 20).await {
            Ok(body) => match lyrics_from_search(&body, track_id) {
                Some((address, inline)) => {
                    lyrics_debug(
                        app,
                        &format!("[{track_id}] 搜索命中，有地址={} 内嵌={} 字符", !address.is_empty(), inline.len()),
                    );
                    if let Some(doc) = resolve_lyrics_from_parts(app, state, track_id, &address, &inline).await {
                        return Ok(doc);
                    }
                }
                None => lyrics_debug(app, &format!("[{track_id}] 搜索没找到同 ID 条目")),
            },
            Err(error) => lyrics_debug(app, &format!("[{track_id}] 搜索兜底失败：{}", error.message)),
        }
    }

    lyrics_debug(app, &format!("[{track_id}] 全部来源都没有歌词"));
    Ok(LyricsDoc { synced: false, lines: Vec::new(), source: None })
}


async fn resolve_lyrics_from_parts(
    app: &AppHandle,
    state: &State<'_, AppState>,
    track_id: &str,
    address: &str,
    inline: &str,
) -> Option<LyricsDoc> {
    if !address.is_empty() {
        if let Ok(raw) = state.client.fetch_text(address, lyrics::max_bytes()).await {
            let doc = lyrics::parse(&raw);
            if doc.synced || !doc.lines.is_empty() {
                let doc = LyricsDoc { source: Some("huawei".into()), ..doc };
                state::write_lyrics(app, track_id, &doc);
                return Some(doc);
            }
        }
    }
    if !inline.is_empty() {
        let doc = lyrics::parse(inline);
        if doc.synced || !doc.lines.is_empty() {
            let doc = LyricsDoc { source: Some("huawei".into()), ..doc };
            state::write_lyrics(app, track_id, &doc);
            return Some(doc);
        }
    }
    None
}

#[tauri::command]
pub async fn load_state(app: AppHandle) -> Result<PersistedState, AppError> {
    Ok(state::load_state(&app))
}

#[tauri::command]
pub async fn save_state(app: AppHandle, snapshot: PersistedState) -> Result<(), AppError> {
    state::save_state(&app, &snapshot).map_err(|message| AppError::invalid(format!("保存状态失败：{message}")))
}

#[tauri::command]
pub async fn lyrics_cache_size(app: AppHandle) -> Result<u64, AppError> {
    Ok(state::lyrics_cache_size(&app))
}

#[tauri::command]
pub async fn clear_lyrics_cache(app: AppHandle) -> Result<u64, AppError> {
    Ok(state::clear_lyrics_cache(&app))
}
