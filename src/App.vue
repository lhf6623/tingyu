<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import TitleBar from "./components/TitleBar.vue";
import SideBar from "./components/SideBar.vue";
import PlayerBar from "./components/PlayerBar.vue";
import QueuePane from "./components/QueuePane.vue";
import LyricsOverlay from "./components/LyricsOverlay.vue";
import LoginView from "./components/LoginView.vue";
import DiscoverView from "./views/DiscoverView.vue";
import FavoritesView from "./views/FavoritesView.vue";
import SearchView from "./views/SearchView.vue";
import SettingsView from "./views/SettingsView.vue";
import PlaylistView from "./views/PlaylistView.vue";
import AlbumView from "./views/AlbumView.vue";
import ArtistView from "./views/ArtistView.vue";
import { api, errorText, type AuthStatus } from "./api";
import { useCache } from "./composables/useCache";
import { useNav } from "./composables/useNav";
import { usePlayer } from "./composables/usePlayer";
import { useToast } from "./composables/useToast";

const nav = useNav();
const route = nav.route;
const player = usePlayer();
const { state } = player;
const { toasts, error: toastError } = useToast();

const auth = ref<AuthStatus | null>(null);
const cache = useCache();
const booting = ref(true);
const loggingIn = ref(false);
const loginError = ref("");
let unlistenTray: (() => void) | undefined;

const VIEWS = {
  discover: DiscoverView,
  favorites: FavoritesView,
  search: SearchView,
  settings: SettingsView,
  playlist: PlaylistView,
  album: AlbumView,
  artist: ArtistView,
};

const view = computed(() => VIEWS[route.value.name]);
const queueVisible = computed(
  () => state.queueOpen && !!auth.value?.authenticated && !state.lyricsOpen,
);
const viewKey = computed(() => {
  const current = route.value;
  return current.name === "playlist" || current.name === "album" || current.name === "artist"
    ? `${current.name}:${current.id}`
    : current.name;
});

const keepKey = computed(() => `${cache.version.value}:${viewKey.value}`);

const viewProps = computed(() => {
  const current = route.value;
  switch (current.name) {
    case "playlist":
      return { id: current.id, kind: current.kind, title: current.title, coverUrl: current.coverUrl };
    case "album":
      return { id: current.id, title: current.title, coverUrl: current.coverUrl };
    case "artist":
      return { id: current.id, title: current.title };
    case "settings":
      return { accountLabel: auth.value?.accountLabel, expireTime: auth.value?.musicExpireTime };
    default:
      return {};
  }
});

async function refreshAuth() {
  try {
    auth.value = await api.authStatus();
  } catch (error) {
    toastError(`无法读取登录状态：${errorText(error)}`);
  } finally {
    booting.value = false;
  }
}

async function doLogin() {
  if (loggingIn.value) return;
  loggingIn.value = true;
  loginError.value = "";
  try {
    auth.value = await api.login();
    cache.invalidate();
    nav.reset({ name: "discover" });
  } catch (error) {
    loginError.value = errorText(error);
  } finally {
    loggingIn.value = false;
  }
}

async function doLogout() {
  try {
    await api.logout();
  } catch (error) {
    toastError(errorText(error));
  }
  auth.value = { authenticated: false };
  // 退出后覆盖层不能继续挂在登录门上面。
  player.closeLyrics();
  cache.invalidate();
  nav.reset({ name: "discover" });
}

/** 重新登录：先清掉旧会话再走一遍扫码，避免残留的音乐 Token 继续被使用。 */
async function doRelogin() {
  try {
    await api.logout();
  } catch {
    // 清不掉也要继续，否则用户会被卡在旧的失效会话里。
  }
  auth.value = { authenticated: false };
  player.closeLyrics();
  cache.invalidate();
  await doLogin();
}

function onKeydown(event: KeyboardEvent) {
  // 未登录时主界面没有挂载，快捷键不应驱动一个看不见的播放器。
  if (!auth.value?.authenticated) return;
  const target = event.target as HTMLElement | null;
  if (target && ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName)) return;
  if (event.metaKey || event.ctrlKey || event.altKey) return;

  switch (event.key) {
    case "Escape":
      // 歌词覆盖层吃掉 Esc，避免顺带触发其他关闭行为。
      if (!state.lyricsOpen) return;
      event.preventDefault();
      player.closeLyrics();
      break;
    case " ":
      event.preventDefault();
      player.toggle();
      break;
    case "ArrowRight":
      event.preventDefault();
      player.seek(Math.min(player.progress.value + 5 / Math.max(state.duration, 1), 1));
      break;
    case "ArrowLeft":
      event.preventDefault();
      player.seek(Math.max(player.progress.value - 5 / Math.max(state.duration, 1), 0));
      break;
    case "ArrowUp":
      event.preventDefault();
      player.setVolume(state.volume + 0.05);
      break;
    case "ArrowDown":
      event.preventDefault();
      player.setVolume(state.volume - 0.05);
      break;
    default:
      break;
  }
}

onMounted(async () => {
  void player.hydrate();
  void refreshAuth();
  window.addEventListener("keydown", onKeydown);
  // 托盘菜单的播放控制由 Rust 侧发事件过来，音频元素只在前端。
  unlistenTray = await listen<string>("tray-player", (event) => {
    if (!auth.value?.authenticated) return;
    if (event.payload === "toggle") player.toggle();
    else if (event.payload === "next") player.next();
    else if (event.payload === "previous") player.previous();
  });
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeydown);
  unlistenTray?.();
});
</script>

<template>
  <LoginView
    v-if="!auth?.authenticated"
    :booting="booting"
    :logging-in="loggingIn"
    :error="loginError"
    @login="doLogin"
  />

  <div v-else class="app">
    <SideBar
      :authenticated="!!auth?.authenticated"
      :account-label="auth?.accountLabel"
      :expire-time="auth?.musicExpireTime"
      @logout="doLogout"
      @relogin="doRelogin"
    />

    <!-- 右侧整块是浮在底层之上的「第二层」，四边留白 + 圆角 + 投影做出视觉差 -->
    <div class="stage">
      <TitleBar />

      <div class="stage-body">
        <main class="main">
          <KeepAlive :max="10">
            <component :is="view" :key="keepKey" v-bind="viewProps" />
          </KeepAlive>
        </main>

        <QueuePane v-if="queueVisible" />
      </div>

      <PlayerBar />
    </div>

    <LyricsOverlay />
  </div>

  <div class="toasts">
    <div v-for="item in toasts" :key="item.id" class="toast" :class="{ bad: item.bad }">
      {{ item.text }}
    </div>
  </div>
</template>

<style scoped>
/* 整窗是底层：左侧导航直接坐在这层色上；右侧 stage 与它之间留 10px 沟，
 * 让投影和圆角有地方落，形成「左在后、右在前」的层次。 */
.app {
  height: 100%;
  display: grid;
  grid-template-columns: var(--nav-w) minmax(0, 1fr);
  grid-template-rows: minmax(0, 1fr);
  column-gap: 10px;
  padding: 10px 10px 10px 0;
  background: var(--base);
}

.stage {
  grid-column: 2;
  grid-row: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
  background: var(--bg);
  border-radius: var(--r-xl);
  box-shadow: var(--stage-shadow);
}

.stage-body {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
}

.main {
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow-y: auto;
  padding: 20px 24px 0;
}

.toasts {
  position: fixed;
  left: 50%;
  bottom: calc(var(--player-h) + 16px);
  transform: translateX(-50%);
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: center;
  pointer-events: none;
  z-index: 20;
}

.toast {
  font-size: 12px;
  color: var(--muted);
  background: var(--bg);
  box-shadow: var(--pop-shadow);
  border-radius: var(--r-md);
  padding: 6px 14px;
  max-width: 60vw;
}

.toast.bad {
  color: var(--danger);
}
</style>
