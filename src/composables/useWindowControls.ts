import { ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

/**
 * 窗口控制（最小化 / 最大化 / 关闭）与最大化状态。
 *
 * 标题栏和歌词覆盖层都要用：歌词盖住整个窗口后，原生标题栏被压在下面，
 * 必须由覆盖层自己提供这几个按钮，否则窗口既关不掉也拖不动。
 * 状态放模块级，两处共用同一份，避免各自维护导致图标状态不一致。
 */
const win = getCurrentWindow();
const maximized = ref(false);

async function sync() {
  try {
    maximized.value = await win.isMaximized();
  } catch {
    maximized.value = false;
  }
}

let started = false;

function start() {
  if (started) return;
  started = true;
  void sync();
  // 监听挂到应用生命周期结束：窗口只有一个，没必要反复解绑。
  void win.onResized(() => void sync());
}

async function minimize() {
  await win.minimize();
}

async function toggleMaximize() {
  await win.toggleMaximize();
  await sync();
}

/** 走 Tauri 的关闭请求：开了「关闭到托盘」时会被 Rust 侧拦下并隐藏窗口。 */
async function close() {
  await win.close();
}

export function useWindowControls() {
  start();
  return { maximized, minimize, toggleMaximize, close };
}
