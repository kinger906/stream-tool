<script setup lang="ts">
import type { DependencyStatus } from "../types";

defineProps<{
  deps: DependencyStatus;
}>();
</script>

<template>
  <div v-if="!deps.ffmpegAvailable || !deps.mediamtxAvailable" class="banner warn">
    <strong>依赖缺失：</strong>
    <span v-if="!deps.ffmpegAvailable">FFmpeg → {{ deps.ffmpegPathHint }}</span>
    <span v-if="!deps.mediamtxAvailable">MediaMTX → {{ deps.mediamtxPathHint }}</span>
    <span class="hint">运行 <code>npm run fetch-sidecars</code> 自动下载，或手动放入上述路径。</span>
  </div>
</template>

<style scoped>
.banner {
  padding: 10px 14px;
  border-radius: 8px;
  font-size: 13px;
  line-height: 1.5;
}
.warn {
  background: #3d2a00;
  border: 1px solid #8a6400;
  color: #ffd666;
}
.hint {
  display: block;
  margin-top: 4px;
  opacity: 0.85;
}
code {
  background: rgba(0, 0, 0, 0.25);
  padding: 1px 5px;
  border-radius: 4px;
}
</style>
