<script setup lang="ts">
import { onActivated, onMounted, ref } from "vue";
import TrackList from "../components/TrackList.vue";
import { api, errorText, type Track } from "../api";
import { useInfiniteScroll } from "../composables/useInfiniteScroll";
import { usePlayer } from "../composables/usePlayer";
import { useToast } from "../composables/useToast";

const player = usePlayer();
const { current } = player;
const toast = useToast();

const tracks = ref<Track[]>([]);
const loading = ref(false);
const loadError = ref(false);
const nextStart = ref(0);
const hasMore = ref(false);

async function load(reset: boolean) {
  if (loading.value) return;
  loading.value = true;
  loadError.value = false;
  try {
    const page = await api.favorites(reset ? 0 : nextStart.value, 50);
    tracks.value = reset ? page.items : [...tracks.value, ...page.items];
    nextStart.value = page.nextStart;
    hasMore.value = page.hasMore;
  } catch (error) {
    loadError.value = true;
    toast.error(`加载收藏失败：${errorText(error)}`);
  } finally {
    loading.value = false;
  }
}

// 滚到底自动续页；失败时停在原地，由用户点重试。
const { sentinel } = useInfiniteScroll(
  () => hasMore.value && !loading.value && !loadError.value,
  () => void load(false),
);

onMounted(() => void load(true));
// 页面被 KeepAlive 缓存后 onMounted 不会再跑；首次加载失败时回来要能自动重试。
onActivated(() => {
  if (!tracks.value.length && !loading.value) void load(true);
});

function play(track: Track, index: number) {
  player.playList(tracks.value, index);
}

function playAll() {
  if (tracks.value.length) player.playList(tracks.value, 0);
}
</script>

<template>
  <section class="page">
    <header class="head">
      <h2>我的收藏</h2>
      <span v-if="tracks.length" class="count">已加载 {{ tracks.length }} 首</span>
      <div class="spacer" />
      <button class="ghost" :disabled="!tracks.length" @click="playAll">播放全部</button>
    </header>

    <TrackList
      :tracks="tracks"
      :current-id="current?.id"
      :loading="loading && !tracks.length"
      empty-text="收藏里还没有歌曲"
      @play="play"
    />

    <div v-if="hasMore || loading" ref="sentinel" class="sentinel">
      <span v-if="loading">加载中…</span>
      <button v-else-if="loadError" class="retry" @click="load(false)">加载失败，点击重试</button>
    </div>
  </section>
</template>

<style scoped>
.page {
  padding-bottom: 24px;
}

.head {
  display: flex;
  align-items: baseline;
  gap: 10px;
  margin-bottom: 16px;
}

.head h2 {
  font-size: 18px;
  font-weight: 500;
}

.count {
  font-size: 12px;
  color: var(--faint);
}

.spacer {
  flex: 1;
}

.ghost {
  font-size: 12px;
  color: var(--muted);
  border: 0.5px solid var(--line-strong);
  border-radius: var(--r-sm);
  padding: 3px 10px;
}

.ghost:hover:not(:disabled) {
  color: var(--text);
}

.ghost:disabled {
  opacity: 0.4;
}

.more {
  display: block;
  margin: 18px auto 0;
  font-size: 12px;
  color: var(--muted);
}

.more:hover {
  color: var(--text);
}

.sentinel {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 44px;
  font-size: 12px;
  color: var(--faint);
}

.retry {
  font-size: 12px;
  color: var(--muted);
}

.retry:hover {
  color: var(--text);
}

.hint {
  margin-top: 14px;
  text-align: center;
  font-size: 12px;
  color: var(--faint);
}
</style>
