<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";

const emit = defineEmits<{ add: [paths: string[]] }>();
const dragging = ref(false);

const videoExt = /\.(mp4|mkv|avi|mov|wmv|flv|webm|m4v|ts)$/i;

function filterVideos(paths: string[]) {
  return paths.filter((p) => videoExt.test(p));
}

async function pickFiles() {
  const selected = await open({
    multiple: true,
    filters: [{ name: "视频", extensions: ["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "ts"] }],
  });
  if (!selected) return;
  const paths = Array.isArray(selected) ? selected : [selected];
  emit("add", paths);
}

function onDrop(e: DragEvent) {
  dragging.value = false;
  e.preventDefault();
  const files = e.dataTransfer?.files;
  if (!files?.length) return;
  const paths = filterVideos(Array.from(files).map((f) => (f as File & { path?: string }).path || f.name));
  if (paths.length) emit("add", paths);
}

function onDragOver(e: DragEvent) {
  e.preventDefault();
  dragging.value = true;
}

function onDragLeave() {
  dragging.value = false;
}
</script>

<template>
  <div
    class="dropzone"
    :class="{ active: dragging }"
    @drop="onDrop"
    @dragover="onDragOver"
    @dragleave="onDragLeave"
    @click="pickFiles"
  >
    <div class="icon">+</div>
    <p>拖拽视频到此处，或点击选择文件</p>
    <p class="hint">支持多文件并行推流，每路生成独立 RTSP / HLS 地址</p>
    <div class="future">
      <span class="tag soon">摄像头（即将支持）</span>
      <span class="tag soon">桌面采集（即将支持）</span>
    </div>
  </div>
</template>

<style scoped>
.dropzone {
  border: 2px dashed var(--border);
  border-radius: 12px;
  padding: 28px;
  text-align: center;
  cursor: pointer;
  background: var(--surface);
  transition: border-color 0.2s, background 0.2s;
}
.dropzone:hover,
.dropzone.active {
  border-color: var(--accent);
  background: var(--surface-2);
}
.icon {
  font-size: 28px;
  color: var(--accent);
  margin-bottom: 8px;
}
.hint {
  color: var(--text-muted);
  font-size: 13px;
  margin-top: 6px;
}
.future {
  margin-top: 14px;
  display: flex;
  gap: 8px;
  justify-content: center;
}
.tag.soon {
  font-size: 11px;
  padding: 3px 8px;
  border-radius: 4px;
  background: var(--surface-2);
  color: var(--text-muted);
  border: 1px dashed var(--border);
}
</style>
