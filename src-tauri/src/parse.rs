use serde::{Deserialize, Serialize};
use serde_json::Value;

const ALLOWED_SUFFIXES: [&str; 3] = [".dbankcdn.cn", ".tingmall.com", ".kuke.com"];

/// 音频与封面共用的 CDN 白名单：必须 https、无端口、无账号密码。
pub fn is_safe_resource_url(raw: &str) -> bool {
    let Ok(parsed) = url::Url::parse(raw) else {
        return false;
    };
    if parsed.scheme() != "https"
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.port().is_some()
    {
        return false;
    }
    match parsed.host_str() {
        Some(host) => {
            let host = host.to_ascii_lowercase();
            ALLOWED_SUFFIXES
                .iter()
                .any(|suffix| host.ends_with(suffix) && host.len() > suffix.len())
        }
        None => false,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: Option<String>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Album {
    pub id: String,
    pub title: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artists: Vec<Artist>,
    pub album: Option<Album>,
    pub cover_url: Option<String>,
    pub duration_seconds: Option<f64>,
    pub available: bool,
    /// 服务端直接给的收藏标记（`isFavorite`），列表首屏就能据此点亮红心。
    #[serde(default)]
    pub favorite: bool,
}

fn text(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|v| v.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn at<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.get(key).filter(|v| !v.is_null())
}

fn numeric(value: Option<&Value>) -> Option<f64> {
    let raw = value?;
    if raw.is_boolean() {
        return None;
    }
    let parsed = match raw {
        Value::Number(n) => n.as_f64()?,
        Value::String(s) => s.parse::<f64>().ok()?,
        _ => return None,
    };
    (parsed.is_finite() && parsed >= 0.0).then_some(parsed)
}

/// 艺人优先取 artistNamePairs（带 id），没有才退回 artistName 整体作为一个显示名，
/// 不按逗号硬切，否则会丢掉艺名里的分隔信息。
fn artists_from(value: &Value) -> Vec<Artist> {
    let mut artists: Vec<Artist> = at(value, "artistNamePairs")
        .and_then(|v| v.as_array())
        .map(|pairs| {
            pairs
                .iter()
                .filter_map(|pair| {
                    let name = text(at(pair, "artistName").or_else(|| at(pair, "name")))?;
                    let id = text(at(pair, "artistID").or_else(|| at(pair, "id")));
                    Some(Artist { id, name })
                })
                .collect()
        })
        .unwrap_or_default();

    if artists.is_empty() {
        if let Some(name) = text(at(value, "artistName").or_else(|| at(value, "singerName"))) {
            let ids: Vec<String> = at(value, "artistID")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|i| text(Some(i))).collect())
                .unwrap_or_default();
            let id = if ids.len() == 1 { Some(ids[0].clone()) } else { None };
            artists.push(Artist { id, name });
        }
    }
    artists
}

fn cover_from(value: &Value) -> Option<String> {
    let picture = at(value, "picture");
    let candidates = [
        picture.and_then(|p| at(p, "bigImgURL")),
        picture.and_then(|p| at(p, "middleImgURL")),
        picture.and_then(|p| at(p, "smallImgURL")),
        at(value, "coverUrl"),
        at(value, "cover"),
    ];
    candidates
        .into_iter()
        .filter_map(|v| text(v))
        .find(|url| is_safe_resource_url(url))
}

const DURATION_KEYS: [&str; 12] = [
    "duration", "songDuration", "contentTime", "timeLength", "totalTime", "playTime",
    // 歌单 / 专辑详情里常见的时长字段名，收藏接口用的是前面几个
    "durationMs", "songDurationMs", "timing", "length", "songLength", "musicDuration",
];

/// 10 小时以上的数值按毫秒处理——单曲不可能这么长，但毫秒值几乎必然超过这个门槛。
fn normalize_duration(raw: f64) -> Option<f64> {
    if !raw.is_finite() || raw <= 0.0 {
        return None;
    }
    Some(if raw > 36_000.0 { raw / 1000.0 } else { raw })
}

/// 收藏列表这类响应把时长埋在 `contentExInfo`（JSON 字符串）里，逐层找不到就深搜。
fn deep_duration(value: &Value, depth: usize) -> Option<f64> {
    if depth > 3 {
        return None;
    }
    if let Some(map) = value.as_object() {
        for key in DURATION_KEYS {
            if let Some(seconds) = numeric(map.get(key)).and_then(normalize_duration) {
                return Some(seconds);
            }
        }
        for child in map.values() {
            if let Some(seconds) = deep_duration(child, depth + 1) {
                return Some(seconds);
            }
        }
    }
    None
}

fn content_extra(value: &Value) -> Option<Value> {
    match value.get("contentExInfo") {
        Some(Value::String(raw)) => serde_json::from_str::<Value>(raw).ok(),
        Some(v) if v.is_object() => Some(v.clone()),
        _ => None,
    }
}

fn duration_seconds(value: &Value) -> Option<f64> {
    for key in DURATION_KEYS {
        if let Some(seconds) = numeric(at(value, key)).and_then(normalize_duration) {
            return Some(seconds);
        }
    }
    // 先搜 contentExInfo（收藏列表埋这里），再对整个曲目对象深搜兜底。
    // 歌单详情的时长可能在别的嵌套层，只搜 contentExInfo 会漏。
    if let Some(seconds) = content_extra(value).and_then(|extra| deep_duration(&extra, 0)) {
        return Some(seconds);
    }
    deep_duration(value, 0)
}

fn favorite_flag(value: &Value) -> bool {
    match value.get("isFavorite") {
        Some(Value::Bool(flag)) => *flag,
        Some(Value::Number(num)) => num.as_i64() == Some(1),
        Some(Value::String(text)) => text == "1" || text.eq_ignore_ascii_case("true"),
        _ => false,
    }
}

pub fn track_from(value: &Value) -> Option<Track> {
    let id = text(at(value, "contentID").or_else(|| at(value, "contentId")).or_else(|| at(value, "songCode")))?;
    let title = text(at(value, "contentName").or_else(|| at(value, "songName")))?;
    let album_id = text(
        at(value, "albumID")
            .or_else(|| at(value, "albumId"))
            .or_else(|| at(value, "albumNamePair").and_then(|p| at(p, "id")))
            .or_else(|| at(value, "albumNamePair").and_then(|p| at(p, "albumID"))),
    );
    let album_title = text(
        at(value, "albumName")
            .or_else(|| at(value, "albumNamePair").and_then(|p| at(p, "name")))
            .or_else(|| at(value, "albumNamePair").and_then(|p| at(p, "albumName"))),
    );
    Some(Track {
        id,
        title,
        artists: artists_from(value),
        album: match (album_id, album_title) {
            (Some(id), Some(title)) => Some(Album { id, title }),
            _ => None,
        },
        cover_url: cover_from(value),
        duration_seconds: duration_seconds(value),
        available: true,
        favorite: favorite_flag(value),
    })
}

pub fn tracks_from(value: &Value) -> Vec<Track> {
    value
        .as_array()
        .map(|items| items.iter().filter_map(track_from).collect())
        .unwrap_or_default()
}

pub fn array_at<'a>(value: &'a Value, key: &str) -> &'a Value {
    value.get(key).unwrap_or(&Value::Null)
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: String,
    pub title: String,
    pub cover_url: Option<String>,
    pub count: Option<f64>,
    pub kind: String,
    pub description: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AlbumCard {
    pub id: String,
    pub title: String,
    pub cover_url: Option<String>,
    pub count: Option<f64>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AlbumMeta {
    pub id: String,
    pub title: String,
    pub cover_url: Option<String>,
    pub artists: Vec<Artist>,
    pub count: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ArtistMeta {
    pub id: String,
    pub name: String,
    pub cover_url: Option<String>,
}

/// 1 歌曲、4 收藏歌单、5 自建歌单、7 榜单；其余类型保留但标 unknown，不猜语义。
fn kind_of(content_type: Option<&str>) -> &'static str {
    match content_type {
        Some("4") => "collected",
        Some("5") => "owned",
        Some("7") => "chart",
        _ => "unknown",
    }
}

pub fn content_type_of(value: &Value) -> Option<String> {
    match value.get("contentType") {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Number(n)) => Some(n.to_string()),
        _ => None,
    }
}

fn playlist_with_kind(value: &Value, kind: &'static str) -> Option<Playlist> {
    let id = text(
        at(value, "contentID")
            .or_else(|| at(value, "contentId"))
            .or_else(|| at(value, "musicListID")),
    )?;
    let title = text(at(value, "contentName").or_else(|| at(value, "name")))?;
    Some(Playlist {
        id,
        title,
        cover_url: cover_from(value),
        count: numeric(
            at(value, "totalCount").or_else(|| at(value, "songCount")).or_else(|| at(value, "count")),
        ),
        kind: kind.to_string(),
        description: text(at(value, "listDesc").or_else(|| at(value, "description"))),
    })
}

pub fn playlist_from(value: &Value) -> Option<Playlist> {
    let kind = kind_of(content_type_of(value).as_deref());
    playlist_with_kind(value, kind)
}

/// 歌单发现接口把歌曲和各类歌单混在一页里，这里按 contentType 过滤并去重。
pub fn playlists_from_entries(array: &Value) -> Vec<Playlist> {
    let mut seen = std::collections::HashSet::new();
    array
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter(|item| content_type_of(item).as_deref() != Some("1"))
                .filter_map(|item| {
                    let id = text(
                        at(item, "contentID")
                            .or_else(|| at(item, "contentId"))
                            .or_else(|| at(item, "musicListID")),
                    )?;
                    let key = format!("{}:{id}", content_type_of(item).unwrap_or_default());
                    if !seen.insert(key) {
                        return None;
                    }
                    playlist_from(item)
                })
                .collect()
        })
        .unwrap_or_default()
}

/// 自建歌单详情：playListInfoEx.playListInfo + playListSongInfoList
pub fn owned_playlist_from(body: &Value) -> (Option<Playlist>, Vec<Track>, usize) {
    let detail = body.get("playListInfoEx");
    let playlist = detail
        .and_then(|d| d.get("playListInfo"))
        .and_then(|info| playlist_with_kind(info, "owned"));
    let songs = detail.map(|d| array_at(d, "playListSongInfoList")).unwrap_or(&Value::Null);
    let count = songs.as_array().map(|a| a.len()).unwrap_or(0);
    (playlist, tracks_from(songs), count)
}

/// 公开/收藏歌单详情：musicListInfoEx.musicListDetail + songSimpleInfos
pub fn public_playlist_from(body: &Value) -> (Option<Playlist>, Vec<Track>, usize) {
    let detail = body.get("musicListInfoEx");
    let playlist = detail.and_then(|d| d.get("musicListDetail")).and_then(|info| {
        let kind = kind_of(content_type_of(info).as_deref().or(Some("4")));
        playlist_with_kind(info, kind)
    });
    let songs = detail.map(|d| array_at(d, "songSimpleInfos")).unwrap_or(&Value::Null);
    let count = songs.as_array().map(|a| a.len()).unwrap_or(0);
    (playlist, tracks_from(songs), count)
}

pub fn album_from(body: &Value) -> (AlbumMeta, Vec<Track>, usize) {
    let detail = body.get("albumInfoEx");
    let info = detail.and_then(|d| d.get("albumInfo"));
    let songs = detail.map(|d| array_at(d, "songSimpleInfos")).unwrap_or(&Value::Null);
    let count = songs.as_array().map(|a| a.len()).unwrap_or(0);
    let album = AlbumMeta {
        id: info
            .and_then(|i| text(at(i, "albumCode").or_else(|| at(i, "albumID"))))
            .unwrap_or_default(),
        title: info.and_then(|i| text(at(i, "albumName"))).unwrap_or_default(),
        cover_url: info.and_then(cover_from),
        artists: info
            .and_then(|i| text(at(i, "artistName")))
            .map(|name| vec![Artist { id: None, name }])
            .unwrap_or_default(),
        count: info.and_then(|i| numeric(at(i, "songCount").or_else(|| at(i, "totalCount")))),
    };
    (album, tracks_from(songs), count)
}

pub fn artist_from(body: &Value) -> (ArtistMeta, Vec<Track>, Vec<AlbumCard>) {
    let detail = body.get("artistInfoEx");
    let info = detail.and_then(|d| d.get("artistInfo"));
    let artist = ArtistMeta {
        id: info
            .and_then(|i| text(at(i, "artistCode").or_else(|| at(i, "artistID"))))
            .unwrap_or_default(),
        name: info.and_then(|i| text(at(i, "artistName"))).unwrap_or_default(),
        cover_url: info.and_then(cover_from),
    };
    let albums = detail
        .map(|d| array_at(d, "albumSimpleInfos"))
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|entry| {
                    let id = text(at(entry, "albumCode").or_else(|| at(entry, "albumID")))?;
                    let title = text(at(entry, "albumName"))?;
                    Some(AlbumCard {
                        id,
                        title,
                        cover_url: cover_from(entry),
                        count: numeric(at(entry, "songCount")),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let tracks = detail
        .map(|d| tracks_from(array_at(d, "songSimpleInfos")))
        .unwrap_or_default();
    (artist, tracks, albums)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn duration_from_flat_field() {
        let v = json!({ "contentID": "x", "contentName": "t", "duration": 200 });
        assert_eq!(duration_seconds(&v), Some(200.0));
    }

    #[test]
    fn duration_from_nested_field() {
        // 歌单详情可能把时长埋在嵌套层，深搜要能兜住
        let v = json!({
            "contentID": "x", "contentName": "t",
            "songInfo": { "durationMs": 205000 },
            "albumInfo": { "songCount": 12 }
        });
        assert_eq!(duration_seconds(&v), Some(205.0));
    }

    #[test]
    fn duration_from_content_ex_info() {
        let v = json!({
            "contentID": "x", "contentName": "t",
            "contentExInfo": "{\"duration\":198}"
        });
        assert_eq!(duration_seconds(&v), Some(198.0));
    }

    #[test]
    fn duration_ignores_non_duration_numbers() {
        // albumInfo.songCount 这类字段不能被误判成时长
        let v = json!({
            "contentID": "x", "contentName": "t",
            "albumInfo": { "songCount": 12, "contentID": "999" }
        });
        assert_eq!(duration_seconds(&v), None);
    }
}
