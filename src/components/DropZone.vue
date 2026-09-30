<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";

const emit = defineEmits<{
  add: [paths: string[]];
  addCamera: [];
  addDisplay: [];
  addWindow: [];
  addRegion: [];
}>();

const dragging = ref(false);

const videoExt = /\.(mp4|mkv|avi|mov|wmv|flv|webm|m4v|ts)$/i;

function filterVideos(paths: string[]) {
  return paths.filter((p) => videoExt.test(p));
}

async function pickFiles() {
  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "视频",
        extensions: ["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "ts"],
      },
    ],
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
  const paths = filterVideos(
    Array.from(files).map((f) => (f as File & { path?: string }).path || f.name),
  );
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
  >
    <div class="main" @click="pickFiles">
      <div class="icon">+</div>
      <p>拖拽视频到此处，或点击选择文件</p>
      <p class="hint">支持多路并行、RTMP 转推、自动重连与场景预设</p>
    </div>
    <div class="sources">
      <button class="btn source" type="button" @click.stop="emit('addCamera')">
        摄像头
      </button>
      <button class="btn source" type="button" @click.stop="emit('addDisplay')">
        桌面
      </button>
      <button class="btn source" type="button" @click.stop="emit('addWindow')">
        窗口
      </button>
      <button class="btn source" type="button" @click.stop="emit('addRegion')">
        区域截取
      </button>
    </div>
  </div>
</template>

<style scoped>
.dropzone {
  border: 2px dashed var(--border);
  border-radius: 12px;
  padding: 24px;
  text-align: center;
  background: var(--surface);
  transition: border-color 0.2s, background 0.2s;
}
.dropzone:hover,
.dropzone.active {
  border-color: var(--accent);
  background: var(--surface-2);
}
.main {
  cursor: pointer;
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
.sources {
  margin-top: 16px;
  display: flex;
  gap: 10px;
  justify-content: center;
  flex-wrap: wrap;
}
.btn.source {
  border-color: var(--accent);
  color: #69b1ff;
}
</style>
