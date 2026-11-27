<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(defineProps<{ name: string; size?: number }>(), { size: 16 });

const PATHS: Record<string, string> = {
  play: '<path d="M8 5.6v12.8L19 12z" fill="currentColor" stroke="none"/>',
  pause: '<path d="M9.5 5v14M14.5 5v14" stroke-width="2.4"/>',
  prev: '<path d="M18 6.5v11L9.5 12z" fill="currentColor" stroke="none"/><path d="M6 5v14"/>',
  next: '<path d="M6 6.5v11L14.5 12z" fill="currentColor" stroke="none"/><path d="M18 5v14"/>',
  heart:
    '<path d="M12 20s-7-4.4-7-9.2A4 4 0 0 1 12 8a4 4 0 0 1 7 2.8C19 15.6 12 20 12 20z"/>',
  "heart-filled":
    '<path d="M12 20s-7-4.4-7-9.2A4 4 0 0 1 12 8a4 4 0 0 1 7 2.8C19 15.6 12 20 12 20z" fill="currentColor"/>',
  search: '<circle cx="11" cy="11" r="6"/><path d="M16 16l4 4"/>',
  list: '<path d="M4 7h12M4 12h12M4 17h8"/>',
  "list-plus": '<path d="M4 7h12M4 12h12M4 17h8"/><path d="M18 13v6M15 16h6"/>',
  note: '<circle cx="7" cy="18" r="2.6"/><path d="M9.6 18V6.4L19 4.6v10.8"/><circle cx="16.4" cy="15.4" r="2.6"/>',
  back: '<path d="M15 5l-7 7 7 7"/>',
  repeat:
    '<path d="M4 11V9.5A3.5 3.5 0 0 1 7.5 6H18"/><path d="M15.5 3.5L18 6l-2.5 2.5"/><path d="M20 13v1.5A3.5 3.5 0 0 1 16.5 18H6"/><path d="M8.5 20.5L6 18l2.5-2.5"/>',
  "repeat-one":
    '<path d="M4 11V9.5A3.5 3.5 0 0 1 7.5 6H18"/><path d="M15.5 3.5L18 6l-2.5 2.5"/><path d="M20 13v1.5A3.5 3.5 0 0 1 16.5 18H6"/><path d="M8.5 20.5L6 18l2.5-2.5"/><path d="M11.6 10.6l1.1-.9v4.6"/>',
  shuffle:
    '<path d="M4 6.5h3.2l8.6 11h3.2"/><path d="M4 17.5h3.2l2.4-3"/><path d="M16.5 3.5L19.5 6l-3 2.5"/><path d="M16.5 14.5l3 2.5-3 2.5"/>',
  order: '<path d="M4 7h10M4 12h10M4 17h10"/><path d="M18 5v13"/><path d="M15.4 15.4L18 18l2.6-2.6"/>',
  volume: '<path d="M5 10v4h3l4 3.2V6.8L8 10H5z"/><path d="M15.6 9.4a3.6 3.6 0 0 1 0 5.2"/>',
  mute: '<path d="M5 10v4h3l4 3.2V6.8L8 10H5z"/><path d="M15.5 10.2l4 4M19.5 10.2l-4 4"/>',
  close: '<path d="M6.5 6.5l11 11M17.5 6.5l-11 11"/>',
  "win-min": '<path d="M6 12h12"/>',
  "win-max": '<rect x="6.5" y="6.5" width="11" height="11" rx="1.2"/>',
  "win-restore":
    '<rect x="8.5" y="8.5" width="9" height="9" rx="1.2"/><path d="M6.5 15V7.7a1.2 1.2 0 0 1 1.2-1.2H15"/>',
  lyrics: '<path d="M4 6.5h16M4 12h11M4 17.5h13"/>',
  folder: '<path d="M4 7.5A1.5 1.5 0 0 1 5.5 6h3.2l2 2.2h7.8A1.5 1.5 0 0 1 20 9.7v7.8A1.5 1.5 0 0 1 18.5 19h-13A1.5 1.5 0 0 1 4 17.5z"/>',
  clock: '<circle cx="12" cy="12" r="8"/><path d="M12 7.5V12l3 1.8"/>',
  gear: '<circle cx="12" cy="12" r="3"/><path d="M12 3.5v2M12 18.5v2M4.9 7.5l1.7 1M17.4 15.5l1.7 1M4.9 16.5l1.7-1M17.4 8.5l1.7-1"/>',
  spark: '<path d="M12 4l1.8 5.2L19 11l-5.2 1.8L12 18l-1.8-5.2L5 11l5.2-1.8z"/>',
};

const body = computed(() => PATHS[props.name] ?? "");
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.6"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
    v-html="body"
  />
</template>
