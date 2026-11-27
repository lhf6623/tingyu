<script setup lang="ts">
import { onActivated, onMounted, ref } from "vue";
import DetailHeader from "../components/DetailHeader.vue";
import TrackList from "../components/TrackList.vue";
import { api, errorText, type Playlist, type Track } from "../api";
import { useInfiniteScroll } from "../composables/useInfiniteScroll";
import { usePlayer } from "../composables/usePlayer";
import { useToast } from "../composables/useToast";

const props = defineProps<{ id: string; kind: string; title: string; coverUrl?: string }>();

const player = usePlayer();
const { current } = player;
const toast = useToast();

const playlist = ref<Playlist | null>(null);
const tracks = ref<Track[]>([]);
const loading = ref(true);
const loadError = ref(false);
const nextStart = ref(0);
const hasMore = ref(false);

async function load(reset: boolean) {
  if (!reset && loading.value) return;
  loading.value = true;
  loadError.value = false;
  try {
    const page = await api.playlistDetail(props.id, props.kind, reset ? 0 : nextStart.value, 50);
    if (page.playlist) playlist.value = page.playlist;
    tracks.value = reset ? page.items : [...tracks.value, ...page.items];
    nextStart.value = page.nextStart;
    hasMore.value = page.hasMore;
  } catch (error) {
    loadError.value = true;
    toast.error(`加载歌单失败：${errorText(error)}`);
  } finally {
    loading.value = false;
  }
}

const { sentinel } = useInfiniteScroll(
  () => hasMore.value && !loading.value && !loadError.value,
  () => void load(false),
);

onMounted(() => void load(true));
onActivated(() => {
  if (!tracks.value.length && !loading.value) void load(true);
});

function playAll() {
  if (tracks.value.length) player.playList(tracks.value, 0);
}

function play(track: Track, index: number) {
  player.playList(tracks.value, index);
}
</script>

<template>
  <section class="page">
    <DetailHeader
      :cover-url="playlist?.coverUrl ?? coverUrl"
      :title="playlist?.title ?? title"
      :subtitle="playlist?.description"
      :meta="playlist?.count ? `${playlist.count} 首` : tracks.length ? `${tracks.length} 首已加载` : ''"
      :playable="tracks.length > 0"
      @play-all="playAll"
    />

    <TrackList
      :tracks="tracks"
      :current-id="current?.id"
      :show-duration="false"
      :loading="loading && !tracks.length"
      empty-text="这个歌单还没有歌曲"
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
</style>
