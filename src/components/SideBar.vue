<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import Icon from "./Icon.vue";
import type { Playlist } from "../api";
import { useCache } from "../composables/useCache";
import { useNav, type Route } from "../composables/useNav";
import { usePlaylists } from "../composables/usePlaylists";

const props = defineProps<{
  authenticated: boolean;
  accountLabel?: string;
  expireTime?: string;
}>();

const emit = defineEmits<{ logout: []; relogin: [] }>();

const nav = useNav();
const route = nav.route;
const cache = useCache();
// 歌单数据在模块级 store 里，侧栏卸载重建（进/出歌词页）不会重新请求。
const {
  items: playlists,
  loading: playlistsLoading,
  load: loadPlaylists,
  ensure: ensurePlaylists,
  reset: resetPlaylists,
} = usePlaylists();
const menuOpen = ref(false);
const rootRef = ref<HTMLElement | null>(null);

const items: { name: Route["name"]; label: string; icon: string }[] = [
  { name: "discover", label: "发现", icon: "spark" },
  { name: "favorites", label: "我的收藏", icon: "heart" },
];

const active = computed(() => {
  const name = route.value.name;
  // 专辑和歌手是从曲目点进去的，归到「我的收藏」下面。
  if (name === "album" || name === "artist") return "favorites";
  return name;
});

const currentPlaylistId = computed(() =>
  route.value.name === "playlist" ? route.value.id : "",
);

const initial = computed(() => {
  const tail = (props.accountLabel ?? "").replace(/[^0-9A-Za-z]/g, "").slice(-2);
  return tail || "音";
});

const expireText = computed(() => {
  const raw = props.expireTime;
  if (!raw || raw.length < 8) return "未知";
  return `${raw.slice(0, 4)}-${raw.slice(4, 6)}-${raw.slice(6, 8)}`;
});

function select(name: Route["name"]) {
  nav.goRoot({ name } as Route);
}

function openPlaylist(playlist: Playlist) {
  nav.go({
    name: "playlist",
    id: playlist.id,
    kind: playlist.kind,
    title: playlist.title,
    coverUrl: playlist.coverUrl,
  });
}

function openSettings() {
  menuOpen.value = false;
  nav.goRoot({ name: "settings" });
}

function onDocumentClick(event: MouseEvent) {
  if (!menuOpen.value) return;
  if (!rootRef.value?.contains(event.target as Node)) menuOpen.value = false;
}

// 设置页手动刷新时，侧栏的歌单列表也要跟着重拉。
watch(
  () => cache.version.value,
  () => void loadPlaylists(true),
);

watch(
  () => props.authenticated,
  (authenticated) => {
    if (authenticated) void ensurePlaylists();
    else resetPlaylists();
  },
);

onMounted(() => {
  window.addEventListener("click", onDocumentClick);
  void ensurePlaylists();
});

onBeforeUnmount(() => window.removeEventListener("click", onDocumentClick));
</script>

<template>
  <nav ref="rootRef" class="nav">
    <button
      v-for="item in items"
      :key="item.name"
      class="item"
      :class="{ active: active === item.name }"
      @click="select(item.name)"
    >
      <Icon :name="item.icon" :size="17" />
      <span>{{ item.label }}</span>
    </button>

    <div class="section">我的歌单</div>
    <div class="plist">
      <button
        v-for="playlist in playlists"
        :key="playlist.id"
        class="pitem ellipsis"
        :class="{ active: currentPlaylistId === playlist.id }"
        :title="playlist.title"
        @click="openPlaylist(playlist)"
      >
        {{ playlist.title }}
      </button>
      <div v-if="!playlists.length" class="pempty">
        {{ props.authenticated ? (playlistsLoading ? "加载中…" : "还没有歌单") : "登录后显示" }}
      </div>
    </div>

    <div class="foot">
      <button class="user" @click.stop="menuOpen = !menuOpen">
        <span class="avatar">{{ initial }}</span>
        <span class="who ellipsis">{{
          props.authenticated ? props.accountLabel ?? "已登录" : "未登录"
        }}</span>
      </button>
      <button
        class="gear"
        :class="{ on: route.name === 'settings' }"
        title="设置"
        @click="openSettings"
      >
        <Icon name="gear" :size="16" />
      </button>
    </div>

    <div v-if="menuOpen" class="account">
      <div class="account-head">
        <span class="avatar big">{{ initial }}</span>
        <div class="account-meta">
          <span class="account-name">{{
            props.authenticated ? props.accountLabel ?? "已登录" : "未登录"
          }}</span>
          <span class="account-sub">音乐会话到期 {{ expireText }}</span>
        </div>
      </div>
      <button
        v-if="props.authenticated"
        class="account-action"
        @click="menuOpen = false; emit('relogin')"
      >
        重新登录
      </button>
      <button
        v-if="props.authenticated"
        class="account-action danger"
        @click="menuOpen = false; emit('logout')"
      >
        退出登录
      </button>
      <button v-else class="account-action" @click="menuOpen = false; emit('relogin')">
        扫码登录
      </button>
    </div>
  </nav>
</template>

<style scoped>
.nav {
  grid-column: 1;
  grid-row: 1 / -1;
  display: flex;
  flex-direction: column;
  background: var(--surface);
  border-right: 0.5px solid var(--line);
  padding: 14px 0 10px;
  min-height: 0;
  position: relative;
}

.item {
  display: flex;
  align-items: center;
  gap: 11px;
  height: 36px;
  margin: 0 8px;
  padding: 0 10px;
  font-size: 13px;
  color: var(--muted);
  border-radius: var(--r-md);
  text-align: left;
  flex: none;
  transition: background 0.13s ease, color 0.13s ease;
}

.item:hover {
  color: var(--text);
  background: var(--bg);
}

.item.active {
  color: var(--text);
  font-weight: 500;
  background: var(--bg);
}

.section {
  padding: 18px 18px 8px;
  font-size: 11px;
  color: var(--faint);
  flex: none;
}

.plist {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding-bottom: 8px;
}

.pitem {
  display: block;
  width: calc(100% - 16px);
  margin: 0 8px;
  text-align: left;
  padding: 6px 10px;
  font-size: 12px;
  color: var(--muted);
  border-radius: var(--r-sm);
  transition: background 0.13s ease, color 0.13s ease;
}

.pitem:hover {
  color: var(--text);
  background: var(--bg);
}

.pitem.active {
  color: var(--text);
  background: var(--bg);
}

.pempty {
  padding: 6px 18px;
  font-size: 11px;
  color: var(--faint);
}

.foot {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 10px 0 12px;
  border-top: 0.5px solid var(--line);
  flex: none;
}

.user {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
  padding: 5px 6px;
  border-radius: var(--r-md);
  text-align: left;
}

.user:hover {
  background: var(--bg);
}

.avatar {
  width: 22px;
  height: 22px;
  flex: none;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: var(--text);
  color: var(--bg);
  font-size: 10px;
  font-variant-numeric: tabular-nums;
}

.avatar.big {
  width: 30px;
  height: 30px;
  font-size: 11px;
}

.who {
  font-size: 11px;
  color: var(--muted);
}

.gear {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  flex: none;
  border-radius: var(--r-sm);
  color: var(--muted);
}

.gear:hover,
.gear.on {
  color: var(--text);
  background: var(--bg);
}

.account {
  position: absolute;
  left: 10px;
  right: 10px;
  bottom: 46px;
  background: var(--bg);
  border: 0.5px solid var(--line-strong);
  border-radius: var(--r-md);
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  z-index: 30;
}

.account-head {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 2px 4px 10px;
  border-bottom: 0.5px solid var(--line);
  margin-bottom: 6px;
}

.account-meta {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.account-name {
  font-size: 12px;
  font-weight: 500;
}

.account-sub {
  font-size: 11px;
  color: var(--faint);
}

.account-action {
  font-size: 12px;
  color: var(--text);
  text-align: left;
  padding: 6px;
  border-radius: var(--r-sm);
}

.account-action:hover {
  background: var(--surface);
}

.account-action.danger {
  color: var(--accent);
}
</style>
