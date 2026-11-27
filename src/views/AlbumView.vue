<script setup lang="ts">
import { onMounted, ref } from "vue";
import DetailHeader from "../components/DetailHeader.vue";
import TrackList from "../components/TrackList.vue";
import { api, artistText, errorText, type AlbumMeta, type Track } from "../api";
import { usePlayer } from "../composables/usePlayer";
import { useToast } from "../composables/useToast";

const props = defineProps<{ id: string; title: string; coverUrl?: string }>();

const player = usePlayer();
const { current } = player;
const toast = useToast();

const album = ref<AlbumMeta | null>(null);
const tracks = ref<Track[]>([]);
const loading = ref(true);

onMounted(async () => {
  try {
    const page = await api.album(props.id);
    album.value = page.album;
    tracks.value = page.items;
  } catch (error) {
    toast.error(`加载专辑失败：${errorText(error)}`);
  } finally {
    loading.value = false;
  }
});

function playAll() {  if (tracks.value.length) player.playList(tracks.value, 0);
}

function play(track: Track, index: number) {
  player.playList(tracks.value, index);
}

function artistLine(meta: AlbumMeta | null): string {
  if (!meta) return "";
  return meta.artists.map((a) => a.name).join(" / ");
}
</script>

<template>
  <section class="page">
    <DetailHeader
      :cover-url="album?.coverUrl ?? coverUrl"
      :title="album?.title ?? title"
      :subtitle="artistLine(album)"
      :meta="album?.count ? `${album.count} 首` : tracks.length ? `${tracks.length} 首` : ''"
      :playable="tracks.length > 0"
      @play-all="playAll"
    />

    <TrackList
      :tracks="tracks"
      :current-id="current?.id"
      :show-album="false"
      :loading="loading"
      empty-text="这张专辑没有可播放的曲目"
      @play="play"
    />
  </section>
</template>

<style scoped>
.page {
  padding-bottom: 24px;
}
</style>
