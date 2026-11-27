<script setup lang="ts">
import { computed, ref } from "vue";
import Icon from "./Icon.vue";
import { artistText, maskDuration } from "../api";
import { useNav } from "../composables/useNav";
import { usePlayer, type PlayMode } from "../composables/usePlayer";

const player = usePlayer();
const { state, current, progress } = player;
const nav = useNav();

const onLyricsPage = computed(() => nav.route.value.name === "lyrics");

function openLyrics() {
  if (!current.value || onLyricsPage.value) return;
  nav.go({ name: "lyrics" });
}

const bar = ref<HTMLDivElement | null>(null);
const dragging = ref(false);

const modeIcon = computed(() => {
  const map: Record<PlayMode, string> = {
    "repeat-all": "repeat",
    order: "order",
    "repeat-one": "repeat-one",
    shuffle: "shuffle",
  };
  return map[state.mode];
});

const modeLabel = computed(() => {
  const map: Record<PlayMode, string> = {
    "repeat-all": "列表循环",
    order: "顺序播放",
    "repeat-one": "单曲循环",
    shuffle: "随机播放",
  };
  return map[state.mode];
});

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
    <button class="now" :disabled="!current" title="打开歌词页" @click="openLyrics">
      <img v-if="current?.coverUrl" class="cover" :src="current.coverUrl" alt="" />
      <span v-else class="cover" />
      <span class="meta">
        <span class="title ellipsis">{{ current?.title ?? "未在播放" }}</span>
        <span class="sub ellipsis">{{ current ? artistText(current) : "" }}</span>
      </span>
    </button>

    <div class="controls">
      <button class="icon-btn" title="上一首" :disabled="!current" @click="player.previous()">
        <Icon name="prev" :size="16" />
      </button>
      <button class="play" :disabled="!current" title="播放 / 暂停" @click="player.toggle()">
        <Icon :name="state.playing ? 'pause' : 'play'" :size="15" />
      </button>
      <button class="icon-btn" title="下一首" :disabled="!current" @click="player.next()">
        <Icon name="next" :size="16" />
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
        <Icon :name="modeIcon" :size="15" />
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
        :class="{ on: onLyricsPage }"
        title="歌词"
        @click="openLyrics"
      >
        <Icon name="lyrics" :size="15" />
      </button>
      <button
        class="icon-btn"
        :class="{ on: state.queueOpen }"
        title="播放队列"
        @click="player.toggleQueue()"
      >
        <Icon name="list" :size="15" />
      </button>
      <div class="volume">
        <button class="icon-btn" title="静音" @click="player.setVolume(state.volume > 0 ? 0 : 0.8)">
          <Icon :name="state.volume > 0 ? 'volume' : 'mute'" :size="15" />
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
  grid-column: 2 / -1;
  grid-row: 3;
  display: flex;
  align-items: center;
  gap: 18px;
  height: var(--player-h);
  padding: 0 18px;
  border-top: 0.5px solid var(--line);
  background: var(--bg);
}

.now {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 200px;
  flex: none;
  min-width: 0;
  text-align: left;
}

.now:disabled {
  cursor: default;
}

.cover {
  width: 38px;
  height: 38px;
  flex: none;
  border-radius: 4px;
  background: var(--surface);
  object-fit: cover;
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
  width: 26px;
  height: 26px;
  border-radius: 5px;
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
  width: 30px;
  height: 30px;
  border-radius: 50%;
  background: var(--text);
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
  border-radius: 2px;
  background: var(--line-strong);
}

.fill {
  position: absolute;
  left: 0;
  height: 3px;
  border-radius: 2px;
  background: var(--text);
}

.knob {
  position: absolute;
  width: 8px;
  height: 8px;
  margin-left: -4px;
  border-radius: 50%;
  background: var(--text);
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
  border: 0.5px solid var(--line-strong);
  border-radius: 4px;
  padding: 2px 6px;
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
  border-radius: 2px;
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
