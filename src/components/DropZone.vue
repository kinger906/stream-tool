<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";

const emit = defineEmits<{
  add: [paths: string[]];
  addAndStart: [paths: string[]];
  addCamera: [];
  addDisplay: [];
  addWindow: [];
  addRegion: [];
}>();

const dragging = ref(false);

const videoExt =
  /\.(mp4|mkv|avi|mov|wmv|flv|webm|m4v|ts|m2ts|mpg|mpeg|vob|3gp)$/i;

const videoExtensions = [
  "mp4",
  "mkv",
  "avi",
  "mov",
  "wmv",
  "flv",
  "webm",
  "m4v",
  "ts",
  "m2ts",
  "mpg",
  "mpeg",
  "vob",
  "3gp",
];

function filterVideos(paths: string[]) {
  return paths.filter((p) => videoExt.test(p));
}

async function pickFiles(andStart: boolean) {
  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "视频",
        extensions: videoExtensions,
      },
    ],
  });
  if (!selected) return;
  const paths = Array.isArray(selected) ? selected : [selected];
  if (andStart) emit("addAndStart", paths);
  else emit("add", paths);
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
    <div class="main" @click="pickFiles(false)">
      <div class="icon">+</div>
      <p>拖拽或点击选择本地视频（支持超大文件）</p>
      <p class="hint">
        推流后终端用 RTSP/HLS 边播边收，无需整文件加载，缓解大视频卡顿
      </p>
    </div>
    <div class="actions">
      <button class="btn primary" type="button" @click.stop="pickFiles(true)">
        大视频一键推流
      </button>
      <button class="btn" type="button" @click.stop="pickFiles(false)">
        仅添加文件
      </button>
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
.actions {
  margin-top: 14px;
  display: flex;
  gap: 10px;
  justify-content: center;
  flex-wrap: wrap;
}
.btn.primary {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
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
