<script setup lang="ts">
import Icon from "./Icon.vue";
import { useWindowControls } from "../composables/useWindowControls";

/** 全屏登录门：未登录时它是唯一渲染的内容，主界面所有功能都不挂载。 */
defineProps<{
  booting: boolean;
  loggingIn: boolean;
  error: string;
}>();

const emit = defineEmits<{ login: [] }>();

// 无边框窗口没有原生标题栏，登录页必须自带窗口控制，否则关不掉也拖不动。
const { maximized, minimize, toggleMaximize, close } = useWindowControls();
</script>

<template>
  <div class="login" data-tauri-drag-region>
    <div class="winbar" data-tauri-drag-region>
      <div class="drag" data-tauri-drag-region />
      <button class="win" title="最小化" @click="minimize">
        <Icon name="win-min" :size="15" />
      </button>
      <button class="win" :title="maximized ? '还原' : '最大化'" @click="toggleMaximize">
        <Icon :name="maximized ? 'win-restore' : 'win-max'" :size="15" />
      </button>
      <button class="win close" title="关闭" @click="close">
        <Icon name="close" :size="15" />
      </button>
    </div>

    <div class="card">
      <!-- 唱片 + 海浪：音乐小岛的品牌标记，手写 SVG，不引入素材 -->
      <div class="logo">
        <svg width="60" height="60" viewBox="0 0 64 64" fill="none" aria-hidden="true">
          <circle cx="32" cy="28" r="17" stroke="currentColor" stroke-width="2.4" />
          <circle cx="32" cy="28" r="5.5" fill="currentColor" />
          <path
            d="M8 51q6-4 12 0t12 0 12 0 12 0"
            stroke="currentColor"
            stroke-width="2.4"
            stroke-linecap="round"
          />
        </svg>
      </div>

      <h1 class="brand">听屿</h1>
      <p class="slogan">一座只放你自己音乐的小岛</p>

      <template v-if="booting">
        <p class="checking">正在检查登录状态…</p>
      </template>

      <template v-else>
        <div class="mark" />
        <p class="desc">使用华为账号扫码登录，收听你已有权益的音乐。</p>

        <ul class="features">
          <li>
            <Icon name="spark" :size="15" />
            <span>官方扫码授权，不接触你的密码</span>
          </li>
          <li>
            <Icon name="note" :size="15" />
            <span>只在本机保存最小音乐会话</span>
          </li>
          <li>
            <Icon name="heart" :size="15" />
            <span>不下载、不解密、不转码</span>
          </li>
        </ul>

        <button class="primary" :disabled="loggingIn" @click="emit('login')">
          <Icon name="spark" :size="16" />
          {{ loggingIn ? "等待授权…" : "扫码登录" }}
        </button>

        <p v-if="error" class="error">{{ error }}</p>

        <p class="footnote">登录即表示你确认仅播放本人账号已有权益的音频</p>
      </template>
    </div>
  </div>
</template>

<style scoped>
.login {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
  background: var(--base);
}

/* 窗口按钮挂在最上沿，和主界面标题栏保持同一套操作位置 */
.winbar {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: var(--titlebar-h);
  display: flex;
  align-items: stretch;
  justify-content: flex-end;
  z-index: 2;
}

.drag {
  flex: 1;
  height: 100%;
}

.win {
  width: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--muted);
}

.win:hover {
  color: var(--text);
  background: var(--surface);
}

.win.close:hover {
  color: #fff;
  background: #c8443c;
}

.card {
  width: 420px;
  max-width: calc(100vw - 48px);
  padding: 48px 44px 34px;
  text-align: center;
  background: var(--bg);
  border-radius: var(--r-xl);
  box-shadow: var(--stage-shadow);
}

.logo {
  display: flex;
  justify-content: center;
  color: var(--accent);
  margin-bottom: 14px;
}

.brand {
  font-size: 25px;
  font-weight: 600;
  letter-spacing: 5px;
  text-indent: 5px;
}

.slogan {
  margin-top: 7px;
  font-size: 12px;
  letter-spacing: 1px;
  color: var(--faint);
}

.checking {
  margin-top: 34px;
  font-size: 12px;
  color: var(--faint);
}

/* 不用 1px 分割线，靠一小段强调色色块做视觉停顿 */
.mark {
  width: 26px;
  height: 3px;
  margin: 24px auto 0;
  border-radius: 2px;
  background: var(--accent);
  opacity: 0.75;
}

.desc {
  margin-top: 20px;
  font-size: 13px;
  line-height: 1.7;
  color: var(--muted);
}

.features {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin: 24px 0 30px;
  list-style: none;
  text-align: left;
}

.features li {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 12px;
  color: var(--muted);
}

.features svg {
  flex: none;
  color: var(--accent);
}

.primary {
  width: 100%;
  height: 42px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: 13px;
  color: var(--bg);
  background: var(--text);
  border-radius: var(--r-md);
  transition: opacity 0.15s ease;
}

.primary:hover:not(:disabled) {
  opacity: 0.85;
}

.primary:disabled {
  opacity: 0.5;
  cursor: default;
}

.error {
  margin-top: 14px;
  font-size: 12px;
  line-height: 1.5;
  color: var(--danger);
}

.footnote {
  margin-top: 22px;
  font-size: 11px;
  line-height: 1.6;
  color: var(--faint);
}
</style>
