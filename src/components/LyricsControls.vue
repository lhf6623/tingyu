<script setup lang="ts">
import { computed, ref } from "vue";
import Icon from "./Icon.vue";
import { maskDuration } from "../api";
import { PLAY_MODE_META, usePlayer } from "../composables/usePlayer";

/**
 * 歌词页自己的播放控制。
 *
 * 不复用底部播放栏：那一套是给「一边浏览一边听」设计的，
 * 信息密度高、控件小；歌词页整屏只有一首歌，控件要少、要大、要居中，
 * 所以这里单独写一份，只保留进度、上一首/播放/下一首、模式和音量。
 */
const player = usePlayer();
const { state, current, progress } = player;

const seekBar = ref<HTMLDivElement | null>(null);
const dragging = ref(false);

const modeIcon = computed(() => PLAY_MODE_META[state.mode].icon);
const modeLabel = computed(() => PLAY_MODE_META[state.mode].label);

function ratioFromEvent(event: PointerEvent): number {
  const element = seekBar.value;
  if (!element) return 0;
  const rect = element.getBoundingClientRect();
  return Math.min(Math.max((event.clientX - rect.left) / rect.width, 0), 1);
}

function onPointerDown(event: PointerEvent) {
  if (!current.value) return;
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
  <div class="transport">
    <div class="seek-row">
      <span class="time">{{ maskDuration(state.position * 1000) }}</span>
      <div
        ref="seekBar"
        class="seek"
        :class="{ dragging }"
        @pointerdown="onPointerDown"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerUp"
      >
        <div class="rail" />
        <div class="fill" :style="{ width: progress * 100 + '%' }" />
        <div class="knob" :style="{ left: progress * 100 + '%' }" />
      </div>
      <span class="time">{{ maskDuration(state.duration * 1000) }}</span>
    </div>

    <div class="keys">
      <div class="side">
        <button class="ghost" :title="modeLabel" @click="player.cycleMode()">
          <Icon :name="modeIcon" :size="15" />
        </button>
      </div>

      <div class="center">
        <button
          class="step"
          title="上一首"
          :disabled="!current"
          @click="player.previous()"
        >
          <Icon name="prev" :size="18" />
        </button>
        <button class="play" :disabled="!current" title="播放 / 暂停" @click="player.toggle()">
          <Icon :name="state.playing ? 'pause' : 'play'" :size="20" />
        </button>
        <button class="step" title="下一首" :disabled="!current" @click="player.next()">
          <Icon name="next" :size="18" />
        </button>
      </div>

      <div class="side right">
        <Icon name="volume" :size="14" />
        <input
          class="volume"
          type="range"
          min="0"
          max="100"
          :value="Math.round(state.volume * 100)"
          aria-label="音量"
          @input="onVolume"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.transport {
  flex: none;
  padding: 0 4px 22px;
}

.seek-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.time {
  flex: none;
  width: 38px;
  font-size: 11px;
  color: var(--faint);
  font-variant-numeric: tabular-nums;
  text-align: center;
}

.seek {
  position: relative;
  flex: 1;
  height: 18px;
  display: flex;
  align-items: center;
  cursor: pointer;
  touch-action: none;
}

.rail,
.fill {
  position: absolute;
  left: 0;
  height: 3px;
  border-radius: 2px;
}

.rail {
  right: 0;
  background: var(--line-strong);
}

.fill {
  background: var(--text);
}

.knob {
  position: absolute;
  width: 9px;
  height: 9px;
  margin-left: -4.5px;
  border-radius: 50%;
  background: var(--text);
  opacity: 0;
  transition: opacity 0.12s ease;
}

.seek:hover .knob,
.seek.dragging .knob {
  opacity: 1;
}

.keys {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  margin-top: 4px;
}

.center {
  display: flex;
  align-items: center;
  gap: 26px;
}

.side {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--muted);
}

.side.right {
  justify-content: flex-end;
}

.step {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border-radius: 50%;
  color: var(--muted);
}

.step:hover:not(:disabled) {
  color: var(--text);
  background: var(--surface);
}

.play {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 46px;
  border-radius: 50%;
  color: var(--bg);
  background: var(--text);
  padding-left: 2px;
}

.play:disabled {
  opacity: 0.4;
}

.ghost {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 5px;
  color: var(--muted);
}

.ghost:hover {
  color: var(--text);
  background: var(--surface);
}

.volume {
  width: 78px;
  height: 16px;
  accent-color: var(--text);
  cursor: pointer;
}
</style>
