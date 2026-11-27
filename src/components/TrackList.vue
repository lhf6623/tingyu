<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import Icon from "./Icon.vue";
import { api, errorText, formatDuration, type Artist, type Track } from "../api";
import { useLibrary } from "../composables/useLibrary";
import { useNav } from "../composables/useNav";
import { usePlayer } from "../composables/usePlayer";
import { useToast } from "../composables/useToast";

withDefaults(
  defineProps<{
    tracks: Track[];
    currentId?: string;
    showIndex?: boolean;
    showAlbum?: boolean;
    dense?: boolean;
    emptyText?: string;
    loading?: boolean;
  }>(),
  { showIndex: true, showAlbum: true, dense: false, emptyText: "这里还没有歌曲", loading: false },
);

const emit = defineEmits<{ play: [track: Track, index: number] }>();

const toast = useToast();
const nav = useNav();
const player = usePlayer();
const library = useLibrary();
const pending = ref(new Set<string>());
const menu = ref<{ x: number; y: number; track: Track } | null>(null);

function openArtist(artist: Artist, event: MouseEvent) {
  event.stopPropagation();
  if (!artist.id) return;
  nav.go({ name: "artist", id: artist.id, title: artist.name });
}

async function toggleFavorite(track: Track) {
  if (pending.value.has(track.id)) return;
  pending.value = new Set(pending.value).add(track.id);
  try {
    // 先查真实状态再取反，不盲切布尔值。
    const exists = await api.favoriteExist(track.id);
    await api.setFavorite(track.id, !exists);
    library.setFavorite(track.id, !exists);
    toast.info(exists ? "已取消收藏" : "已收藏");
  } catch (error) {
    toast.error(`操作失败：${errorText(error)}`);
  } finally {
    const next = new Set(pending.value);
    next.delete(track.id);
    pending.value = next;
  }
}

function openMenu(track: Track, event: MouseEvent) {
  event.preventDefault();
  // 菜单贴着窗口右下角会溢出，先夹回可视区内。
  const x = Math.min(event.clientX, window.innerWidth - 132);
  const y = Math.min(event.clientY, window.innerHeight - 80);
  menu.value = { x, y, track };
}

function closeMenu() {
  menu.value = null;
}

function queueNext() {
  if (!menu.value) return;
  player.playNext(menu.value.track);
  toast.info(`已插入到下一首：${menu.value.track.title}`);
  closeMenu();
}

function queueAppend() {
  if (!menu.value) return;
  player.appendToQueue(menu.value.track);
  toast.info(`已加入队列：${menu.value.track.title}`);
  closeMenu();
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") closeMenu();
}

onMounted(() => {
  window.addEventListener("click", closeMenu);
  window.addEventListener("contextmenu", closeMenu, true);
  window.addEventListener("keydown", onKeydown);
  window.addEventListener("blur", closeMenu);
});

onBeforeUnmount(() => {
  window.removeEventListener("click", closeMenu);
  window.removeEventListener("contextmenu", closeMenu, true);
  window.removeEventListener("keydown", onKeydown);
  window.removeEventListener("blur", closeMenu);
});
</script>

<template>
  <div v-if="loading" class="hint">加载中…</div>
  <div v-else-if="!tracks.length" class="hint">{{ emptyText }}</div>
  <div v-else class="list" :class="{ dense }">
    <div
      v-for="(track, index) in tracks"
      :key="`${track.id}-${index}`"
      class="row"
      :class="{ active: currentId === track.id }"
      @contextmenu="openMenu(track, $event)"
    >
      <span v-if="showIndex" class="slot">
        <span class="idx">{{ String(index + 1).padStart(2, "0") }}</span>
        <button class="go" title="播放" @click.stop="emit('play', track, index)">
          <Icon name="play" :size="13" />
        </button>
      </span>
      <img v-if="track.coverUrl" class="cover" :src="track.coverUrl" alt="" loading="lazy" />
      <span v-else class="cover" />
      <span class="meta">
        <span class="title ellipsis">{{ track.title }}</span>
        <span class="sub ellipsis">
          <template v-if="track.artists.length">
            <template v-for="(artist, at) in track.artists" :key="artist.id ?? at">
              <span v-if="at" class="sep"> / </span>
              <span :class="{ link: !!artist.id }" @click="openArtist(artist, $event)">{{
                artist.name
              }}</span>
            </template>
          </template>
          <template v-else>未知艺人</template>
        </span>
      </span>
      <span v-if="showAlbum" class="album ellipsis">{{ track.album?.title ?? "" }}</span>
      <span class="dur">{{ formatDuration(track.durationSeconds) }}</span>
      <button
        class="fav"
        :class="{ on: library.isFavorite(track), busy: pending.has(track.id) }"
        title="收藏 / 取消收藏"
        @click.stop="toggleFavorite(track)"
      >
        <Icon :name="library.isFavorite(track) ? 'heart-filled' : 'heart'" :size="15" />
      </button>
    </div>

    <div v-if="menu" class="ctx" :style="{ left: menu.x + 'px', top: menu.y + 'px' }">
      <button @click="queueNext">下一首播放</button>
      <button @click="queueAppend">加入队列</button>
    </div>
  </div>
</template>

<style scoped>
.list {
  display: flex;
  flex-direction: column;
}

.row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 10px 8px 2px;
  border-bottom: 0.5px solid var(--line);
  border-radius: var(--r-sm);
  text-align: left;
  width: 100%;
}

.dense .row {
  padding: 6px 10px 6px 2px;
}

.row:hover {
  background: var(--surface);
}

.row.active .title {
  color: var(--accent);
}

.slot {
  position: relative;
  width: 24px;
  height: 22px;
  flex: none;
}

.idx,
.go {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.idx {
  font-size: 11px;
  color: var(--faint);
  font-variant-numeric: tabular-nums;
}

.row.active .idx {
  color: var(--accent);
}

.go {
  opacity: 0;
  color: var(--text);
}

.row:hover .go {
  opacity: 1;
}

.row:hover .idx {
  opacity: 0;
}

.cover {
  width: 32px;
  height: 32px;
  flex: none;
  border-radius: var(--r-sm);
  background: var(--surface);
  object-fit: cover;
}

.dense .cover {
  width: 28px;
  height: 28px;
}

.meta {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
}

.title {
  font-size: 13px;
}

.sub {
  font-size: 11px;
  color: var(--faint);
}

.sep {
  color: var(--faint);
}

.link:hover {
  color: var(--text);
  text-decoration: underline;
}

.album {
  width: 170px;
  flex: none;
  font-size: 11px;
  color: var(--faint);
}

.dur {
  width: 42px;
  flex: none;
  font-size: 11px;
  color: var(--faint);
  font-variant-numeric: tabular-nums;
  text-align: right;
}

.fav {
  width: 22px;
  flex: none;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--faint);
  opacity: 0;
}

.row:hover .fav,
.fav.on {
  opacity: 1;
}

.fav.on {
  color: var(--accent);
}

.fav.busy {
  opacity: 0.4;
}

.hint {
  padding: 28px 0;
  text-align: center;
  font-size: 12px;
  color: var(--faint);
}

.ctx {
  position: fixed;
  z-index: 40;
  display: flex;
  flex-direction: column;
  min-width: 124px;
  padding: 4px;
  background: var(--bg);
  border: 0.5px solid var(--line-strong);
  border-radius: var(--r-md);
}

.ctx button {
  font-size: 12px;
  color: var(--text);
  text-align: left;
  padding: 6px 10px;
  border-radius: var(--r-sm);
}

.ctx button:hover {
  background: var(--surface);
}
</style>
