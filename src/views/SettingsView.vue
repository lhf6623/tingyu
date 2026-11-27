<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { api, errorText } from "../api";
import { useCache } from "../composables/useCache";
import { usePlayer, type PlayMode } from "../composables/usePlayer";
import { useToast } from "../composables/useToast";

const props = defineProps<{ accountLabel?: string; expireTime?: string }>();
const emit = defineEmits<{ logout: [] }>();

const player = usePlayer();
const { state } = player;
const toast = useToast();
const cache = useCache();

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
  </section>
</template>

<style scoped>
.page {
  padding-bottom: 24px;
  max-width: 640px;
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
  color: var(--accent);
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
  background: var(--text);
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
