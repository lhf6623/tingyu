<script setup lang="ts">
import { computed, nextTick, onActivated, onDeactivated, ref, watch } from "vue";
import Icon from "../components/Icon.vue";
import { artistText } from "../api";
import { useNav } from "../composables/useNav";
import { usePlayer } from "../composables/usePlayer";

const nav = useNav();
const player = usePlayer();
const { state, current } = player;

const scroller = ref<HTMLDivElement | null>(null);
const lineEls = ref<HTMLElement[]>([]);
// 页面被 KeepAlive 缓存后监听仍在跑，离开时不该继续算滚动位置。
const pageActive = ref(true);
onActivated(() => (pageActive.value = true));
onDeactivated(() => (pageActive.value = false));

const activeIndex = computed(() => {
  const doc = state.lyrics;
  if (!doc?.synced) return -1;
  const positionMs = state.position * 1000;
  let found = -1;
  for (let i = 0; i < doc.lines.length; i += 1) {
    const time = doc.lines[i].timeMs;
    if (typeof time !== "number") continue;
    if (time <= positionMs) found = i;
    else break;
  }
  return found;
});

watch(
  () => activeIndex.value,
  async (index) => {
    if (index < 0 || !pageActive.value) return;
    await nextTick();
    const el = lineEls.value[index];
    const box = scroller.value;
    if (!el || !box) return;
    const target = el.offsetTop - box.clientHeight / 2 + el.clientHeight / 2;
    box.scrollTo({ top: Math.max(target, 0), behavior: "smooth" });
  },
);

watch(
  () => current.value?.id,
  () => {
    lineEls.value = [];
    scroller.value?.scrollTo({ top: 0 });
  },
);

function setLineRef(el: Element | null, index: number) {
  if (el instanceof HTMLElement) lineEls.value[index] = el;
}

function jump(timeMs: number | null) {
  if (timeMs === null || !state.duration) return;
  player.seek(timeMs / 1000 / state.duration);
}
</script>

<template>
  <section class="page">
    <header class="top">
      <button class="back" @click="nav.back()">
        <Icon name="back" :size="15" />
        返回
      </button>
      <span class="hint">
        {{ state.lyrics?.synced ? "逐行歌词 · 点任意一句可跳转" : "" }}
      </span>
    </header>

    <div v-if="!current" class="empty">还没有播放中的歌曲</div>

    <div v-else class="stage">
      <aside class="meta-col">
        <img v-if="current.coverUrl" class="cover" :src="current.coverUrl" alt="" />
        <span v-else class="cover" />
        <h1 class="title">{{ current.title }}</h1>
        <p class="artists">{{ artistText(current) || "未知艺人" }}</p>
        <p v-if="current.album" class="album">{{ current.album.title }}</p>
      </aside>

      <div class="lyric-col">
        <div v-if="state.lyricsLoading" class="empty small">正在获取歌词…</div>
        <div v-else-if="!state.lyrics" class="empty small">
          这首歌没有可用歌词
        </div>
        <div v-else ref="scroller" class="lines">
          <p
            v-for="(line, index) in state.lyrics.lines"
            :key="index"
            :ref="(el) => setLineRef(el as Element | null, index)"
            class="line"
            :class="{ active: index === activeIndex, tappable: line.timeMs !== null }"
            @click="jump(line.timeMs)"
          >
            <span class="text">{{ line.text || "♪" }}</span>
            <span v-if="line.translation" class="translation">{{ line.translation }}</span>
          </p>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.page {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 18px 28px 0;
  min-height: 0;
}

.top {
  display: flex;
  align-items: center;
  gap: 14px;
  flex: none;
}

.back {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--muted);
  padding: 4px 8px 4px 4px;
  border-radius: 5px;
}

.back:hover {
  color: var(--text);
  background: var(--surface);
}

.hint {
  font-size: 11px;
  color: var(--faint);
}

.stage {
  display: flex;
  gap: 48px;
  flex: 1;
  min-height: 0;
  padding: 28px 0 0;
}

.meta-col {
  width: 240px;
  flex: none;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
}

.cover {
  width: 240px;
  height: 240px;
  border-radius: 8px;
  background: var(--surface);
  object-fit: cover;
  margin-bottom: 16px;
}

.title {
  font-size: 18px;
  font-weight: 500;
  line-height: 1.4;
  word-break: break-word;
}

.artists {
  font-size: 13px;
  color: var(--muted);
}

.album {
  font-size: 12px;
  color: var(--faint);
}

.lyric-col {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.lines {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0 40vh;
  scrollbar-width: thin;
  max-width: 640px;
}

.line {
  padding: 10px 0;
  font-size: 15px;
  line-height: 1.6;
  color: var(--faint);
  transition: color 0.15s ease;
}

.line.tappable {
  cursor: pointer;
}

.line:hover {
  color: var(--muted);
}

.line.active {
  color: var(--text);
  font-weight: 500;
}

.text {
  display: block;
}

.translation {
  display: block;
  margin-top: 2px;
  font-size: 12px;
  color: var(--faint);
  font-weight: 400;
}

.empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  color: var(--faint);
}

.empty.small {
  font-size: 12px;
  padding: 40px 0;
  align-items: flex-start;
}
</style>
