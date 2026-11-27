import { computed, ref } from "vue";

/**
 * 主题状态（模块级单例）。
 *
 * 只切 data-theme / data-accent 两个属性，样式全部由 styles.css 的 CSS 变量承接，
 * 组件里不碰主题逻辑。选中态装饰色走 --accent，危险操作走 --danger，互不干扰。
 */
export type ThemeMode = "system" | "light" | "dark";
export type AccentName = "red" | "blue" | "green" | "purple" | "amber";

export const THEME_MODES: { value: ThemeMode; label: string }[] = [
  { value: "system", label: "跟随系统" },
  { value: "light", label: "浅色" },
  { value: "dark", label: "深色" },
];

export const ACCENTS: { value: AccentName; label: string; swatch: string }[] = [
  { value: "red", label: "绯红", swatch: "#c8443c" },
  { value: "blue", label: "靛蓝", swatch: "#3c6ea5" },
  { value: "green", label: "松绿", swatch: "#3f7d5a" },
  { value: "purple", label: "藕紫", swatch: "#7355a8" },
  { value: "amber", label: "琥珀", swatch: "#a8732e" },
];

const MODE_KEY = "tingyu.theme.mode";
const ACCENT_KEY = "tingyu.theme.accent";

function readMode(): ThemeMode {
  try {
    const raw = localStorage.getItem(MODE_KEY);
    return raw === "light" || raw === "dark" || raw === "system" ? raw : "system";
  } catch {
    return "system";
  }
}

function readAccent(): AccentName {
  try {
    const raw = localStorage.getItem(ACCENT_KEY);
    return ACCENTS.some((item) => item.value === raw) ? (raw as AccentName) : "red";
  } catch {
    return "red";
  }
}

const mode = ref<ThemeMode>(readMode());
const accent = ref<AccentName>(readAccent());
const systemDark = ref(
  typeof window !== "undefined" && window.matchMedia("(prefers-color-scheme: dark)").matches,
);

/** 实际生效的明暗：跟随系统时由系统偏好推导。 */
const resolved = computed<"light" | "dark">(() =>
  mode.value === "system" ? (systemDark.value ? "dark" : "light") : mode.value,
);

function apply() {
  const root = document.documentElement;
  root.dataset.theme = resolved.value;
  root.dataset.accent = accent.value;
}

function save() {
  try {
    localStorage.setItem(MODE_KEY, mode.value);
    localStorage.setItem(ACCENT_KEY, accent.value);
  } catch {
    // 存不进去不影响当前会话的使用。
  }
}

function setMode(next: ThemeMode) {
  mode.value = next;
  apply();
  save();
}

function setAccent(next: AccentName) {
  accent.value = next;
  apply();
  save();
}

let started = false;

/** 在 main.ts 挂载前调用：立即落一次属性，避免首屏按旧主题闪一下。 */
export function initTheme() {
  apply();
  if (started) return;
  started = true;
  const media = window.matchMedia("(prefers-color-scheme: dark)");
  media.addEventListener("change", (event) => {
    systemDark.value = event.matches;
    // 只有「跟随系统」才需要跟着变；显式选了明暗就保持用户选择。
    if (mode.value === "system") apply();
  });
}

export function useTheme() {
  return { mode, accent, resolved, setMode, setAccent };
}
