<script setup lang="ts">
import { judgeFillAnswer } from "../utils/judge";

// 核心纯函数自检（技术文档 7.1 填空判分）
const fillExample = judgeFillAnswer("TCP", "TCP;传输控制协议");
</script>

<template>
  <section>
    <h2 class="text-xl font-semibold">刷题</h2>
    <p class="mb-4 mt-1 text-sm text-gray-500">
      顺序 / 随机刷题，支持单选、多选、判断、简答、填空。刷题核心流程开发中。
    </p>

    <div class="space-y-3 rounded-xl border border-gray-200 bg-white p-4 text-sm sm:p-6">
      <div class="text-gray-500">题型与判分规则（业务文档 4.1）：</div>
      <ul class="list-disc space-y-1 pl-5 text-gray-600">
        <li>single 单选：机器判分，1 个正确选项，答错自动入错题</li>
        <li>multi 多选：必须全部匹配才算正确，多选、少选、错选均判错</li>
        <li>judge 判断：true / false，答错自动入错题</li>
        <li>essay 简答：无机器判分，手动标记对错</li>
        <li>fill 填空：机器判分，多标准答案 ; 分隔，可手动覆写对错</li>
      </ul>
      <div class="border-t border-gray-100 pt-3 text-xs text-gray-400">
        填空判分函数自检：judgeFillAnswer("TCP", "TCP;传输控制协议") =
        {{ fillExample ? "正确" : "错误" }}
      </div>
    </div>
  </section>
</template>
