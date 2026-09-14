<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
/**
 * 桌面端自定义标题栏（依据 Tauri v2 窗口自定义文档）。
 * 仅桌面端显示（hidden md:flex）；Android 端保持全屏无标题栏。
 * 根元素带 data-tauri-drag-region 使整条区域可拖拽移动窗口；
 * 双击切换最大化/还原。
 */
const appWindow = getCurrentWindow();

function minimize() {
  void appWindow.minimize();
}

function toggleMaximize() {
  void appWindow.toggleMaximize();
}

function close() {
  void appWindow.close();
}
</script>

<template>
  <header data-tauri-drag-region
    class="hidden h-9 shrink-0 select-none items-center justify-between border-b border-gray-200 bg-white pl-4 md:flex"
    @dblclick="toggleMaximize">
    <div class="pointer-events-none flex items-center gap-2 text-xs font-medium text-gray-500">
      <img src="@assets/st-128x128.svg" alt="logo" class="pointer-events-none h-5 w-5 select-none" />
      
    </div>

    <div class="flex h-full items-stretch">
      <button class="titlebar-button" title="最小化" @click="minimize">
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <line x1="0" y1="5" x2="10" y2="5" stroke="currentColor" stroke-width="1.2" />
        </svg>
      </button>
      <button class="titlebar-button" title="最大化/还原" @click="toggleMaximize">
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <rect x="0.6" y="0.6" width="8.8" height="8.8" fill="none" stroke="currentColor" stroke-width="1.2" />
        </svg>
      </button>
      <button class="titlebar-button titlebar-button--close" title="退出" @click="close">
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <line x1="0.8" y1="0.8" x2="9.2" y2="9.2" stroke="currentColor" stroke-width="1.2" />
          <line x1="9.2" y1="0.8" x2="0.8" y2="9.2" stroke="currentColor" stroke-width="1.2" />
        </svg>
      </button>
    </div>
  </header>
</template>

