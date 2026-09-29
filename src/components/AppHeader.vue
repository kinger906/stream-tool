<script setup lang="ts">
import type { SystemInfo } from "../types";

defineProps<{
  systemInfo: SystemInfo | null;
}>();

defineEmits<{
  startAll: [];
  stopAll: [];
}>();
</script>

<template>
  <header class="header">
    <div class="brand">
      <h1>视频推流工具</h1>
      <p class="subtitle">文件 / 摄像头 / 桌面 / 窗口 → RTSP / HLS / WebRTC，支持录制与认证</p>
    </div>
    <div class="meta" v-if="systemInfo">
      <div class="chip" :class="{ ok: systemInfo.mediamtxRunning }">
        MediaMTX {{ systemInfo.mediamtxRunning ? "运行中" : "未运行" }}
      </div>
      <div class="chip">局域网 IP：{{ systemInfo.lanIp }}</div>
      <div class="chip">RTSP 8554 / HLS 8888 / WebRTC 8889</div>
      <div class="chip">并发上限 {{ systemInfo.settings.maxConcurrent }}</div>
    </div>
    <div class="actions">
      <button class="btn primary" @click="$emit('startAll')">全部开始</button>
      <button class="btn" @click="$emit('stopAll')">全部停止</button>
    </div>
  </header>
</template>

<style scoped>
.header {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  align-items: flex-start;
  justify-content: space-between;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border);
}
.brand h1 {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
}
.subtitle {
  margin: 4px 0 0;
  color: var(--text-muted);
  font-size: 13px;
}
.meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  flex: 1;
  justify-content: center;
}
.chip {
  font-size: 12px;
  padding: 4px 10px;
  border-radius: 999px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  color: var(--text-muted);
}
.chip.ok {
  color: #52c41a;
  border-color: #274916;
  background: #162312;
}
.actions {
  display: flex;
  gap: 8px;
}
</style>
