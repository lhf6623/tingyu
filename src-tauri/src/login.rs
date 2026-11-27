use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::mpsc::{self, error::TryRecvError};

use crate::api::{AppError, Client};
use crate::auth::{self, Attempt, Session};

const LOGIN_LABEL: &str = "login";
const WAIT_LIMIT: Duration = Duration::from_secs(5 * 60);

/// 完整登录链路：内置窗口走官方授权页 → 拦截 hms:// 回调 → 换账号令牌 → 换音乐会话。
/// 官方页面跑在独立 webview 里，没有 IPC 权限，导航只放行华为账号相关域名。
pub async fn run_login(app: &AppHandle, client: &Client) -> Result<Session, AppError> {
    let attempt = Attempt::new();
    let target = url::Url::parse(&attempt.authorize_url())
        .map_err(|_| AppError::invalid("授权地址构造失败"))?;

    let (nav_tx, mut nav_rx) = mpsc::unbounded_channel::<String>();
    let nav_tx = Arc::new(Mutex::new(Some(nav_tx)));
    let nav_for_window = nav_tx.clone();

    let (built_tx, built_rx) = std::sync::mpsc::channel::<Result<(), String>>();
    let app_for_build = app.clone();
    app.run_on_main_thread(move || {
        let result = (|| -> tauri::Result<()> {
            let window = WebviewWindowBuilder::new(&app_for_build, LOGIN_LABEL, WebviewUrl::External(target))
                .title("登录 · 华为账号")
                .inner_size(460.0, 700.0)
                .min_inner_size(380.0, 560.0)
                .center()
                .on_navigation(move |url| {
                    if auth::is_callback(url) {
                        if let Some(tx) = nav_for_window.lock().ok().and_then(|mut slot| slot.take()) {
                            let _ = tx.send(url.to_string());
                        }
                        return false;
                    }
                    let allowed = auth::is_allowed_login_url(url);
                    if !allowed {
                        eprintln!(
                            "[login] 拦截了非白名单导航: {}://{}",
                            url.scheme(),
                            url.host_str().unwrap_or("-")
                        );
                    }
                    allowed
                })
                .build()?;
            let _ = window.set_focus();
            Ok(())
        })();
        let _ = built_tx.send(result.map_err(|e| e.to_string()));
    })
    .map_err(|e| AppError::invalid(format!("无法创建登录窗口：{e}")))?;

    let built = tokio::task::spawn_blocking(move || built_rx.recv_timeout(Duration::from_secs(10)))
        .await
        .map_err(|_| AppError::invalid("登录窗口无响应"))?;
    match built {
        Ok(Ok(())) => {}
        Ok(Err(message)) => return Err(AppError::invalid(format!("登录窗口创建失败：{message}"))),
        Err(_) => return Err(AppError::invalid("登录窗口创建超时")),
    }

    let callback = wait_for_callback(app, &mut nav_rx).await;
    close_login_window(app);

    let raw = callback?;
    let parsed = url::Url::parse(&raw).map_err(|_| AppError::auth("回调地址无法解析"))?;
    let code = auth::consume_callback(&parsed, &attempt).map_err(AppError::auth)?;

    let access_token = client.exchange_code(&code, &attempt.verifier).await?;
    client.exchange_music_session(&access_token).await
}

async fn wait_for_callback(
    app: &AppHandle,
    rx: &mut mpsc::UnboundedReceiver<String>,
) -> Result<String, AppError> {
    let deadline = tokio::time::Instant::now() + WAIT_LIMIT;
    loop {
        match rx.try_recv() {
            Ok(url) => return Ok(url),
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => return Err(AppError::auth("登录已取消")),
        }
        if app.get_webview_window(LOGIN_LABEL).is_none() {
            return Err(AppError::auth("登录已取消"));
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(AppError::auth("登录等待超时，请重新发起"));
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
}

fn close_login_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(LOGIN_LABEL) {
        let _ = window.close();
    }
}
