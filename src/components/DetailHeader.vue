<script setup lang="ts">
import Icon from "./Icon.vue";

defineProps<{
  coverUrl?: string;
  title: string;
  subtitle?: string;
  meta?: string;
  playable?: boolean;
}>();

const emit = defineEmits<{ playAll: [] }>();
</script>

<template>
  <header class="detail">
    <img v-if="coverUrl" class="cover" :src="coverUrl" alt="" />
    <span v-else class="cover" />
    <div class="meta">
      <h2 class="ellipsis">{{ title }}</h2>
      <p v-if="subtitle" class="sub ellipsis">{{ subtitle }}</p>
      <p v-if="meta" class="count">{{ meta }}</p>
    </div>
    <button v-if="playable" class="play-all" @click="emit('playAll')">
      <Icon name="play" :size="14" />
      播放全部
    </button>
  </header>
</template>

<style scoped>
.detail {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 2px 2px 22px;
  margin-bottom: 12px;
}

.cover {
  width: 96px;
  height: 96px;
  flex: none;
  border-radius: var(--r-md);
  background: var(--surface);
  object-fit: cover;
}

.meta {
  min-width: 0;
  flex: 1;
}

.meta h2 {
  font-size: 18px;
  font-weight: 500;
}

.sub {
  margin-top: 4px;
  font-size: 12px;
  color: var(--muted);
}

.count {
  margin-top: 2px;
  font-size: 11px;
  color: var(--faint);
}

.play-all {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: none;
  font-size: 12px;
  color: var(--bg);
  background: var(--text);
  border-radius: var(--r-md);
  padding: 6px 14px;
}
</style>
