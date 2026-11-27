<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { api, errorText } from "../api";
import { useCache } from "../composables/useCache";
import { usePlayer, type PlayMode } from "../composables/usePlayer";
import { ACCENTS, THEME_MODES, useTheme } from "../composables/useTheme";
import { useToast } from "../composables/useToast";

const props = defineProps<{ accountLabel?: string; expireTime?: string }>();
const emit = defineEmits<{ logout: [] }>();

const player = usePlayer();
const { state } = player;
const toast = useToast();
const cache = useCache();
// 解构后 mode / accent 是顶层 ref，模板里会自动解包成值。
const { mode: themeMode, accent: themeAccent, setMode, setAccent } = useTheme();

const cacheBytes = ref(0);

const modeText = computed(() => {
  const map: Record<PlayMode, string> = {
    "repeat-all": "列表循环",
    order: "顺序播放",
    "repeat-one": "单曲循环",
    shuffle: "随机播放",
  };
  return map[state.mode];
});

const cacheText = computed(() => {
  const bytes = cacheBytes.value;
  if (bytes <= 0) return "空";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
});

const expireText = computed(() => {
  const raw = props.expireTime;
  if (!raw || raw.length < 8) return "未知";
  return `${raw.slice(0, 4)}-${raw.slice(4, 6)}-${raw.slice(6, 8)}`;
});

async function refreshCache() {
  try {
    cacheBytes.value = await api.lyricsCacheSize();
  } catch {
    cacheBytes.value = 0;
  }
}

/** 清掉页面级缓存，收藏 / 歌单 / 最近播放在下次进入时重新请求。 */
function refreshData() {
  cache.invalidate();
  toast.info("已清空页面缓存，切换页面会重新加载");
}

async function clearCache() {
  try {
    const freed = await api.clearLyricsCache();
    toast.info(freed > 0 ? "歌词缓存已清除" : "缓存本来就是空的");
    await refreshCache();
  } catch (error) {
    toast.error(errorText(error));
  }
}

onMounted(() => void refreshCache());
</script>

<template>
  <section class="page">
    <header class="head"><h2>设置</h2></header>

    <div class="cols">
      <div class="col">
        <div class="group">
          <h3>外观</h3>
          <div class="row">
            <span class="label">主题</span>
            <div class="segmented">
              <button
                v-for="item in THEME_MODES"
                :key="item.value"
                :class="{ on: themeMode === item.value }"
                @click="setMode(item.value)"
              >
                {{ item.label }}
              </button>
            </div>
          </div>
          <div class="row">
            <span class="label">强调色</span>
            <div class="swatches">
              <button
                v-for="item in ACCENTS"
                :key="item.value"
                class="swatch"
                :class="{ on: themeAccent === item.value }"
                :style="{ background: item.swatch }"
                :title="item.label"
                :aria-label="item.label"
                :aria-pressed="themeAccent === item.value"
                @click="setAccent(item.value)"
              />
            </div>
          </div>
          <p class="note">选择「跟随系统」会随 macOS / Windows 的深浅模式自动切换；强调色只影响选中与标记，删除、清空等危险操作始终为红色。</p>
        </div>

        <div class="group">
          <h3>播放</h3>
          <div class="row">
            <span class="label">音质</span>
            <div class="segmented">
              <button :class="{ on: state.quality === 'standard' }" @click="player.setQuality('standard')">
                标准
              </button>
              <button :class="{ on: state.quality === 'lossless' }" @click="player.setQuality('lossless')">
                无损
              </button>
            </div>
          </div>
          <div class="row">
            <span class="label">播放模式</span>
            <button class="value" @click="player.cycleMode()">{{ modeText }} ›</button>
          </div>
          <p class="note">无损需要账号有对应权限；实际返回的音质会在播放时以服务端为准。</p>
        </div>

        <div class="group">
          <h3>窗口</h3>
          <div class="row">
            <span class="label">关闭窗口时最小化到托盘</span>
            <button
              class="switch"
              :class="{ on: state.closeToTray }"
              :aria-pressed="state.closeToTray"
              @click="player.setCloseToTray(!state.closeToTray)"
            >
              <span class="knob" />
            </button>
          </div>
          <p class="note">
            关闭后应用继续在托盘运行，右键托盘图标可以退出；托盘菜单也能控制上一首 / 播放暂停 / 下一首。
          </p>
        </div>
          </div>

          <div class="col">
        <div class="group">
          <h3>缓存</h3>
          <div class="row">
            <span class="label">刷新收藏与歌单</span>
            <button class="action" @click="refreshData">刷新</button>
          </div>
          <div class="row">
            <span class="label">歌词缓存</span>
            <span class="value static">{{ cacheText }}</span>
          </div>
          <div class="row">
            <span class="label">清除歌词缓存</span>
            <button class="danger" @click="clearCache">清除</button>
          </div>
          <p class="note">
            刷新会清空页面缓存，切换页面时重新拉取收藏、歌单和最近播放；歌词文本另按 90 天缓存，只存歌词和曲目标识，不含账号信息。
          </p>
        </div>

        <div class="group">
          <h3>账号</h3>
          <div class="row">
            <span class="label">当前账号</span>
            <span class="value static">{{ props.accountLabel ?? "未登录" }}</span>
          </div>
          <div class="row">
            <span class="label">音乐会话到期</span>
            <span class="value static">{{ expireText }}</span>
          </div>
          <div class="row">
            <span class="label">退出登录</span>
            <button class="danger" @click="emit('logout')">退出</button>
          </div>
          <p class="note">会话失效后需要重新扫码登录，本地只保存音乐会话，不保存账号令牌。</p>
        </div>

        <div class="group">
          <h3>关于</h3>
          <div class="row">
            <span class="label">版本</span>
            <span class="value static">0.1.0</span>
          </div>
          <p class="note">
            听屿 Tingyu · Tauri v2 + Vue 3 实现的第三方桌面客户端。非官方项目，与任何音乐服务提供方无隶属或合作关系，仅供个人学习自用。播放地址不落盘、不写日志、无遥测。
          </p>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.page {
  padding-bottom: 24px;
}

/* 两列铺满内容区。用 container query 按内容区自身宽度判断，比 viewport 媒体查询
 * 准——视口宽度还要先扣掉侧栏 202px，用媒体查询会把默认 1100 宽的窗口误判成单列。 */
.page {
  container-type: inline-size;
}

.cols {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0 14px;
  align-items: start;
}

/* 内容区窄到放不下两列（每列至少 340px）时退回单列 */
@container (max-width: 694px) {
  .cols {
    grid-template-columns: minmax(0, 1fr);
  }
}

.col {
  min-width: 0;
}

.head {
  margin-bottom: 18px;
}

.head h2 {
  font-size: 18px;
  font-weight: 500;
}

/* 每个分组是白色面板里一块浅灰圆角卡，行之间靠留白分开，全站不再有横线 */
.group {
  margin-bottom: 14px;
  padding: 14px 18px 16px;
  background: var(--surface);
  border-radius: var(--r-lg);
}

.group h3 {
  font-size: 12px;
  font-weight: 500;
  color: var(--muted);
  margin-bottom: 6px;
}

.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 9px 0;
}

.label {
  font-size: 13px;
}

.value {
  font-size: 12px;
  color: var(--muted);
}

.value:hover:not(.static) {
  color: var(--text);
}

.value.static {
  color: var(--faint);
}

.segmented {
  display: flex;
  gap: 2px;
  padding: 3px;
  background: var(--base);
  border-radius: var(--r-md);
}

.segmented button {
  font-size: 12px;
  color: var(--faint);
  padding: 3px 12px;
  border-radius: var(--r-sm);
}

.segmented button.on {
  color: var(--text);
  background: var(--bg);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.09);
}

.swatches {
  display: flex;
  align-items: center;
  gap: 12px;
  padding-right: 4px;
}

.swatch {
  position: relative;
  width: 17px;
  height: 17px;
  flex: none;
  border-radius: 50%;
  transition: transform 0.14s ease;
}

.swatch:hover {
  transform: scale(1.12);
}

/* 选中环用伪元素 border 画圆：整数像素、跟随圆角，不会像非整数 box-shadow
 * 那样在缩放屏上被像素对齐成一厚一薄。inset 负值留出底色间隙。 */
.swatch.on::after {
  content: "";
  position: absolute;
  inset: -4px;
  border-radius: 50%;
  border: 2px solid var(--muted);
}

.action,
.danger {
  font-size: 12px;
  color: var(--muted);
  background: var(--bg);
  border-radius: var(--r-sm);
  padding: 4px 13px;
}

.action:hover {
  color: var(--text);
}

.danger {
  color: var(--danger);
}

.switch {
  position: relative;
  width: 34px;
  height: 19px;
  border-radius: var(--r-lg);
  background: var(--line-strong);
  transition: background 0.15s ease;
  flex: none;
}

.switch.on {
  background: var(--accent);
}

.knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 15px;
  height: 15px;
  border-radius: 50%;
  background: var(--bg);
  transition: transform 0.15s ease;
}

.switch.on .knob {
  transform: translateX(15px);
}

.note {
  margin-top: 8px;
  font-size: 11px;
  line-height: 1.7;
  color: var(--faint);
}
</style>
