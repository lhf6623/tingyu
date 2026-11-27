<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from "vue";
import Icon from "./Icon.vue";
import LyricsControls from "./LyricsControls.vue";
import { artistText } from "../api";
import { usePlayer } from "../composables/usePlayer";
import { useWindowControls } from "../composables/useWindowControls";

/**
 * 歌词覆盖层。
 *
 * 不做成独立路由：走路由就得卸载下层的侧栏和列表，返回时全部重建，
 * 而歌词本身是「在播放时临时看一眼」的操作，关掉应当原样回到原位置。
 * 这一层盖住整个窗口（含标题栏与播放栏），窗口按钮与播放控制由它自己提供。
 */
const player = usePlayer();
const { state, current } = player;
// 覆盖层盖住标题栏后，窗口按钮必须由这里提供，否则关不掉也拖不动。
const { maximized, minimize, toggleMaximize, close } = useWindowControls();

const scroller = ref<HTMLDivElement | null>(null);
const lineEls = ref<HTMLElement[]>([]);
/**
 * 上下留白 = 可视高度的一半，第一句和最后一句也能滚到正中。
 * 固定 vh 在窗口高度不同时做不到这件事，所以按实际高度算。
 */
const pad = ref(0);
let resizeObserver: ResizeObserver | undefined;

function syncPad() {
  const box = scroller.value;
  pad.value = box ? Math.max(box.clientHeight / 2 - 22, 0) : 0;
}

/**
 * 歌词里的空行（间奏占位）原来会渲染成一排 ♪，很杂。
 * 这里只保留有文字的行；时间戳信息仍然保留，跳转和定位不受影响。
 */
const visibleLines = computed(() =>
  (state.lyrics?.lines ?? []).filter(
    (line) => line.text.trim().length > 0 || (line.translation ?? "").trim().length > 0,
  ),
);

const activeIndex = computed(() => {
  if (!state.lyrics?.synced) return -1;
  const positionMs = state.position * 1000;
  const lines = visibleLines.value;
  let found = -1;
  for (let i = 0; i < lines.length; i += 1) {
    const time = lines[i].timeMs;
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

// 容器高度变化（窗口缩放、切歌后歌词区变高）时重算留白。
watch(
  scroller,
  (box) => {
    resizeObserver?.disconnect();
    if (!box) return;
    syncPad();
    resizeObserver = new ResizeObserver(syncPad);
    resizeObserver.observe(box);
  },
  { flush: "post" },
);

onBeforeUnmount(() => resizeObserver?.disconnect());

watch(
  () => current.value?.id,
  () => {
    lineEls.value = [];
    scroller.value?.scrollTo({ top: 0 });
  },
);

/** 只用封面在左上角散一层光晕，不做整屏模糊——那样会把文字压成灰雾。 */
const glowStyle = computed(() =>
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
    <div class="glow" :style="glowStyle" />

    <header class="top" data-tauri-drag-region>
      <button class="back" @click="player.closeLyrics()">
        <Icon name="back" :size="16" />
        收起
      </button>
      <div class="spacer" data-tauri-drag-region />
      <div class="ctls">
        <button class="ctl" title="最小化" @click="minimize">
          <Icon name="win-min" :size="15" />
        </button>
        <button class="ctl" :title="maximized ? '还原' : '最大化'" @click="toggleMaximize">
          <Icon :name="maximized ? 'win-restore' : 'win-max'" :size="15" />
        </button>
        <button class="ctl close" title="关闭" @click="close">
          <Icon name="close" :size="15" />
        </button>
      </div>
    </header>

    <div v-if="!current" class="empty">还没有播放中的歌曲</div>

    <div v-else class="stage">
      <aside class="meta-col">
        <img v-if="current.coverUrl" class="cover" :src="current.coverUrl" alt="" />
        <span v-else class="cover" />
        <div class="meta-text">
          <h1 class="title">{{ current.title }}</h1>
          <p class="artists">{{ artistText(current) || "未知艺人" }}</p>
          <p v-if="current.album" class="album">{{ current.album.title }}</p>
        </div>
      </aside>

      <div class="lyric-col">
        <div v-if="state.lyricsLoading" class="hint">正在获取歌词…</div>
        <div v-else-if="!state.lyrics" class="hint">这首歌没有可用歌词</div>
        <div
          v-else
          ref="scroller"
          class="lines noscroll"
          :style="{ paddingTop: pad + 'px', paddingBottom: pad + 'px' }"
        >
          <p
            v-for="(line, index) in visibleLines"
            :key="index"
            :ref="(el) => setLineRef(el as Element | null, index)"
            class="line"
            :class="{
              active: index === activeIndex,
              tappable: line.timeMs !== null,
              meta: line.timeMs === null,
            }"
            @click="jump(line.timeMs)"
          >
            <span class="text">{{ line.text }}</span>
            <span v-if="line.translation" class="translation">{{ line.translation }}</span>
          </p>
        </div>
      </div>
    </div>

    <div class="transport-wrap">
      <LyricsControls v-if="current" />
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  flex-direction: column;
  background: var(--bg);
  overflow: hidden;
  animation: overlay-in 0.18s ease;
}

@keyframes overlay-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

/* 封面只在左上角散一层虚光，越靠右下越淡，保证歌词区域的底色是干净的。 */
.glow {
  position: absolute;
  top: -25%;
  left: -8%;
  width: 78%;
  height: 82%;
  background-position: center;
  background-size: cover;
  filter: blur(96px) saturate(1.15);
  opacity: 0.34;
  transform: scale(1.1);
  pointer-events: none;
  mask-image: radial-gradient(72% 68% at 28% 26%, #000 0%, rgba(0, 0, 0, 0.45) 45%, transparent 76%);
  -webkit-mask-image: radial-gradient(
    72% 68% at 28% 26%,
    #000 0%,
    rgba(0, 0, 0, 0.45) 45%,
    transparent 76%
  );
}

.top {
  position: relative;
  z-index: 2;
  display: flex;
  align-items: center;
  height: var(--titlebar-h);
  padding-left: 12px;
  flex: none;
}

.back {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--muted);
  padding: 4px 9px 4px 4px;
  border-radius: var(--r-sm);
}

.back:hover {
  color: var(--text);
  background: var(--surface);
}

.spacer {
  flex: 1;
  align-self: stretch;
}

.ctls {
  display: flex;
  align-items: stretch;
  height: 100%;
  flex: none;
}

.ctl {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  color: var(--muted);
}

.ctl:hover {
  color: var(--text);
  background: var(--line);
}

.ctl.close:hover {
  color: #fff;
  background: #c8443c;
}

.stage {
  position: relative;
  z-index: 1;
  flex: 1;
  min-height: 0;
  display: flex;
  align-items: stretch;
  gap: 52px;
  width: 100%;
  max-width: 1080px;
  margin: 0 auto;
  padding: 0 32px;
}

/* 左栏：大封面 + 曲目信息，垂直居中。 */
.meta-col {
  width: 248px;
  flex: none;
  align-self: center;
  padding-bottom: 10px;
}

.cover {
  display: block;
  width: 248px;
  height: 248px;
  flex: none;
  border-radius: var(--r-lg);
  background: var(--surface);
  object-fit: cover;
  margin-bottom: 18px;
  box-shadow: 0 18px 46px rgba(0, 0, 0, 0.24);
}

.meta-text {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.title {
  font-size: 17px;
  font-weight: 500;
  line-height: 1.45;
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
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.lines {
  /* offsetTop 要相对这一层算，否则滚到当前句会整体偏移。 */
  position: relative;
  flex: 1;
  width: 100%;
  max-width: 640px;
  margin: 0 auto;
  overflow-y: auto;
  scrollbar-width: none;
  /* 上下边缘淡出，滚动时不会看见文字被切断。 */
  mask-image: linear-gradient(to bottom, transparent, #000 12%, #000 88%, transparent);
  -webkit-mask-image: linear-gradient(to bottom, transparent, #000 12%, #000 88%, transparent);
}

.line {
  padding: 10px 0;
  font-size: 18px;
  line-height: 1.6;
  color: var(--muted);
  text-align: center;
  transition: color 0.18s ease;
}

.line.tappable {
  cursor: pointer;
}

.line:hover {
  color: var(--text);
}

.line.active {
  color: var(--text);
  font-weight: 600;
}

/* 词曲编曲这类署名没有时间戳，压小一档，不和歌词抢注意力。 */
.line.meta {
  padding: 3px 0;
  font-size: 13px;
  color: var(--faint);
}

.line.meta.active {
  color: var(--muted);
  font-weight: 400;
}

.text {
  display: block;
}

.translation {
  display: block;
  margin-top: 4px;
  font-size: 13px;
  font-weight: 400;
  color: var(--faint);
}

.line.active .translation {
  color: var(--muted);
}

.transport-wrap {
  position: relative;
  z-index: 1;
  flex: none;
  width: 100%;
  max-width: 1000px;
  margin: 0 auto;
  padding: 0 32px 26px;
}

.hint {
  padding-top: 12vh;
  font-size: 13px;
  color: var(--faint);
}

.empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  color: var(--faint);
}
</style>

<style>
/* WebView 的默认滚动条在浅色主题下是一条很扎眼的白条，歌词区不需要它。 */
.noscroll::-webkit-scrollbar {
  width: 0;
  height: 0;
}
</style>
