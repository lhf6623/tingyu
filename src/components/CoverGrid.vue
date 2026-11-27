<script setup lang="ts">
import type { Playlist } from "../api";

defineProps<{ items: Playlist[]; emptyText?: string }>();
const emit = defineEmits<{ select: [playlist: Playlist] }>();
</script>

<template>
  <div v-if="!items.length" class="empty">{{ emptyText ?? "还没有歌单" }}</div>
  <div v-else class="grid">
    <button v-for="item in items" :key="item.id" class="card" @click="emit('select', item)">
      <img v-if="item.coverUrl" class="cover" :src="item.coverUrl" alt="" loading="lazy" />
      <span v-else class="cover" />
      <span class="title ellipsis">{{ item.title }}</span>
      <span class="sub">{{ item.count ? `${item.count} 首` : "歌单" }}</span>
    </button>
  </div>
</template>

<style scoped>
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(132px, 1fr));
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
  border-radius: var(--r-md);
  background: var(--surface);
  object-fit: cover;
  transition: opacity 0.15s ease;
}

.card:hover .cover {
  opacity: 0.82;
}

.title {
  font-size: 12px;
}

.sub {
  font-size: 11px;
  color: var(--faint);
}

.empty {
  padding: 24px 0;
  font-size: 12px;
  color: var(--faint);
}
</style>
