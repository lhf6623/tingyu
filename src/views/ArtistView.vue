<script setup lang="ts">
import { onMounted, ref } from "vue";
import DetailHeader from "../components/DetailHeader.vue";
import TrackList from "../components/TrackList.vue";
import { api, errorText, type AlbumCard, type ArtistMeta, type Track } from "../api";
import { useNav } from "../composables/useNav";
import { usePlayer } from "../composables/usePlayer";
import { useToast } from "../composables/useToast";

const props = defineProps<{ id: string; title: string }>();

const nav = useNav();
const player = usePlayer();
const { current } = player;
const toast = useToast();

const artist = ref<ArtistMeta | null>(null);
const tracks = ref<Track[]>([]);
const albums = ref<AlbumCard[]>([]);
const loading = ref(true);

onMounted(async () => {
  try {
    const page = await api.artist(props.id);
    artist.value = page.artist;
    tracks.value = page.items;
    albums.value = page.albums;
  } catch (error) {
    toast.error(`加载歌手失败：${errorText(error)}`);
  } finally {
    loading.value = false;
  }
});

function playAll() {
  if (tracks.value.length) player.playList(tracks.value, 0);
}

function play(track: Track, index: number) {
  player.playList(tracks.value, index);
}

function openAlbum(album: AlbumCard) {
  nav.go({ name: "album", id: album.id, title: album.title, coverUrl: album.coverUrl });
}
</script>

<template>
  <section class="page">
    <DetailHeader
      :cover-url="artist?.coverUrl"
      :title="artist?.name ?? title"
      :meta="tracks.length ? `热门 ${tracks.length} 首` : ''"
      :playable="tracks.length > 0"
      @back="nav.back()"
      @play-all="playAll"
    />

    <TrackList
      :tracks="tracks"
      :current-id="current?.id"
      :show-album="false"
      :loading="loading"
      empty-text="没有可播放的曲目"
      @play="play"
    />

    <template v-if="albums.length">
      <h3 class="block-title">专辑</h3>
      <div class="albums">
        <button v-for="album in albums" :key="album.id" class="card" @click="openAlbum(album)">
          <img v-if="album.coverUrl" class="cover" :src="album.coverUrl" alt="" loading="lazy" />
          <span v-else class="cover" />
          <span class="title ellipsis">{{ album.title }}</span>
        </button>
      </div>
    </template>
  </section>
</template>

<style scoped>
.page {
  padding-bottom: 24px;
}

.block-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--muted);
  margin: 26px 0 10px;
}

.albums {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 16px 14px;
}

.card {
  display: flex;
  flex-direction: column;
  gap: 6px;
  text-align: left;
  min-width: 0;
}

.cover {
  width: 100%;
  aspect-ratio: 1;
  border-radius: 6px;
  background: var(--surface);
  object-fit: cover;
}

.title {
  font-size: 12px;
}
</style>
