<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api } from "../api/tauri";
import type { CaptureDevice } from "../types";

const props = defineProps<{
  mode: "camera" | "display";
}>();

const emit = defineEmits<{
  close: [];
  confirm: [videoDevice: string | null, audioDevice: string | null];
}>();

const loading = ref(true);
const error = ref<string | null>(null);
const videoDevices = ref<CaptureDevice[]>([]);
const audioDevices = ref<CaptureDevice[]>([]);
const selectedVideo = ref("");
const selectedAudio = ref("");
const includeAudio = ref(true);

onMounted(async () => {
  try {
    const devices = await api.listCaptureDevices();
    videoDevices.value = devices.video;
    audioDevices.value = devices.audio;
    if (props.mode === "camera" && devices.video.length) {
      selectedVideo.value = devices.video[0].name;
    }
    if (devices.audio.length) {
      selectedAudio.value = devices.audio[0].name;
    }
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
});

function submit() {
  if (props.mode === "camera" && !selectedVideo.value) {
    error.value = "请选择摄像头";
    return;
  }
  const audio =
    includeAudio.value && selectedAudio.value ? selectedAudio.value : null;
  emit(
    "confirm",
    props.mode === "camera" ? selectedVideo.value : null,
    audio,
  );
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="panel">
      <div class="panel-head">
        <h3>{{ mode === "camera" ? "添加摄像头推流" : "添加桌面采集推流" }}</h3>
        <button class="btn" type="button" @click="emit('close')">取消</button>
      </div>

      <div v-if="loading" class="body">正在枚举设备…</div>
      <div v-else class="body">
        <p v-if="error" class="error">{{ error }}</p>

        <label v-if="mode === 'camera'" class="field">
          <span>摄像头</span>
          <select v-model="selectedVideo" :disabled="!videoDevices.length">
            <option v-if="!videoDevices.length" value="">未检测到摄像头</option>
            <option v-for="d in videoDevices" :key="d.name" :value="d.name">
              {{ d.name }}
            </option>
          </select>
        </label>

        <label v-if="mode === 'display'" class="hint-block">
          将采集整个桌面画面（Windows gdigrab），帧率 25fps。
        </label>

        <label class="check">
          <input v-model="includeAudio" type="checkbox" />
          同时采集麦克风
        </label>

        <label v-if="includeAudio" class="field">
          <span>麦克风</span>
          <select v-model="selectedAudio" :disabled="!audioDevices.length">
            <option v-if="!audioDevices.length" value="">未检测到麦克风</option>
            <option v-for="d in audioDevices" :key="d.name" :value="d.name">
              {{ d.name }}
            </option>
          </select>
        </label>
      </div>

      <div class="footer">
        <button
          class="btn primary"
          type="button"
          :disabled="loading || (mode === 'camera' && !videoDevices.length)"
          @click="submit"
        >
          添加
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.72);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}
.panel {
  width: min(480px, 92vw);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 12px;
  overflow: hidden;
}
.panel-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
}
.panel-head h3 {
  margin: 0;
  font-size: 15px;
}
.body {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 13px;
}
.field select {
  background: var(--surface-2);
  border: 1px solid var(--border);
  color: inherit;
  border-radius: 6px;
  padding: 8px;
}
.check {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  cursor: pointer;
}
.hint-block {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.5;
}
.error {
  color: #ff7875;
  font-size: 13px;
  margin: 0;
}
.footer {
  padding: 12px 16px;
  border-top: 1px solid var(--border);
  display: flex;
  justify-content: flex-end;
}
</style>
