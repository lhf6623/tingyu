use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::parse::is_safe_resource_url;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub time_ms: Option<i64>,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LyricLine {
    pub time_ms: Option<i64>,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segments: Option<Vec<Segment>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LyricsDoc {
    pub synced: bool,
    pub lines: Vec<LyricLine>,
    pub source: Option<String>,
}

struct Stamp {
    start: usize,
    end: usize,
    ms: i64,
}

const MAX_LYRIC_BYTES: usize = 512 * 1024;

/// `mm:ss` / `mm:ss.xx` / `mm:ss:xx` 统一成毫秒，秒位超过 59 视为无效而不是硬算。
fn parse_stamp(inner: &str, offset: i64) -> Option<i64> {
    let (minute_text, rest) = inner.split_once(':')?;
    let minutes: i64 = minute_text.trim().parse().ok()?;
    if !(0..=999).contains(&minutes) {
        return None;
    }
    let (second_text, fraction) = match rest.split_once(['.', ':']) {
        Some((s, f)) => (s, Some(f)),
        None => (rest, None),
    };
    let seconds: i64 = second_text.trim().parse().ok()?;
    if seconds > 59 {
        return None;
    }
    let millis = match fraction {
        Some(raw) => {
            let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).take(3).collect();
            if digits.is_empty() {
                0
            } else {
                format!("{digits:0<3}")[..3].parse::<i64>().ok()?
            }
        }
        None => 0,
    };
    Some((minutes * 60_000 + seconds * 1_000 + millis + offset).max(0))
}

/// 找出指定括号里的时间戳；`[ti:xxx]` 这类元数据因为分钟位解析失败会被自然跳过。
fn find_stamps(line: &str, open: char, close: char, offset: i64) -> Vec<Stamp> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if bytes[cursor] == open as u8 {
            if let Some(rel) = line[cursor + 1..].find(close) {
                let end = cursor + 1 + rel;
                let inner = &line[cursor + 1..end];
                if !inner.contains(' ') {
                    if let Some(ms) = parse_stamp(inner, offset) {
                        out.push(Stamp { start: cursor, end: end + 1, ms });
                    }
                }
                cursor = end + 1;
                continue;
            }
        }
        cursor += 1;
    }
    out
}

fn strip_ranges(line: &str, ranges: &[Stamp]) -> String {
    let mut text = String::with_capacity(line.len());
    let mut cursor = 0usize;
    for stamp in ranges {
        if stamp.start >= cursor {
            text.push_str(&line[cursor..stamp.start]);
        }
        cursor = stamp.end;
    }
    if cursor < line.len() {
        text.push_str(&line[cursor..]);
    }
    text.trim().to_string()
}

fn word_segments(body: &str, offset: i64, fallback: Option<i64>) -> Option<Vec<Segment>> {
    let marks = find_stamps(body, '<', '>', offset);
    if marks.is_empty() {
        return None;
    }
    let mut segments = Vec::new();
    let leading = body[..marks[0].start].trim();
    if !leading.is_empty() {
        segments.push(Segment { time_ms: fallback, text: leading.to_string() });
    }
    for (index, mark) in marks.iter().enumerate() {
        let from = mark.end;
        let to = marks.get(index + 1).map(|next| next.start).unwrap_or(body.len());
        if from >= to {
            continue;
        }
        let text = body[from..to].to_string();
        if text.trim().is_empty() {
            continue;
        }
        segments.push(Segment { time_ms: Some(mark.ms), text });
    }
    if segments.is_empty() {
        None
    } else {
        Some(segments)
    }
}

fn is_metadata(line: &str) -> bool {
    const KEYS: [&str; 8] = ["ti", "ar", "al", "by", "re", "ve", "length", "offset"];
    let Some(inner) = line.strip_prefix('[').and_then(|rest| rest.strip_suffix(']')) else {
        return false;
    };
    let Some((key, _)) = inner.split_once(':') else {
        return false;
    };
    let key = key.trim().to_ascii_lowercase();
    KEYS.contains(&key.as_str())
}

/// 同一时间戳的第二行按双语翻译处理，而不是当成重复行。
fn merge_translations(lines: Vec<LyricLine>) -> Vec<LyricLine> {
    let mut merged: Vec<LyricLine> = Vec::with_capacity(lines.len());
    for line in lines {
        let mergeable = merged.last().is_some_and(|previous| {
            previous.time_ms == line.time_ms
                && line.segments.is_none()
                && previous.translation.is_none()
                && !line.text.is_empty()
                && line.text != previous.text
        });
        match (mergeable, merged.last_mut()) {
            (true, Some(previous)) => previous.translation = Some(line.text),
            _ => merged.push(line),
        }
    }
    merged
}

pub fn parse(raw: &str) -> LyricsDoc {
    if raw.trim().is_empty() {
        return LyricsDoc { synced: false, lines: Vec::new(), source: None };
    }
    let mut offset = 0i64;
    let mut timed: Vec<LyricLine> = Vec::new();
    let mut plain: Vec<LyricLine> = Vec::new();

    for raw_line in raw.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(inner) = line.strip_prefix("[offset:").and_then(|r| r.strip_suffix(']')) {
            if let Ok(value) = inner.trim().parse::<i64>() {
                offset = value;
                continue;
            }
        }
        if is_metadata(line) {
            continue;
        }

        let stamps = find_stamps(line, '[', ']', offset);
        if stamps.is_empty() {
            let segments = word_segments(line, offset, None);
            let text = strip_ranges(line, &find_stamps(line, '<', '>', offset));
            plain.push(LyricLine { time_ms: None, text, translation: None, segments });
            continue;
        }

        let last = stamps.last().expect("non-empty");
        let body = line[last.end..].trim().to_string();
        let text = strip_ranges(&body, &find_stamps(&body, '<', '>', offset));
        for stamp in &stamps {
            if text.is_empty() && stamp.ms == 0 {
                continue;
            }
            timed.push(LyricLine {
                time_ms: Some(stamp.ms),
                text: text.clone(),
                translation: None,
                segments: word_segments(&body, offset, Some(stamp.ms)),
            });
        }
    }

    if timed.is_empty() {
        return LyricsDoc { synced: false, lines: plain, source: None };
    }
    timed.sort_by_key(|line| line.time_ms.unwrap_or(i64::MAX));
    LyricsDoc { synced: true, lines: merge_translations(timed), source: None }
}

fn nested<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter().try_fold(value, |node, key| node.get(key))
}

/// `contentExInfo` 经常是 JSON 字符串，需要二次解析；解析失败按没有处理，不能让整页崩掉。
fn extra_info(body: &Value) -> Option<Value> {
    match body.get("contentExInfo") {
        Some(Value::String(raw)) => serde_json::from_str::<Value>(raw).ok(),
        Some(value) if value.is_object() => Some(value.clone()),
        _ => None,
    }
}

fn text_at(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// 歌词资源地址可能出现在多个位置：播放响应顶层、songInfo 里、contentExInfo（含历史拼写
/// `lyricAddres`）、fileInfos 的每个条目。逐个试，只认白名单内的 https 地址。
pub fn lyric_address_from(body: &Value) -> Option<String> {
    let extra = extra_info(body);
    let song_info_extra = body
        .get("songInfo")
        .and_then(|info| extra_info(info));
    let mut candidates: Vec<Option<String>> = vec![
        text_at(body.get("lyricAddress")),
        text_at(body.get("lyricAddres")),
        extra.as_ref().and_then(|e| text_at(e.get("lyricAddress"))),
        extra.as_ref().and_then(|e| text_at(e.get("lyricAddres"))),
        extra.as_ref().and_then(|e| text_at(nested(e, &["lyrics", "lyricAddress"]))),
        extra.as_ref().and_then(|e| text_at(nested(e, &["lyrics", "lineByLineLyric"]))),
        text_at(nested(body, &["lyrics", "lineByLineLyric"])),
        text_at(nested(body, &["songInfo", "lyricAddress"])),
        text_at(nested(body, &["songInfo", "lyricAddres"])),
        text_at(nested(body, &["songInfo", "lyrics", "lineByLineLyric"])),
        text_at(nested(body, &["songInfo", "lyrics", "lyricAddress"])),
        song_info_extra.as_ref().and_then(|e| text_at(e.get("lyricAddress"))),
        song_info_extra.as_ref().and_then(|e| text_at(e.get("lyricAddres"))),
        song_info_extra
            .as_ref()
            .and_then(|e| text_at(nested(e, &["lyrics", "lineByLineLyric"]))),
    ];

    // fileInfos 可能是数组也可能是 key→对象 的映射，两种都要覆盖。
    if let Some(entries) = body.get("fileInfos") {
        let values: Vec<&Value> = match entries {
            Value::Array(items) => items.iter().collect(),
            Value::Object(map) => map.values().collect(),
            _ => Vec::new(),
        };
        for entry in values {
            candidates.push(text_at(entry.get("lyricAddress")));
            candidates.push(text_at(entry.get("lyricAddres")));
        }
    }

    candidates.into_iter().flatten().find(|url| is_safe_resource_url(url))
}

fn unwrap_lyric(value: Option<&Value>, depth: usize) -> Option<String> {
    let raw = value?.as_str()?.trim();
    if raw.is_empty() || raw.len() > MAX_LYRIC_BYTES || depth > 2 {
        return None;
    }
    if !raw.starts_with('{') {
        return Some(raw.to_string());
    }
    let parsed: Value = serde_json::from_str(raw).ok()?;
    for key in ["lyric", "lyrics", "content", "lrc"] {
        if let Some(inner) = unwrap_lyric(parsed.get(key), depth + 1) {
            return Some(inner);
        }
    }
    None
}

/// 有些响应直接内嵌歌词文本，而不是给资源地址。
pub fn inline_lyrics_from(body: &Value) -> Option<String> {
    let extra = extra_info(body);
    let song_info = body.get("songInfo");
    let candidates = [
        extra.as_ref().and_then(|e| e.get("lyrics")),
        extra.as_ref().and_then(|e| e.get("Lyrics")),
        body.get("lyrics"),
        body.get("Lyrics"),
        song_info.and_then(|s| s.get("lyrics")),
        song_info.and_then(|s| s.get("Lyrics")),
    ];
    let found = candidates.into_iter().find_map(|value| unwrap_lyric(value, 0));
    found
}

pub fn max_bytes() -> usize {
    MAX_LYRIC_BYTES
}
