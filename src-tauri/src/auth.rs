use std::time::{Duration, Instant};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const AUTHORIZE_ENDPOINT: &str = "https://oauth-login.platform.hicloud.com/oauth2/v3/authorize";
pub const TOKEN_ENDPOINT: &str = "https://oauth-login.cloud.huawei.com/oauth2/v3/token";
pub const MUSIC_ENDPOINT: &str = "https://api-drcn.music.dbankcloud.cn";
/// 华为音乐官方 App 自身的 OAuth 身份，不是本项目注册的应用；仅供个人自用，不随产物分发。
pub const CLIENT_ID: &str = "101444301";
pub const REDIRECT_URI: &str = "hms://redirect_uri";

const LOGIN_HOSTS: [&str; 6] = [
    "oauth-login.platform.hicloud.com",
    "id1.cloud.huawei.com",
    "oauth-login.cloud.huawei.com",
    "account.huawei.com",
    "consumer.huawei.com",
    "oauth-login1.cloud.huawei.com",
];

const ATTEMPT_TTL: Duration = Duration::from_secs(10 * 60);

pub fn random_token(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    rand::thread_rng().fill_bytes(&mut buf);
    URL_SAFE_NO_PAD.encode(buf)
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub token: String,
    pub sns_user_id: Option<String>,
    pub expire_time: Option<String>,
}

impl Session {
    /// 只保留末四位，避免把完整 snsUserID 放上界面。
    pub fn account_label(&self) -> Option<String> {
        let id = self.sns_user_id.as_deref()?.trim();
        if id.is_empty() {
            return None;
        }
        let len = id.chars().count();
        let tail: String = id.chars().skip(len.saturating_sub(4)).collect();
        Some(format!("尾号 {tail}"))
    }
}

pub struct Attempt {
    pub verifier: String,
    pub state: String,
    created: Instant,
}

impl Attempt {
    pub fn new() -> Self {
        Self {
            verifier: random_token(48),
            state: random_token(32),
            created: Instant::now(),
        }
    }

    pub fn authorize_url(&self) -> String {
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(self.verifier.as_bytes()));
        let params = [
            ("client_id", CLIENT_ID),
            ("redirect_uri", REDIRECT_URI),
            ("response_type", "code"),
            ("access_type", "offline"),
            ("scope", "openid"),
            ("countryCode", "cn"),
            ("ui_locales", "zh-cn"),
            ("state", &self.state),
            ("code_challenge", &challenge),
            ("code_challenge_method", "S256"),
        ];
        let query = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencode(v)))
            .collect::<Vec<_>>()
            .join("&");
        format!("{AUTHORIZE_ENDPOINT}?{query}")
    }

    pub fn verify_state(&self, state: &str) -> bool {
        if self.created.elapsed() > ATTEMPT_TTL {
            return false;
        }
        // 等长比较，避免按前缀泄露匹配长度。
        let (a, b) = (state.as_bytes(), self.state.as_bytes());
        if a.len() != b.len() {
            return false;
        }
        a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
    }
}

fn urlencode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            b' ' => "%20".to_string(),
            other => format!("%{other:02X}"),
        })
        .collect()
}

/// 登录窗口只允许访问华为账号相关域名，别的导航一律拦掉。
pub fn is_allowed_login_url(url: &url::Url) -> bool {
    if url.scheme() != "https" || url.username() != "" || url.password().is_some() || url.port().is_some()
    {
        return false;
    }
    match url.host_str() {
        Some(host) => {
            let host = host.to_ascii_lowercase();
            if host == "oauth-login1.cloud.huawei.com" {
                // 区域 OAuth 服务只在 CAS 回调这一条路径上出现。
                return url.path() == "/oauth2/v3/loginCallback";
            }
            LOGIN_HOSTS.iter().any(|allowed| *allowed == host)
        }
        None => false,
    }
}

/// hms://redirect_uri?code=...&state=...
pub fn is_callback(url: &url::Url) -> bool {
    url.scheme() == "hms" && url.host_str() == Some("redirect_uri")
}

/// 从回调里取授权码；state 不匹配、出现 error 或参数重复都算失败。
pub fn consume_callback(url: &url::Url, attempt: &Attempt) -> Result<String, String> {
    let mut state: Option<String> = None;
    let mut code: Option<String> = None;
    let mut has_error = false;

    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "state" => {
                if state.is_some() {
                    return Err("回调参数重复".into());
                }
                state = Some(value.into_owned());
            }
            "code" => {
                if code.is_some() {
                    return Err("回调参数重复".into());
                }
                code = Some(value.into_owned());
            }
            "error" => has_error = true,
            _ => {}
        }
    }

    match state {
        Some(value) if attempt.verify_state(&value) => {}
        Some(_) => return Err("OAuth state 不匹配".into()),
        None => return Err("回调缺少 state".into()),
    }
    if has_error {
        return Err("授权被取消或拒绝".into());
    }
    code.filter(|c| !c.is_empty()).ok_or_else(|| "回调缺少授权码".into())
}

/// 音乐接口的 Digest 头：created 必须是带 Z 的 UTC 秒级时间，nonce 每请求随机。
pub fn digest_authorization(token: Option<&str>) -> String {
    let created = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    let nonce = random_token(24);
    match token {
        Some(t) => format!(
            "Digest username=app-music-baseline, nonce={nonce}, created={created}, token={t}"
        ),
        None => format!("Digest username=app-music-baseline, nonce={nonce}, created={created}"),
    }
}
