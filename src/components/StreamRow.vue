<script setup lang="ts">
import { computed, ref } from "vue";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import type { QualityPreset, StreamTaskInfo } from "../types";

const props = defineProps<{ stream: StreamTaskInfo }>();

const emit = defineEmits<{
  start: [];
  stop: [];
  remove: [];
  preview: [];
  qr: [];
  toggleLoop: [enabled: boolean];
  toggleCopy: [copyMode: boolean];
  toggleRecord: [enabled: boolean];
  updateStreamName: [name: string];
  updateQuality: [preset: QualityPreset];
}>();

const copiedKey = ref<string | null>(null);
const editingName = ref(false);
const nameDraft = ref("");
let copiedTimer: ReturnType<typeof setTimeout> | null = null;

const statusLabel: Record<string, string> = {
  idle: "待推流",
  running: "推流中",
  stopped: "已停止",
  error: "失败",
};

const sourceLabel: Record<string, string> = {
  file: "文件",
  camera: "摄像头",
  display: "桌面",
  window: "窗口",
};

const qualityLabel: Record<QualityPreset, string> = {
  low: "低 (360p)",
  medium: "中 (720p)",
  high: "高 (1080p)",
};

const isFile = computed(() => props.stream.sourceType === "file");
const isLive = computed(() => props.stream.sourceType !== "file");

const progressText = computed(() => {
  const { elapsedSecs, durationSecs, loopEnabled, sourceType, fps, bitrateKbps } =
    props.stream;
  const elapsed = formatTime(elapsedSecs);
  let base =
    sourceType !== "file"
      ? `${elapsed}（直播）`
      : durationSecs == null
        ? elapsed
        : loopEnabled
          ? `${elapsed} / ${formatTime(durationSecs)}（循环）`
          : `${elapsed} / ${formatTime(durationSecs)}`;

  if (fps != null || bitrateKbps != null) {
    const parts = [];
    if (fps != null) parts.push(`${fps.toFixed(1)} fps`);
    if (bitrateKbps != null) parts.push(`${bitrateKbps.toFixed(0)} kbps`);
    base += ` · ${parts.join(" · ")}`;
  }
  return base;
});

function formatTime(secs: number) {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = Math.floor(secs % 60);
  return h > 0
    ? `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`
    : `${m}:${String(s).padStart(2, "0")}`;
}

async function copy(label: string, text: string) {
  try {
    await writeText(text);
    copiedKey.value = label;
    if (copiedTimer) clearTimeout(copiedTimer);
    copiedTimer = setTimeout(() => {
      copiedKey.value = null;
    }, 1500);
  } catch (err) {
    console.error("clipboard write failed", err);
    window.prompt("复制失败，请手动复制：", text);
  }
}

function startEditName() {
  if (props.stream.status === "running") return;
  nameDraft.value = props.stream.streamName;
  editingName.value = true;
}

function commitName() {
  editingName.value = false;
  if (nameDraft.value && nameDraft.value !== props.stream.streamName) {
    emit("updateStreamName", nameDraft.value);
  }
}
</script>

<template>
  <tr :class="stream.status">
    <td class="name">
      <div class="filename">
        <span class="source-tag">{{ sourceLabel[stream.sourceType] }}</span>
        {{ stream.filename }}
      </div>
      <div class="stream-name">
        <template v-if="editingName">
          <input
            v-model="nameDraft"
            class="name-input"
            @keyup.enter="commitName"
            @blur="commitName"
          />
        </template>
        <template v-else>
          <code>/{{ stream.streamName }}</code>
          <button
            v-if="stream.status !== 'running'"
            class="btn link"
            type="button"
            @click="startEditName"
          >
            编辑
          </button>
        </template>
      </div>
      <div class="id">{{ stream.id }}</div>
    </td>
    <td>
      <span class="badge" :class="stream.status">{{ statusLabel[stream.status] }}</span>
      <div v-if="stream.error" class="error">{{ stream.error }}</div>
    </td>
    <td class="progress">{{ progressText }}</td>
    <td class="urls">
      <div class="url-row">
        <span class="label">RTSP</span>
        <code>{{ stream.rtspUrl }}</code>
        <button class="btn sm" type="button" @click.stop="copy('rtsp', stream.rtspUrl)">
          {{ copiedKey === "rtsp" ? "已复制" : "复制" }}
        </button>
      </div>
      <div class="url-row">
        <span class="label">HLS</span>
        <code>{{ stream.hlsUrl }}</code>
        <button class="btn sm" type="button" @click.stop="copy('hls', stream.hlsUrl)">
          {{ copiedKey === "hls" ? "已复制" : "复制" }}
        </button>
      </div>
    </td>
    <td class="opts">
      <template v-if="isFile">
        <label class="check">
          <input
            type="checkbox"
            :checked="stream.loopEnabled"
            :disabled="stream.status === 'running'"
            @change="$emit('toggleLoop', ($event.target as HTMLInputElement).checked)"
          />
          循环
        </label>
        <label class="check">
          <input
            type="checkbox"
            :checked="stream.copyMode"
            :disabled="stream.status === 'running'"
            @change="$emit('toggleCopy', ($event.target as HTMLInputElement).checked)"
          />
          Copy
        </label>
      </template>
      <label v-if="isLive" class="field-inline">
        <span>画质</span>
        <select
          :value="stream.qualityPreset"
          :disabled="stream.status === 'running'"
          @change="
            $emit('updateQuality', ($event.target as HTMLSelectElement).value as QualityPreset)
          "
        >
          <option v-for="(label, key) in qualityLabel" :key="key" :value="key">
            {{ label }}
          </option>
        </select>
      </label>
      <label class="check">
        <input
          type="checkbox"
          :checked="stream.recordEnabled"
          :disabled="stream.status === 'running'"
          @change="$emit('toggleRecord', ($event.target as HTMLInputElement).checked)"
        />
        录制
      </label>
    </td>
    <td class="actions">
      <button
        v-if="stream.status !== 'running'"
        class="btn sm primary"
        type="button"
        @click="$emit('start')"
      >
        开始
      </button>
      <button v-else class="btn sm" type="button" @click="$emit('stop')">停止</button>
      <button class="btn sm" type="button" @click="$emit('preview')">预览</button>
      <button class="btn sm" type="button" @click="$emit('qr')">二维码</button>
      <button class="btn sm danger" type="button" @click="$emit('remove')">删除</button>
    </td>
  </tr>
</template>

<style scoped>
tr.running {
  background: rgba(22, 119, 255, 0.06);
}
tr.error {
  background: rgba(255, 77, 79, 0.06);
}
.name .filename {
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 6px;
}
.stream-name {
  margin-top: 4px;
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
}
.stream-name code {
  color: #69b1ff;
}
.name-input {
  width: 120px;
  font-size: 12px;
  padding: 2px 6px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 4px;
  color: inherit;
}
.btn.link {
  padding: 0 4px;
  font-size: 11px;
  border: none;
  background: none;
  color: var(--text-muted);
  cursor: pointer;
}
.source-tag {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 4px;
  background: #111d2c;
  color: #69b1ff;
  flex-shrink: 0;
}
.name .id {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
}
.badge {
  display: inline-block;
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  background: var(--surface-2);
}
.badge.running {
  color: #69b1ff;
  background: #111d2c;
}
.badge.error {
  color: #ff7875;
  background: #2a1215;
}
.badge.idle {
  color: var(--text-muted);
}
.error {
  margin-top: 4px;
  font-size: 11px;
  color: #ff7875;
  max-width: 180px;
}
.progress {
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
  max-width: 200px;
}
.urls {
  min-width: 280px;
}
.url-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
}
.url-row .label {
  font-size: 10px;
  color: var(--text-muted);
  width: 32px;
}
.url-row code {
  flex: 1;
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 220px;
}
.opts {
  font-size: 12px;
}
.check {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-bottom: 4px;
  cursor: pointer;
}
.field-inline {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 6px;
  font-size: 11px;
}
.field-inline select {
  font-size: 11px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  color: inherit;
  border-radius: 4px;
  padding: 2px 4px;
}
.actions {
  white-space: nowrap;
}
</style>
