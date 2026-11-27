<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import Icon from "./Icon.vue";
import { artistText, type Track } from "../api";
import { useNav } from "../composables/useNav";
import { useSearch } from "../composables/useSearch";
import { useWindowControls } from "../composables/useWindowControls";

const nav = useNav();
const search = useSearch();
const { keyword, suggestions, suggesting, history } = search;

const open = ref(false);
const active = ref(-1);
const searchBox = ref<HTMLElement | null>(null);

const { maximized, minimize, toggleMaximize, close } = useWindowControls();
let debounce: number | undefined;

const typed = computed(() => keyword.value.trim().length > 0);

function openDropdown() {
  open.value = true;
  active.value = -1;
}

function closeDropdown() {
  open.value = false;
  active.value = -1;
}

function onInput() {
  openDropdown();
  window.clearTimeout(debounce);
  debounce = window.setTimeout(() => void search.suggest(keyword.value), 350);
}

/**
 * focus 和 click 都走这里：输入框已经聚焦时再点不会触发 focus，
 * 而 Esc 收起、点拖拽区这些情况下输入框可能一直保持聚焦，只靠 focus 会打不开。
 */
function onFocus() {
  if (open.value) return;
  openDropdown();
  if (typed.value) void search.suggest(keyword.value);
}

function goResults(query: string) {
  const text = query.trim();
  if (!text) return;
  closeDropdown();
  void search.runSearch(text, true);
  nav.goRoot({ name: "search" });
}

function submit() {
  goResults(keyword.value);
}

/** 点放大镜进搜索页空白态，清掉输入免得带一个没有结果的词过去。 */
function openSearchPage() {
  closeDropdown();
  search.reset();
  nav.goRoot({ name: "search" });
}

function suggestionLabel(track: Track) {
  return `${track.title} ${artistText(track)}`.trim();
}

function onKeydown(event: KeyboardEvent) {
  const rows = typed.value ? suggestions.value.length + 1 : history.value.length;
  if (event.key === "Escape") {
    closeDropdown();
    return;
  }
  if (!open.value || rows === 0) return;
  if (event.key === "ArrowDown") {
    event.preventDefault();
    active.value = (active.value + 1) % rows;
  } else if (event.key === "ArrowUp") {
    event.preventDefault();
    active.value = active.value <= 0 ? rows - 1 : active.value - 1;
  } else if (event.key === "Enter") {
    event.preventDefault();
    if (active.value < 0) {
      submit();
      return;
    }
    if (typed.value) {
      const track = suggestions.value[active.value];
      goResults(track ? suggestionLabel(track) : keyword.value);
    } else {
      goResults(history.value[active.value] ?? keyword.value);
    }
  }
}

/**
 * 用 pointerdown 捕获阶段判断：比 click 更早，拖拽标题栏、点窗口按钮都能立刻收起。
 * 只把搜索框（含下拉本身）当内部，标题栏其余区域一律算外部。
 */
function onDocumentPointerDown(event: PointerEvent) {
  if (!open.value) return;
  if (!searchBox.value?.contains(event.target as Node)) closeDropdown();
}

/** 点到其它应用或拖窗时浏览器窗口会失焦，这时也要收起。 */
function onWindowBlur() {
  closeDropdown();
}

onMounted(() => {
  window.addEventListener("pointerdown", onDocumentPointerDown, true);
  window.addEventListener("blur", onWindowBlur);
});

onBeforeUnmount(() => {
  window.clearTimeout(debounce);
  window.removeEventListener("pointerdown", onDocumentPointerDown, true);
  window.removeEventListener("blur", onWindowBlur);
});
</script>

<template>
  <header class="titlebar" data-tauri-drag-region>
    <div class="drag" data-tauri-drag-region />

    <div ref="searchBox" class="search">
      <button class="search-btn" title="打开搜索页" @click="openSearchPage">
        <Icon name="search" :size="13" />
      </button>
      <input
        v-model="keyword"
        type="text"
        placeholder="搜索歌曲、艺人"
        spellcheck="false"
        @focus="onFocus"
        @click="onFocus"
        @input="onInput"
        @keydown="onKeydown"
      />

      <div v-if="open" class="dropdown">
        <template v-if="!typed">
          <template v-if="history.length">
            <div class="dd-head">
              <span>最近搜索</span>
              <button class="dd-clear" @click="search.clearHistory()">清空</button>
            </div>
            <button
              v-for="(item, index) in history"
              :key="item"
              class="dd-row"
              :class="{ active: active === index }"
              @mousedown.prevent="goResults(item)"
            >
              <span class="dd-title ellipsis">{{ item }}</span>
            </button>
          </template>
          <div v-else class="dd-empty">输入关键词开始搜索</div>
        </template>

        <template v-else>
          <div v-if="suggesting" class="dd-empty">搜索中…</div>
          <button
            v-for="(track, index) in suggestions"
            :key="track.id"
            class="dd-row"
            :class="{ active: active === index }"
            @mousedown.prevent="goResults(suggestionLabel(track))"
          >
            <span class="dd-text">
              <span class="dd-title ellipsis">{{ track.title }}</span>
              <span class="dd-sub ellipsis">{{ artistText(track) }}</span>
            </span>
          </button>
          <button
            class="dd-row dd-search"
            :class="{ active: active === suggestions.length }"
            @mousedown.prevent="submit()"
          >
            搜索 “{{ keyword }}”
          </button>
        </template>
      </div>
    </div>

    <div class="controls">
      <button class="ctl" title="最小化" @click="minimize">
        <Icon name="win-min" :size="14" />
      </button>
      <button class="ctl" :title="maximized ? '还原' : '最大化'" @click="toggleMaximize">
        <Icon :name="maximized ? 'win-restore' : 'win-max'" :size="14" />
      </button>
      <button class="ctl close" title="关闭" @click="close">
        <Icon name="close" :size="14" />
      </button>
    </div>
  </header>
</template>

<style scoped>
.titlebar {
  grid-column: 2 / -1;
  grid-row: 1;
  display: flex;
  align-items: center;
  gap: 10px;
  height: var(--titlebar-h);
  flex: none;
  padding-left: 16px;
  background: var(--surface);
  border-bottom: 0.5px solid var(--line);
  user-select: none;
  position: relative;
  z-index: 40;
}

.drag {
  flex: 1;
  height: 100%;
}

.search {
  position: relative;
  display: flex;
  align-items: center;
  gap: 6px;
  height: 28px;
  width: 280px;
  padding: 0 11px;
  border: 0.5px solid var(--line-strong);
  border-radius: 6px;
  color: var(--faint);
  background: var(--bg);
}

.search:focus-within {
  border-color: var(--muted);
}

.search-btn {
  display: flex;
  align-items: center;
  color: var(--faint);
}

.search-btn:hover {
  color: var(--text);
}

.search input {
  flex: 1;
  min-width: 0;
  font-size: 12.5px;
  color: var(--text);
}

.dropdown {
  position: absolute;
  top: calc(100% + 6px);
  right: -8px;
  width: 340px;
  max-height: 320px;
  overflow-y: auto;
  padding: 6px;
  background: var(--bg);
  border: 0.5px solid var(--line-strong);
  border-radius: 8px;
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.08);
  scrollbar-width: thin;
}

.dd-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px 6px;
  font-size: 11px;
  color: var(--faint);
}

.dd-clear:hover {
  color: var(--accent);
}

.dd-row {
  display: flex;
  align-items: center;
  width: 100%;
  padding: 7px 8px;
  border-radius: 5px;
  text-align: left;
  color: var(--text);
}

.dd-row:hover,
.dd-row.active {
  background: var(--surface);
}

.dd-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.dd-title {
  font-size: 12.5px;
}

.dd-sub {
  font-size: 11px;
  color: var(--faint);
}

.dd-search {
  font-size: 12px;
  color: var(--muted);
}

.dd-empty {
  padding: 10px 8px;
  font-size: 12px;
  color: var(--faint);
}

.controls {
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
</style>
