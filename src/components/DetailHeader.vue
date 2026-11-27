<script setup lang="ts">
import Icon from "./Icon.vue";

defineProps<{
  coverUrl?: string;
  title: string;
  subtitle?: string;
  meta?: string;
  playable?: boolean;
}>();

const emit = defineEmits<{ back: []; playAll: [] }>();
</script>

<template>
  <header class="detail">
    <button class="back" title="返回" @click="emit('back')">
      <Icon name="back" :size="16" />
    </button>
    <img v-if="coverUrl" class="cover" :src="coverUrl" alt="" />
    <span v-else class="cover" />
    <div class="meta">
      <h2 class="ellipsis">{{ title }}</h2>
      <p v-if="subtitle" class="sub ellipsis">{{ subtitle }}</p>
      <p v-if="meta" class="count">{{ meta }}</p>
    </div>
    <button v-if="playable" class="play-all" @click="emit('playAll')">
      <Icon name="play" :size="12" />
      播放全部
    </button>
  </header>
</template>

<style scoped>
.detail {
  display: flex;
  align-items: center;
  gap: 16px;
  padding-bottom: 20px;
  border-bottom: 0.5px solid var(--line);
  margin-bottom: 8px;
}

.back {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: 5px;
  color: var(--muted);
  flex: none;
}

.back:hover {
  color: var(--text);
  background: var(--surface);
}

.cover {
  width: 96px;
  height: 96px;
  flex: none;
  border-radius: 6px;
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
  border-radius: 6px;
  padding: 6px 14px;
}
</style>
