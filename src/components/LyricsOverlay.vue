<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import Icon from "./Icon.vue";
import { artistText } from "../api";
import { usePlayer } from "../composables/usePlayer";

/**
 * 歌词覆盖层。
 *
 * 不做成独立路由：走路由就得卸载下层的侧栏和列表，返回时全部重建，
 * 而歌词本身是「在播放时临时看一眼」的操作，关掉应当原样回到原位置。
 * 这一层只盖住内容区与侧栏，标题栏（窗口按钮）和播放栏（播放控制）留在上层。
 */
const player = usePlayer();
const { state, current } = player;

const scroller = ref<HTMLDivElement | null>(null);
const lineEls = ref<HTMLElement[]>([]);

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

/** 把当前句滚到垂直居中；打开时直接定位，播放中跟随时用平滑滚动。 */
async function scrollToActive(smooth: boolean) {
  const index = activeIndex.value;
  if (index < 0) return;
  await nextTick();
  const el = lineEls.value[index];
  const box = scroller.value;
  if (!el || !box) return;
  const target = el.offsetTop - box.clientHeight / 2 + el.clientHeight / 2;
  box.scrollTo({ top: Math.max(target, 0), behavior: smooth ? "smooth" : "auto" });
}

watch(
  () => activeIndex.value,
  () => void scrollToActive(true),
);

// 关闭后 DOM 被移除，重开时不会触发 activeIndex 变化，需要主动定位一次。
watch(
  () => state.lyricsOpen,
  (open) => {
    if (open) void scrollToActive(false);
    else lineEls.value = [];
  },
);

watch(
  () => current.value?.id,
  () => {
    lineEls.value = [];
    scroller.value?.scrollTo({ top: 0 });
  },
);

const backdropStyle = computed(() =>
  current.value?.coverUrl ? { backgroundImage: `url("${current.value.coverUrl}")` } : {},
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
  <div v-if="state.lyricsOpen" class="overlay">
    <div class="backdrop" :style="backdropStyle" />

    <header class="top">
      <button class="back" @click="player.closeLyrics()">
        <Icon name="back" :size="15" />
        收起
      </button>
      <span class="hint">
        {{ state.lyrics?.synced ? "逐行歌词 · 点任意一句可跳转 · Esc 收起" : "" }}
      </span>
      <div class="spacer" />
      <button class="close" title="收起歌词（Esc）" @click="player.closeLyrics()">
        <Icon name="close" :size="15" />
      </button>
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
        <div v-else-if="!state.lyrics" class="empty small">这首歌没有可用歌词</div>
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
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  top: var(--titlebar-h);
  right: 0;
  bottom: var(--player-h);
  left: 0;
  z-index: 40;
  display: flex;
  flex-direction: column;
  padding: 18px 28px 0;
  overflow: hidden;
  background: var(--bg);
}

/* 用当前封面做一层模糊底色，避免整块纯色显得空。 */
.backdrop {
  position: absolute;
  inset: -15%;
  background-position: center;
  background-size: cover;
  filter: blur(64px) saturate(1.3);
  opacity: 0.2;
  pointer-events: none;
}

.top,
.stage,
.empty {
  position: relative;
  z-index: 1;
}

.top {
  display: flex;
  align-items: center;
  gap: 14px;
  flex: none;
}

.back,
.close {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--muted);
  padding: 4px 8px;
  border-radius: 5px;
}

.back {
  padding-left: 4px;
}

.back:hover,
.close:hover {
  color: var(--text);
  background: var(--surface);
}

.hint {
  font-size: 11px;
  color: var(--faint);
}

.spacer {
  flex: 1;
}

.stage {
  display: flex;
  gap: 48px;
  flex: 1;
  min-height: 0;
  padding: 20px 0 0;
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
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.18);
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
