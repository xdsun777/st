import { createApp } from "vue";
import { createPinia, getActivePinia } from "pinia";
import App from "./App.vue";
import { useAppStore } from "./stores/app";
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
