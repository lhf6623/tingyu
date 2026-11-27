use std::time::Duration;

use serde::Serialize;
use serde_json::{json, Value};

use crate::auth::{self, Session, MUSIC_ENDPOINT};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub kind: &'static str,
    pub message: String,
    pub retryable: bool,
    pub code: Option<String>,
}

impl AppError {
    pub(crate) fn new(kind: &'static str, message: impl Into<String>, retryable: bool) -> Self {
        Self { kind, message: message.into(), retryable, code: None }
    }

    pub fn auth(message: impl Into<String>) -> Self {
        Self::new("auth", message, false)
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new("invalid", message, false)
    }

    pub fn network() -> Self {
        Self::new("network", "网络请求失败，请检查连接", true)
    }

    fn provider(status: u16, body: Option<&Value>) -> Self {
        let code = body
            .and_then(|v| {
                v.get("result")
                    .and_then(|r| r.get("resultCode"))
                    .and_then(|c| c.as_str())
                    .or_else(|| v.get("error").and_then(|e| e.as_str().map(|s| s)))
            })
            .map(|c| c.chars().filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_').take(40).collect::<String>());
        Self {
            kind: "provider",
            message: format!("服务端拒绝了这次请求（HTTP {status}）"),
            retryable: status >= 500,
            code,
        }
    }
}

type Result<T> = std::result::Result<T, AppError>;

pub struct Client {
    http: reqwest::Client,
    /// 资源下载单独一个 client：不跟随重定向，避免歌词地址把我们带到白名单之外。
    resource: reqwest::Client,
}

impl Client {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(25))
            .connect_timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
            .build()
            .expect("http client");
        let resource = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
            .build()
            .expect("resource client");
        Self { http, resource }
    }

    fn url(path: &str) -> String {
        format!("{MUSIC_ENDPOINT}{path}")
    }

    /// 业务成功要求 HTTP 成功且 result.resultCode == "000000"。
    async fn finish(resp: reqwest::Response) -> Result<Value> {
        let status = resp.status().as_u16();
        let bytes = resp.bytes().await.map_err(|_| AppError::network())?;
        let body: Value = serde_json::from_slice(&bytes).map_err(|_| AppError::provider(status, None))?;
        let ok = body
            .get("result")
            .and_then(|r| r.get("resultCode"))
            .and_then(|c| c.as_str())
            .map(|c| c == "000000")
            .unwrap_or(false);
        if !(200..300).contains(&status) || !ok || body.get("error").is_some() {
            return Err(AppError::provider(status, Some(&body)));
        }
        Ok(body)
    }

    async fn get(&self, path: &str, query: &[(&str, String)], token: Option<&str>) -> Result<Value> {
        let resp = self
            .http
            .get(Self::url(path))
            .header("Authorization", auth::digest_authorization(token))
            .query(query)
            .send()
            .await
            .map_err(|_| AppError::network())?;
        Self::finish(resp).await
    }

    async fn post(&self, path: &str, body: Value, token: Option<&str>) -> Result<Value> {
        let resp = self
            .http
            .post(Self::url(path))
            .header("Authorization", auth::digest_authorization(token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|_| AppError::network())?;
        Self::finish(resp).await
    }

    /// 授权码换账号令牌。官方这个公共客户端不传 client_secret。
    pub async fn exchange_code(&self, code: &str, verifier: &str) -> Result<String> {
        let params = [
            ("grant_type", "authorization_code"),
            ("client_id", auth::CLIENT_ID),
            ("redirect_uri", auth::REDIRECT_URI),
            ("code", code),
            ("code_verifier", verifier),
        ];
        let resp = self
            .http
            .post(auth::TOKEN_ENDPOINT)
            .form(&params)
            .send()
            .await
            .map_err(|_| AppError::network())?;
        let status = resp.status().as_u16();
        let body: Value = resp.json().await.map_err(|_| AppError::provider(status, None))?;
        if !(200..300).contains(&status) {
            return Err(AppError::provider(status, Some(&body)));
        }
        body.get("access_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::auth("授权响应缺少访问令牌"))
    }

    /// 账号令牌换音乐会话。这才是后续业务请求要用的凭据。
    pub async fn exchange_music_session(&self, access_token: &str) -> Result<Session> {
        let body = self
            .post(
                "/music-odpapp-service/v5/service/userinfo/loginex",
                json!({ "accessToken": access_token }),
                None,
            )
            .await?;
        let token = body
            .get("token")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| AppError::auth("音乐会话响应缺少令牌"))?
            .to_string();
        Ok(Session {
            token,
            sns_user_id: body.get("snsUserID").and_then(|v| v.as_str()).map(|s| s.to_string()),
            expire_time: body.get("expireTime").and_then(|v| v.as_str()).map(|s| s.to_string()),
        })
    }

    pub async fn favorites(&self, session: &Session, start: i64, limit: i64) -> Result<Value> {
        let sns = session
            .sns_user_id
            .as_deref()
            .ok_or_else(|| AppError::auth("音乐会话缺少用户标识，请重新登录"))?;
        self.post(
            "/music-userbehavior-service/v3/service/favorite/simple/all",
            json!({ "targetSnsUserID": sns, "start": start, "limit": limit, "contentTypes": [1] }),
            Some(&session.token),
        )
        .await
    }

    pub async fn search(&self, query: &str, start: i64, limit: i64) -> Result<Value> {
        self.post(
            "/music-search-service/v10/service/fuzzysearch",
            json!({
                "contentType": "1", "correct": "1", "limit": limit.to_string(),
                "queryWord": query, "start": start.to_string()
            }),
            None,
        )
        .await
    }

    pub async fn playback(&self, session: &Session, track_id: &str, quality: &str) -> Result<Value> {
        let quality_type = match quality {
            "lossless" => "3",
            "standard" => "1",
            _ => return Err(AppError::invalid("不支持的音质")),
        };
        self.post(
            "/music-play-service/v3/service/file/bycontentcode",
            json!({ "contentCode": track_id, "contentType": "1", "qualityType": quality_type }),
            Some(&session.token),
        )
        .await
    }

    pub async fn favorite_exist(&self, session: &Session, track_id: &str) -> Result<bool> {
        let body = self
            .get(
                "/music-userbehavior-service/v1/service/favorite/simple/exist",
                &[("contentType", "1".into()), ("contentID", track_id.into())],
                Some(&session.token),
            )
            .await?;
        match body.get("exist").map(|v| v.as_str().unwrap_or("")) {
            Some("1") => Ok(true),
            Some("0") => Ok(false),
            _ => Err(AppError::invalid("服务端未确认收藏状态")),
        }
    }

    /// 写操作不重试：超时也可能已经在服务端生效。
    pub async fn set_favorite(&self, session: &Session, track_id: &str, favorite: bool) -> Result<()> {
        let action = if favorite { "create" } else { "delete" };
        self.post(
            &format!("/music-userbehavior-service/v1/service/favorite/{action}"),
            json!({ "contentType": "1", "contentID": track_id }),
            Some(&session.token),
        )
        .await
        .map(|_| ())
    }

    pub async fn recent(&self, session: &Session) -> Result<Value> {
        self.get(
            "/music-userbehavior-service/v1/service/recentlyplayed/query",
            &[("contentType", "1".into())],
            Some(&session.token),
        )
        .await
    }

    /// 一次调用同时返回歌曲与本人在建的各类歌单，调用方按 contentType 分流。
    pub async fn library(&self, session: &Session, start: i64, limit: i64) -> Result<Value> {
        let sns = session
            .sns_user_id
            .as_deref()
            .ok_or_else(|| AppError::auth("音乐会话缺少用户标识，请重新登录"))?;
        self.post(
            "/music-userbehavior-service/v3/service/favorite/simple/all",
            json!({ "targetSnsUserID": sns, "start": start, "limit": limit, "contentTypes": [1, 4, 5, 7] }),
            Some(&session.token),
        )
        .await
    }

    /// 自建歌单详情必须给真实 playListID，它不是歌单列表接口。
    pub async fn playlist_detail(&self, session: &Session, id: &str, start: i64, limit: i64) -> Result<Value> {
        self.get(
            "/music-userbehavior-service/v1/service/playlist/detail/all",
            &[("playListID", id.to_string()), ("start", start.to_string()), ("limit", limit.to_string())],
            Some(&session.token),
        )
        .await
    }

    pub async fn public_playlist(&self, id: &str, start: i64, limit: i64) -> Result<Value> {
        self.post(
            "/music-operation-service/v2/service/musiclist/detail/bymusiclistid",
            json!({ "musicListID": id, "copyrightSwitch": "1", "start": start, "limit": limit.to_string() }),
            None,
        )
        .await
    }

    pub async fn album(&self, id: &str, start: i64, limit: i64) -> Result<Value> {
        self.get(
            "/music-metacontent-service/v1/service/album/detail/byalbumcode",
            &[("albumCode", id.to_string()), ("start", start.to_string()), ("limit", limit.to_string())],
            None,
        )
        .await
    }

    pub async fn artist(&self, id: &str, song_num: i64, album_num: i64) -> Result<Value> {
        self.post(
            "/music-metacontent-service/v2/service/artist/detail/byartistcode",
            json!({ "artistCode": id, "songNum": song_num, "albumNum": album_num }),
            None,
        )
        .await
    }

    /// 下载歌词文本。调用前地址必须已经过 CDN 白名单校验。
    pub async fn fetch_text(&self, url: &str, max_bytes: usize) -> Result<String> {
        let resp = self
            .resource
            .get(url)
            .header("accept", "text/plain,*/*")
            .send()
            .await
            .map_err(|_| AppError::network())?;
        if !resp.status().is_success() {
            return Err(AppError::new("provider", "歌词下载失败", true));
        }
        if resp.content_length().is_some_and(|len| len as usize > max_bytes) {
            return Err(AppError::invalid("歌词文件过大"));
        }
        let bytes = resp.bytes().await.map_err(|_| AppError::network())?;
        if bytes.len() > max_bytes {
            return Err(AppError::invalid("歌词文件过大"));
        }
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}
