<script setup lang="ts">
import { onActivated, onMounted, ref } from "vue";
import TrackList from "../components/TrackList.vue";
import { api, errorText, type Track } from "../api";
import { usePlayer } from "../composables/usePlayer";
import { useToast } from "../composables/useToast";

const player = usePlayer();
const { current } = player;
const toast = useToast();

const recent = ref<Track[]>([]);
const loading = ref(false);
const loaded = ref(false);

async function loadRecent() {
  if (loading.value) return;
  loading.value = true;
  try {
    recent.value = await api.recent();
    loaded.value = true;
  } catch (error) {
    toast.error(`加载失败：${errorText(error)}`);
  } finally {
    loading.value = false;
  }
}

onMounted(() => void loadRecent());
onActivated(() => {
  if (!loaded.value) void loadRecent();
});

function playRecent(track: Track, index: number) {
  player.playList(recent.value, index);
}
</script>

<template>
  <section class="page">
    <header class="head">
      <h2>发现</h2>
    </header>

    <div class="block">
      <div class="block-head">
        <h3>最近播放</h3>
        <span v-if="recent.length" class="count">{{ recent.length }} 首</span>
      </div>
      <TrackList
        :tracks="recent"
        :current-id="current?.id"
        :show-album="false"
        :dense="true"
        :loading="!loaded && loading"
        empty-text="账号里还没有播放记录"
        @play="playRecent"
      />
    </div>

  </section>
</template>

<style scoped>
.page {
  padding-bottom: 24px;
}

.head {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 20px;
}

.head h2 {
  font-size: 18px;
  font-weight: 500;
}

.spacer {
  flex: 1;
}

.block {
  margin-bottom: 26px;
}

.block-head {
  display: flex;
  align-items: baseline;
  gap: 10px;
  margin-bottom: 8px;
}

.block-head h3 {
  font-size: 12px;
  font-weight: 500;
  color: var(--muted);
}

.count {
  font-size: 11px;
  color: var(--faint);
}

.link {
  font-size: 11px;
  color: var(--faint);
}

.link:hover {
  color: var(--text);
}

.hint {
  padding: 20px 0;
  font-size: 12px;
  color: var(--faint);
}
</style>
