<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import TitleBar from "./components/TitleBar.vue";
import SideBar from "./components/SideBar.vue";
import PlayerBar from "./components/PlayerBar.vue";
import QueuePane from "./components/QueuePane.vue";
import LyricsView from "./views/LyricsView.vue";
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
let unlistenTray: (() => void) | undefined;

const VIEWS = {
  discover: DiscoverView,
  favorites: FavoritesView,
  search: SearchView,
  settings: SettingsView,
  playlist: PlaylistView,
  album: AlbumView,
  artist: ArtistView,
  lyrics: LyricsView,
};

const view = computed(() => VIEWS[route.value.name]);
const immersive = computed(() => route.value.name === "lyrics");
const queueVisible = computed(
  () => state.queueOpen && !!auth.value?.authenticated && !immersive.value,
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
  try {
    auth.value = await api.login();
    cache.invalidate();
    nav.goRoot({ name: "discover" });
  } catch (error) {
    toastError(errorText(error));
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
  cache.invalidate();
  nav.goRoot({ name: "discover" });
}

/** 重新登录：先清掉旧会话再走一遍扫码，避免残留的音乐 Token 继续被使用。 */
async function doRelogin() {
  try {
    await api.logout();
  } catch {
    // 清不掉也要继续，否则用户会被卡在旧的失效会话里。
  }
  auth.value = { authenticated: false };
  cache.invalidate();
  await doLogin();
}

function onKeydown(event: KeyboardEvent) {
  const target = event.target as HTMLElement | null;
  if (target && ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName)) return;
  if (event.metaKey || event.ctrlKey || event.altKey) return;

  switch (event.key) {
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
  <div class="app" :class="{ 'with-queue': queueVisible, immersive }">
    <TitleBar />

    <SideBar
      v-if="!immersive"
      :authenticated="!!auth?.authenticated"
      :account-label="auth?.accountLabel"
      :expire-time="auth?.musicExpireTime"
      @logout="doLogout"
      @relogin="doRelogin"
    />

    <main class="main">
      <div v-if="booting" class="center">正在检查登录状态…</div>

      <div v-else-if="!auth?.authenticated" class="gate">
        <h1>登录华为音乐账号</h1>
        <p>在官方的扫码页完成授权。本机只保存音乐会话，播放地址不落盘。</p>
        <button class="primary" :disabled="loggingIn" @click="doLogin">
          {{ loggingIn ? "等待授权…" : "扫码登录" }}
        </button>
      </div>

      <KeepAlive v-else :max="10">
        <component :is="view" :key="keepKey" v-bind="viewProps" />
      </KeepAlive>
    </main>

    <QueuePane v-if="queueVisible" />

    <PlayerBar />

    <div class="toasts">
      <div v-for="item in toasts" :key="item.id" class="toast" :class="{ bad: item.bad }">
        {{ item.text }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.app {
  height: 100%;
  display: grid;
  grid-template-columns: var(--nav-w) minmax(0, 1fr);
  grid-template-rows: 38px minmax(0, 1fr) var(--player-h);
}

.app.with-queue {
  grid-template-columns: var(--nav-w) minmax(0, 1fr) 320px;
}

/* 沉浸歌词页收起侧栏：列宽压成 0，标题栏仍在第二列铺满。 */
.app.immersive {
  grid-template-columns: 0 minmax(0, 1fr);
}

.app.immersive .main {
  padding: 0;
  overflow: hidden;
}

.main {
  grid-column: 2;
  grid-row: 2;
  overflow-y: auto;
  padding: 22px 24px 0;
  min-height: 0;
}

.center {
  padding: 60px 0;
  text-align: center;
  font-size: 12px;
  color: var(--faint);
}

.gate {
  max-width: 340px;
  margin: 16vh auto 0;
  text-align: center;
}

.gate h1 {
  font-size: 18px;
  font-weight: 500;
  margin-bottom: 10px;
}

.gate p {
  font-size: 12px;
  line-height: 1.7;
  color: var(--faint);
  margin-bottom: 22px;
}

.primary {
  background: var(--text);
  color: var(--bg);
  font-size: 13px;
  border-radius: 7px;
  padding: 9px 22px;
}

.primary:disabled {
  opacity: 0.5;
  cursor: default;
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
  background: var(--surface);
  border: 0.5px solid var(--line);
  border-radius: 6px;
  padding: 6px 14px;
  max-width: 60vw;
}

.toast.bad {
  color: var(--accent);
}
</style>
