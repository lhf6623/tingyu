<script setup lang="ts">
import Icon from "./Icon.vue";
import { artistText, formatDuration } from "../api";
import { usePlayer } from "../composables/usePlayer";

const player = usePlayer();
const { state } = player;
</script>

<template>
  <aside class="pane">
    <header class="head">
      <span class="label">队列<span class="num">{{ state.queue.length }}</span></span>
      <div class="spacer" />
      <button v-if="state.queue.length" class="clear" @click="player.clearQueue()">清空</button>
      <button class="close" title="收起" @click="player.toggleQueue()">
        <Icon name="close" :size="13" />
      </button>
    </header>

    <div v-if="!state.queue.length" class="empty">
      队列是空的。在列表里点一首歌，就会把它所在的整张列表放进来。
    </div>

    <div v-else class="rows">
      <div
        v-for="(track, index) in state.queue"
        :key="`${track.id}-${index}`"
        class="row"
        :class="{ active: index === state.cursor }"
        @click="player.selectAt(index)"
      >
        <span class="idx">{{ String(index + 1).padStart(2, "0") }}</span>
        <span class="meta">
          <span class="title ellipsis">{{ track.title }}</span>
          <span class="sub ellipsis">{{ artistText(track) || "未知艺人" }}</span>
        </span>
        <span class="dur">{{ formatDuration(track.durationSeconds) }}</span>
        <button class="del" title="从队列移除" @click.stop="player.removeAt(index)">
          <Icon name="close" :size="11" />
        </button>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.pane {
  grid-column: 3;
  grid-row: 2;
  display: flex;
  flex-direction: column;
  width: 320px;
  flex: none;
  border-left: 0.5px solid var(--line);
  background: var(--surface);
  min-height: 0;
}

.head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 14px 16px 8px;
  flex: none;
}

.label {
  font-size: 12px;
  font-weight: 500;
  color: var(--muted);
}

.num {
  margin-left: 5px;
  font-size: 11px;
  font-weight: 400;
  opacity: 0.7;
  font-variant-numeric: tabular-nums;
}

.spacer {
  flex: 1;
}

.clear {
  font-size: 11px;
  color: var(--faint);
}

.clear:hover {
  color: var(--accent);
}

.close {
  display: flex;
  color: var(--faint);
}

.close:hover {
  color: var(--text);
}

.rows {
  flex: 1;
  overflow-y: auto;
  padding: 0 12px 24px;
  scrollbar-width: thin;
}

.row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 4px;
  cursor: pointer;
}

.row:hover {
  background: var(--bg);
}

.row.active .title,
.row.active .idx {
  color: var(--accent);
}

.idx {
  width: 18px;
  flex: none;
  font-size: 11px;
  color: var(--faint);
  font-variant-numeric: tabular-nums;
  text-align: center;
}

.meta {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
}

.title {
  font-size: 12px;
}

.sub {
  font-size: 11px;
  color: var(--faint);
}

.dur {
  font-size: 11px;
  color: var(--faint);
  font-variant-numeric: tabular-nums;
}

.del {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border-radius: 3px;
  color: var(--faint);
  opacity: 0;
}

.row:hover .del {
  opacity: 1;
}

.del:hover {
  color: var(--accent);
  background: var(--bg);
}

.empty {
  padding: 32px 20px;
  font-size: 12px;
  line-height: 1.7;
  color: var(--faint);
  text-align: center;
}
</style>
