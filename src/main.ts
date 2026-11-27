import { createApp } from "vue";
import App from "./App.vue";
import { initTheme } from "./composables/useTheme";
import "./styles.css";

// 挂载前先落主题属性，首屏不会按旧配色闪一下。
initTheme();
createApp(App).mount("#app");
