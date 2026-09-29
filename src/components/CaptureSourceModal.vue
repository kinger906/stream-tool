<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api } from "../api/tauri";
import type { CaptureDevice, QualityPreset } from "../types";

const props = defineProps<{
  mode: "camera" | "display" | "window";
}>();

const emit = defineEmits<{
  close: [];
  confirm: [
    payload: {
      videoDevice: string | null;
      windowTitle: string | null;
      audioDevice: string | null;
      qualityPreset: QualityPreset;
    },
  ];
}>();

const loading = ref(true);
const error = ref<string | null>(null);
const videoDevices = ref<CaptureDevice[]>([]);
const audioDevices = ref<CaptureDevice[]>([]);
const windowList = ref<CaptureDevice[]>([]);
const selectedVideo = ref("");
const selectedWindow = ref("");
const selectedAudio = ref("");
const includeAudio = ref(true);
const qualityPreset = ref<QualityPreset>("medium");

const titleMap = {
  camera: "添加摄像头推流",
  display: "添加桌面采集推流",
  window: "添加窗口采集推流",
};

onMounted(async () => {
  try {
    const [devices, windows] = await Promise.all([
      api.listCaptureDevices(),
      props.mode === "window" ? api.listWindows() : Promise.resolve([]),
    ]);
    videoDevices.value = devices.video;
    audioDevices.value = devices.audio;
    windowList.value = windows;

    if (props.mode === "camera" && devices.video.length) {
      selectedVideo.value = devices.video[0].name;
    }
    if (props.mode === "window" && windows.length) {
      selectedWindow.value = windows[0].name;
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
  if (props.mode === "window" && !selectedWindow.value) {
    error.value = "请选择窗口";
    return;
  }
  const audio =
    includeAudio.value && selectedAudio.value ? selectedAudio.value : null;
  emit("confirm", {
    videoDevice: props.mode === "camera" ? selectedVideo.value : null,
    windowTitle: props.mode === "window" ? selectedWindow.value : null,
    audioDevice: audio,
    qualityPreset: qualityPreset.value,
  });
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="panel">
      <div class="panel-head">
        <h3>{{ titleMap[mode] }}</h3>
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

        <label v-if="mode === 'window'" class="field">
          <span>窗口</span>
          <select v-model="selectedWindow" :disabled="!windowList.length">
            <option v-if="!windowList.length" value="">未检测到可见窗口</option>
            <option v-for="w in windowList" :key="w.name" :value="w.name">
              {{ w.name }}
            </option>
          </select>
        </label>

        <label v-if="mode === 'display'" class="hint-block">
          将采集整个桌面画面（Windows gdigrab）。
        </label>

        <label class="field">
          <span>画质预设</span>
          <select v-model="qualityPreset">
            <option value="low">低 (640×360)</option>
            <option value="medium">中 (1280×720)</option>
            <option value="high">高 (1920×1080)</option>
          </select>
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
          :disabled="
            loading ||
            (mode === 'camera' && !videoDevices.length) ||
            (mode === 'window' && !windowList.length)
          "
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
