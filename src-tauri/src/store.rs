use crate::auth::Session;

const SERVICE: &str = "com.hf.music.desktop";
const ACCOUNT: &str = "music-session";

fn entry() -> Result<keyring::Entry, keyring::Error> {
    keyring::Entry::new(SERVICE, ACCOUNT)
}

/// 只存最小音乐会话（token / snsUserID / 到期字段），账号 AT 不落地。
pub fn save_session(session: &Session) -> Result<(), String> {
    let payload = serde_json::to_string(session).map_err(|e| e.to_string())?;
    entry()
        .and_then(|e| e.set_password(&payload))
        .map_err(|e| format!("写入系统凭据失败：{e}"))
}

pub fn load_session() -> Option<Session> {
    let raw = entry().ok()?.get_password().ok()?;
    serde_json::from_str::<Session>(&raw).ok().filter(|s| !s.token.is_empty())
}

pub fn clear_session() {
    if let Ok(entry) = entry() {
        let _ = entry.delete_credential();
    }
}
