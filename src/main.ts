import { createApp } from "vue";
import { createPinia, getActivePinia } from "pinia";
import App from "./App.vue";
import { useAppStore, type ThemeMode } from "./stores/app";
import { getSetting } from "./api";
import "./style.css";

const app = createApp(App);
const pinia = createPinia();
app.use(pinia);

// 全局异常兜底：组件渲染/事件处理中的未捕获错误 → 全局提示
app.config.errorHandler = (err) => {
  console.error("[app error]", err);
  if (getActivePinia()) {
    useAppStore().pushToast(err instanceof Error ? err.message : String(err), "error");
  }
};

// Promise 未处理拒绝兜底
window.addEventListener("unhandledrejection", (event) => {
  console.error("[unhandledrejection]", event.reason);
  if (getActivePinia()) {
    const reason =
      event.reason instanceof Error ? event.reason.message : String(event.reason);
    useAppStore().pushToast(reason, "error");
  }
});

app.mount("#app");

// 主题初始化：读取持久化主题模式 + 监听系统深色偏好
(async () => {
  const store = useAppStore();
  const media = matchMedia("(prefers-color-scheme: dark)");
  let saved: string | null = null;
  try {
    saved = await getSetting("theme_mode");
  } catch {
    // 读取失败用默认值（跟随系统）
  }
  const mode: ThemeMode =
    saved === "light" || saved === "dark" || saved === "system" ? saved : "system";
  store.applyTheme(mode, media.matches);
  media.addEventListener("change", (e) => {
    if (store.themeMode === "system") {
      store.applyTheme("system", e.matches);
    }
  });
})();
