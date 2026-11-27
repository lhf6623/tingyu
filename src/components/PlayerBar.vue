<script setup lang="ts">
import { computed, ref } from "vue";
import Icon from "./Icon.vue";
import { artistText, maskDuration } from "../api";
import { PLAY_MODE_META, usePlayer } from "../composables/usePlayer";

const player = usePlayer();
const { state, current, progress } = player;

function toggleLyrics() {
  if (!current.value) return;
  player.toggleLyrics();
}

const bar = ref<HTMLDivElement | null>(null);
const dragging = ref(false);


const modeIcon = computed(() => PLAY_MODE_META[state.mode].icon);
const modeLabel = computed(() => PLAY_MODE_META[state.mode].label);

function ratioFromEvent(event: PointerEvent): number {
  const element = bar.value;
  if (!element) return 0;
  const rect = element.getBoundingClientRect();
  return Math.min(Math.max((event.clientX - rect.left) / rect.width, 0), 1);
}

function onPointerDown(event: PointerEvent) {
  dragging.value = true;
  (event.target as HTMLElement).setPointerCapture?.(event.pointerId);
  player.seek(ratioFromEvent(event));
}

function onPointerMove(event: PointerEvent) {
  if (!dragging.value) return;
  player.seek(ratioFromEvent(event));
}

function onPointerUp(event: PointerEvent) {
  if (!dragging.value) return;
  dragging.value = false;
  player.seek(ratioFromEvent(event));
}

function onVolume(event: Event) {
  player.setVolume(Number((event.target as HTMLInputElement).value) / 100);
}
</script>

<template>
  <footer class="bar">
    <div class="now">
      <button
        class="cover-btn"
        :disabled="!current"
        :title="state.lyricsOpen ? '收起歌词' : '展开歌词'"
        aria-label="展开歌词"
        @click="toggleLyrics"
      >
        <img v-if="current?.coverUrl" class="cover" :src="current.coverUrl" alt="" />
        <span v-else class="cover" />
        <span class="veil">
          <Icon name="lyrics" :size="16" />
        </span>
      </button>
      <span class="meta">
        <span class="title ellipsis">{{ current?.title ?? "未在播放" }}</span>
        <span class="sub ellipsis">{{ current ? artistText(current) : "" }}</span>
      </span>
    </div>

    <div class="controls">
      <button class="icon-btn" title="上一首" :disabled="!current" @click="player.previous()">
        <Icon name="prev" :size="18" />
      </button>
      <button class="play" :disabled="!current" title="播放 / 暂停" @click="player.toggle()">
        <Icon :name="state.playing ? 'pause' : 'play'" :size="17" />
      </button>
      <button class="icon-btn" title="下一首" :disabled="!current" @click="player.next()">
        <Icon name="next" :size="18" />
      </button>
    </div>

    <div class="timeline">
      <span class="time">{{ maskDuration(state.position * 1000) }}</span>
      <div
        ref="bar"
        class="track"
        @pointerdown="onPointerDown"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerUp"
      >
        <div class="fill" :style="{ width: progress * 100 + '%' }" />
        <div class="knob" :style="{ left: progress * 100 + '%' }" />
      </div>
      <span class="time">{{ maskDuration(state.duration * 1000) }}</span>
    </div>

    <div class="right">
      <button class="icon-btn" :title="modeLabel" @click="player.cycleMode()">
        <Icon :name="modeIcon" :size="16" />
      </button>
      <button
        class="pill"
        :title="state.quality === 'lossless' ? '无损' : '标准'"
        @click="player.setQuality(state.quality === 'lossless' ? 'standard' : 'lossless')"
      >
        {{ state.quality === "lossless" ? "无损" : "标准" }}
      </button>
      <button
        class="icon-btn"
        :class="{ on: state.queueOpen }"
        title="播放队列"
        @click="player.toggleQueue()"
      >
        <Icon name="list" :size="16" />
      </button>
      <div class="volume">
        <button class="icon-btn" title="静音" @click="player.setVolume(state.volume > 0 ? 0 : 0.8)">
          <Icon :name="state.volume > 0 ? 'volume' : 'mute'" :size="16" />
        </button>
        <input
          class="slider"
          type="range"
          min="0"
          max="100"
          :value="Math.round(state.volume * 100)"
          aria-label="音量"
          @input="onVolume"
        />
      </div>
    </div>
  </footer>
</template>

<style scoped>
.bar {
  display: flex;
  align-items: center;
  gap: 18px;
  width: 100%;
  height: var(--player-h);
  flex: none;
  padding: 0 18px;
  /* 播放栏是第二层面板的底边，靠圆角收口，不再画上边线 */
  background: transparent;
}

.now {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 200px;
  flex: none;
  min-width: 0;
}

/* 封面是唯一进入歌词页的入口，hover 时浮一层提示，明示可点。 */
.cover-btn {
  position: relative;
  width: 38px;
  height: 38px;
  flex: none;
  border-radius: var(--r-sm);
  overflow: hidden;
}

.cover-btn:disabled {
  cursor: default;
}

.cover {
  display: block;
  width: 100%;
  height: 100%;
  border-radius: var(--r-sm);
  background: var(--surface);
  object-fit: cover;
  transition: transform 0.18s ease;
}

.cover-btn:hover:not(:disabled) .cover {
  transform: scale(1.09);
}

.veil {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.45);
  color: #fff;
  opacity: 0;
  transition: opacity 0.15s ease;
}

.cover-btn:hover:not(:disabled) .veil,
.cover-btn:focus-visible .veil {
  opacity: 1;
}

.meta {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.title {
  font-size: 12px;
}

.sub {
  font-size: 11px;
  color: var(--faint);
}

.controls {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: none;
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: var(--r-sm);
  color: var(--muted);
}

.icon-btn:hover {
  color: var(--text);
  background: var(--surface);
}

.icon-btn.on {
  color: var(--text);
}

.icon-btn:disabled {
  opacity: 0.3;
  background: none;
}

.play {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--accent);
  color: var(--bg);
  flex: none;
}

.play:disabled {
  opacity: 0.3;
}

.timeline {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  min-width: 120px;
}

.time {
  font-size: 11px;
  color: var(--faint);
  font-variant-numeric: tabular-nums;
  flex: none;
  width: 34px;
  text-align: center;
}

.track {
  position: relative;
  flex: 1;
  height: 16px;
  display: flex;
  align-items: center;
  cursor: pointer;
}

.track::before {
  content: "";
  position: absolute;
  left: 0;
  right: 0;
  height: 3px;
  border-radius: var(--r-pill);
  background: var(--line-strong);
}

.fill {
  position: absolute;
  left: 0;
  height: 3px;
  border-radius: var(--r-pill);
  background: var(--accent);
}

.knob {
  position: absolute;
  width: 8px;
  height: 8px;
  margin-left: -4px;
  border-radius: 50%;
  background: var(--accent);
  opacity: 0;
}

.track:hover .knob {
  opacity: 1;
}

.right {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: none;
}

.pill {
  font-size: 11px;
  color: var(--muted);
  background: var(--surface);
  border-radius: var(--r-sm);
  padding: 2px 8px;
  line-height: 1.4;
}

.pill:hover {
  color: var(--text);
}

.volume {
  display: flex;
  align-items: center;
  gap: 4px;
}

.slider {
  width: 64px;
  height: 3px;
  appearance: none;
  background: var(--line-strong);
  border-radius: var(--r-pill);
  cursor: pointer;
}

.slider::-webkit-slider-thumb {
  appearance: none;
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: var(--text);
}
</style>
