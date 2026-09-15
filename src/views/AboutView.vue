<script setup lang="ts">
import { onMounted, ref } from "vue";
import { getVersion } from "@tauri-apps/api/app";
import logo from "@assets/st-128x128.svg";

/**
 * 关于页：寄语、项目信息、开源致谢、开源许可。
 */
const version = ref("0.1.0");

/** 寄语使用中文衬线/楷体字体栈，营造书卷感 */
const quoteFont =
  '"Noto Serif SC", "Noto Serif CJK SC", "Source Han Serif SC", "Source Han Serif CN", "Songti SC", "STSong", "SimSun", "KaiTi", "STKaiti", serif';

onMounted(async () => {
  try {
    version.value = await getVersion();
  } catch {
    // 非 Tauri 环境或读取失败时回退到 package.json 版本号
  }
});
</script>

<template>
  <section>

    <!-- 寄语 -->
    <div class="mb-4 rounded-xl border border-gray-200 bg-white p-6 sm:p-8">
      <blockquote class="border-l-4 border-blue-300 pl-4 sm:pl-5">
        <p class="text-base leading-relaxed text-gray-600" :style="{ fontFamily: quoteFont }">
          学习的本质就是：重复、极致的重复，坚持、极致的坚持，直到熟能生巧、巧能运用、最终练出绝技。
        </p>
        <p class="mt-4 text-base leading-relaxed text-gray-600" :style="{ fontFamily: quoteFont }">
          如果没有天赋，那就一直重复。要相信，那些你咬牙坚持的 “再来一次”，终将在未来的某一天，成为旁人望尘莫及的天赋。
        </p>
      </blockquote>
    </div>
    <!-- 应用信息 -->
    <div class="mb-4 flex items-center gap-4 rounded-xl border border-gray-200 bg-white p-5">
      <img :src="logo" alt="再刷一题" class="h-16 w-26 shrink-0" />
      <div class="min-w-0">
        <div class="text-base font-semibold">再刷一题</div>
        <div class="mt-0.5 text-sm text-gray-400">版本 v{{ version }}</div>
      </div>
      
    </div>


    <!-- 项目信息 -->
    <div class="mb-4 rounded-xl border border-gray-200 bg-white p-4">
      <h3 class="mb-2 text-sm font-medium">项目信息</h3>
      <ul class="space-y-1.5 text-sm text-gray-600">
        <li>
          <span class="text-gray-400">GitHub：</span>
          <a href="https://github.com/xdsun777/st" target="_blank" rel="noopener noreferrer"
            class="text-blue-600 hover:underline">github.com/xdsun777/st</a>
        </li>
        <li><span class="text-gray-400">作者：</span>xdsun777</li>
      </ul>
      
    </div>

    <!-- 开源许可 -->
    <div class="rounded-xl border border-gray-200 bg-white p-4">
      <h3 class="mb-2 text-sm font-medium">开源许可</h3>
      <p class="text-sm text-gray-600">
        本项目基于
        <a href="https://opensource.org/licenses/MIT" target="_blank" rel="noopener noreferrer"
          class="text-blue-600 hover:underline">MIT License</a>
        开源。
      </p>
    </div>
  </section>
</template>
