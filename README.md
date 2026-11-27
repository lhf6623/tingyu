# 音乐 · Tauri 桌面客户端（非官方）

> **免责声明**：本项目为非官方、实验性开源项目，与任何音乐服务提供方及其关联公司**无隶属、合作或代理关系**，未获官方授权、认可或技术支持。客户端仅播放使用者**本人账号已有权益**允许的音频流，不提供音频下载、离线保存、解密或转码功能，**不绕过任何付费、会员、地域或版权限制**。完整说明见 [docs/DISCLAIMER.md](docs/DISCLAIMER.md)。

Tauri v2 + Vue 3 实现的桌面播放器，面向 Windows 与 macOS。仅登录你自己的账号、读取你自己的音乐库、播放该账号已有权益的内容。

**本项目不发布带有 OAuth 应用标识的安装包，也不建议任何人发布。** 请从源码自行构建、自行使用，理由见[免责声明第 5 节](docs/DISCLAIMER.md#5-登录身份与接口)。

## 技术选型

| 层 | 选型 | 说明 |
|---|---|---|
| 外壳 | Tauri v2 | Windows WebView2 / macOS WKWebView |
| 后端 | Rust | 协议、凭据、HTTP 全部在主进程侧 |
| 前端 | Vue 3 + TS + Vite | 无 UI 框架，手写 CSS |
| 凭据 | `keyring` crate | Windows 凭据管理器 / macOS Keychain |
| HTTP | `reqwest` + rustls | 不依赖系统 OpenSSL |

## 目录

```
src-tauri/src/
  main.rs        入口、插件注册、命令表
  auth.rs        PKCE、state 校验、Digest 头、回调判定
  api.rs         音乐服务接口（13 个端点）
  parse.rs       响应归一化（contentExInfo 二次解析规则同参考实现）
  lyrics.rs      LRC 解析 + 歌词资源提取
  login.rs       内置登录窗口 + hms:// 回调拦截
  store.rs       keyring 会话
  state.rs       state.json + 歌词缓存
  commands.rs    IPC 白名单（18 个命令）
src/
  api.ts         invoke 封装与类型
  App.vue        外壳：侧栏 + 视图切换 + 播放栏 + 歌词面板
  styles.css     设计变量（明暗两套）
```

## 运行

```sh
pnpm install
pnpm tauri dev            # 开发（vite + tauri，前端热更新）
pnpm tauri build          # 打包（release + 安装包）
pnpm tauri build --no-bundle   # 只出 exe，不打安装包（快）
pnpm typecheck            # vue-tsc
pnpm test                 # Rust 单测
```

需要 Rust 工具链 + Windows 上装 WebView2 运行时。产物在 `src-tauri/target/release/`（裸 exe）与 `src-tauri/target/release/bundle/`（nsis / msi）。

## 打包

实测产物（Windows x64，LTO + strip）：

| 产物 | 路径 | 体积 |
|---|---|---|
| 裸 exe | `target/release/music-desktop.exe` | 4.7 MiB |
| MSI | `target/release/bundle/msi/音乐_0.1.0_x64_zh-CN.msi` | 2.45 MiB |
| NSIS | `target/release/bundle/nsis/音乐_0.1.0_x64-setup.exe` | 1.85 MiB |

**中文产品名会让 MSI 打包失败**（`error LGHT0311: code page '1252'`）。WiX 默认用 `en-US.wxl`，代码页容纳不下中文。`tauri.conf.json` 里已经固定：

```json
"bundle": {
  "windows": {
    "wix": { "language": "zh-CN" },
    "nsis": { "languages": ["SimpChinese"], "displayLanguageSelector": false }
  }
}
```

改了产品名或加了新的非 ASCII 文案后，如果又报 LGHT0311，检查这段配置是否被覆盖。

产物**未签名**，Windows SmartScreen 会拦截，需自行核对哈希，不要为此关闭系统防护。

## 安全边界（与参考实现一致）

1. 音乐 token / 账号 token 只在 Rust 侧，前端只拿到结构化结果和当次播放地址。
2. 登录页跑在独立 webview，无 IPC 权限；导航只放行该服务的账号域名，其余直接拦掉。
3. 音频、封面地址必须命中 CDN 白名单（`.dbankcdn.cn` / `.tingmall.com` / `.kuke.com`），CSP 也按同一份白名单收紧。
4. 不关闭 TLS 校验，不注册系统级 `hms://` 处理器。

## 当前进度

| 能力 | 状态 |
|---|---|
| 登录（PKCE + 回调拦截 + 音乐会话） | **真机验证通过** |
| 收藏列表 + 分页 / 收藏增删 | **真机验证通过**（播放已验证） |
| 播放器：队列、上/下一首、进度拖动、音量、四种播放模式 | 代码完成，待逐项点验 |
| 搜索 / 歌单详情 / 专辑 / 歌手 / 最近播放 | 代码完成，待验证 |
| 歌词：独立整页（封面 + 逐行歌词，当前句居中，点句跳转），支持翻译行 | 代码完成，待验证 |
| 播放队列：右侧面板查看/跳转/移除/清空，右键「下一首播放」「加入队列」 | 代码完成 |
| 音质切换（标准 / 无损） | 代码完成，待验证 |
| 快捷键（空格、方向键） | 代码完成 |
| 托盘图标 + 关闭到托盘 + 托盘控制上一首/播放暂停/下一首 | 代码完成，实测关闭被拦截 |
| 自绘标题栏（最小化/最大化/关闭 + 顶部全局搜索） | 代码完成 |
| 媒体键（系统 SMTC / MPRIS）、迷你播放器 | 未开始 |

## 持久化

| 数据 | 位置 | 说明 |
|---|---|---|
| 音乐会话（token / snsUserID） | Windows 凭据管理器 / macOS Keychain（`keyring`） | 不含账号 AT/RT |
| 队列 + 当前曲 + 音量 + 播放模式 + 音质 + 队列面板开合 | `%APPDATA%\com.hf.music.desktop\state.json` | 400ms 防抖写入，先写临时文件再改名；恢复后**不自动播放** |
| 歌词文本 | `%LOCALAPPDATA%\com.hf.music.desktop\lyrics\*.json` | 90 天过期，按曲目哈希命名；设置页可查看占用并清除 |
| 窗口尺寸 / 位置 | 插件 `tauri-plugin-window-state` | 退出时落盘，启动时恢复 |

读取时统一做边界收敛：音量夹到 0–1、未知播放模式退回列表循环、越界 cursor 拉回、缺 id 或标题的曲目丢弃、队列截断到 500 首。这些行为由 `state.rs` 的 6 个单元测试覆盖（`pnpm test`）。

## 目录补充

```
src/
  components/    SideBar PlayerBar TrackList QueuePane CoverGrid DetailHeader Icon
  views/         Discover Favorites Search Settings Playlist Album Artist Lyrics
  composables/   usePlayer（队列/播放/歌词）useNav（手写路由栈）useToast
```

## 歌词来源

播放响应的歌单元数据在 `songInfo` 下，**歌词地址是 `songInfo.lyricAddress`**（实测 host 为 `lyric.tingmall.com`），顶层没有 `contentExInfo`。取歌词按以下顺序，任一命中即写缓存：

1. 播放响应（先标准音质，取不到再试无损）
2. 地址下载 → LRC 解析
3. 响应内嵌歌词（`contentExInfo.lyrics` / `songInfo.lyrics`，允许套一层 JSON）
4. 搜索兜底：用「歌名 + 艺人」搜一次，**只接受 contentID 完全一致**的那条，再从列表项里取歌词。不按歌名模糊匹配，避免贴错歌词。

## 已知约束

- `client_id=101444301` 是某音乐 App 官方客户端自身的 OAuth 身份，不属于本项目。它随时可能失效，且**把它随软件分发可能涉及服务条款违约与身份冒用**，故本项目不发布安装包。
- 音乐会话约 7 天到期，标准 refresh 流程未打通，失效后需要重新扫码。

## 许可

代码采用 **AGPL-3.0-only**，见 [LICENSE](LICENSE) 与 [NOTICE](NOTICE)。

许可证**仅覆盖本仓库代码**，不授予音乐内容、商标、服务接口的任何使用或再分发许可。

## 免责声明与使用边界

- **非官方**：与任何音乐服务提供方无隶属或合作关系，未获授权或背书；项目名与界面中出现的服务名称仅用于说明所兼容的服务，相关商标归各自权利人所有。
- **合法前提**：仅使用本人合法持有的账号，在适用法律、服务条款与实际权益允许的范围内、以个人学习研究为目的运行。
- **禁止用途**：绕过付费/会员/地域/版权与技术保护、下载解密转存受保护内容、抓取或滥用接口、代理他人凭据、搭建面向公众的音源解析或商业服务。
- **不构成授权**：「能登录」「能播放」「买了会员」都不等于获得使用第三方客户端、调用接口或分发软件的许可。
- **数据边界**：凭据只存操作系统凭据管理器；播放地址不落盘；无遥测与崩溃上报；歌词缓存仅含歌词文本与曲目标识。
- **风险自担**：项目按现状提供，不承诺持续可用；接口变更、账号风控、数据丢失等风险由使用者自行承担。

**以上不是法律意见，也不能免除法律责任。** 有意公开分发或商业化之前，请咨询律师并完成授权、商标与接口合规评估。[完整版](docs/DISCLAIMER.md)
