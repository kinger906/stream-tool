<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import AppHeader from "./components/AppHeader.vue";
import CaptureSourceModal from "./components/CaptureSourceModal.vue";
import DependencyBanner from "./components/DependencyBanner.vue";
import DropZone from "./components/DropZone.vue";
import PreviewPlayer from "./components/PreviewPlayer.vue";
import QrCodeModal from "./components/QrCodeModal.vue";
import ScenesPanel from "./components/ScenesPanel.vue";
import SettingsPanel from "./components/SettingsPanel.vue";
import StreamList from "./components/StreamList.vue";
import { useStreams } from "./composables/useStreams";
import { api } from "./api/tauri";
import type { CaptureRegion, QualityPreset, StreamTaskInfo } from "./types";

const {
  streams,
  systemInfo,
  scenes,
  error,
  notice,
  addPaths,
  addAndStartPaths,
  start,
  stop,
  startAll,
  stopAll,
  remove,
  toggleLoop,
  toggleCopyMode,
  updateStreamName,
  toggleRecord,
  toggleReconnect,
  updateQuality,
  updateRtmpUrl,
  updateSettings,
  saveScene,
  deleteScene,
  applyScene,
  refresh,
} = useStreams();

const preview = ref<StreamTaskInfo | null>(null);
const qrStream = ref<StreamTaskInfo | null>(null);
const captureMode = ref<"camera" | "display" | "window" | "region" | null>(null);
let unlistenDrop: (() => void) | null = null;

onMounted(async () => {
  unlistenDrop = await getCurrentWebviewWindow().onDragDropEvent((event) => {
    if (event.payload.type === "drop") {
      addPaths(event.payload.paths);
    }
  });
});

onUnmounted(() => {
  unlistenDrop?.();
});

function openPreview(stream: StreamTaskInfo) {
  if (stream.status !== "running") {
    error.value =
      "请先点击「开始」并确认状态为「推流中」后再预览。未推流时 HLS/WebRTC 都会 404。";
    return;
  }
  error.value = null;
  preview.value = stream;
}

async function onCaptureConfirm(payload: {
  videoDevice: string | null;
  windowTitle: string | null;
  audioDevice: string | null;
  qualityPreset: QualityPreset;
  region: CaptureRegion | null;
}) {
  try {
    let stream: StreamTaskInfo | undefined;
    if (captureMode.value === "camera" && payload.videoDevice) {
      stream = await api.addCameraStream(
        payload.videoDevice,
        payload.audioDevice ?? undefined,
      );
    } else if (captureMode.value === "display") {
      stream = await api.addDisplayStream(payload.audioDevice ?? undefined);
    } else if (captureMode.value === "window" && payload.windowTitle) {
      stream = await api.addWindowStream(
        payload.windowTitle,
        payload.audioDevice ?? undefined,
      );
    } else if (captureMode.value === "region" && payload.region) {
      stream = await api.addRegionStream(
        payload.region,
        payload.audioDevice ?? undefined,
      );
    }
    if (stream && payload.qualityPreset !== "medium") {
      await api.updateStream(stream.id, { qualityPreset: payload.qualityPreset });
    }
    captureMode.value = null;
    await refresh();
  } catch (e) {
    error.value = String(e);
  }
}
</script>

<template>
  <div class="app-shell">
    <AppHeader
      :system-info="systemInfo"
      @start-all="startAll"
      @stop-all="stopAll"
    />

    <DependencyBanner
      v-if="systemInfo"
      :deps="systemInfo.dependencies"
    />

    <div v-if="error" class="alert">{{ error }}</div>
    <div v-if="notice" class="notice">{{ notice }}</div>

    <DropZone
      @add="addPaths"
      @add-and-start="addAndStartPaths"
      @add-camera="captureMode = 'camera'"
      @add-display="captureMode = 'display'"
      @add-window="captureMode = 'window'"
      @add-region="captureMode = 'region'"
    />

    <StreamList
      :streams="streams"
      @start="start"
      @stop="stop"
      @remove="remove"
      @preview="openPreview"
      @qr="qrStream = $event"
      @toggle-loop="toggleLoop"
      @toggle-copy="toggleCopyMode"
      @toggle-record="toggleRecord"
      @toggle-reconnect="toggleReconnect"
      @update-stream-name="updateStreamName"
      @update-quality="updateQuality"
      @update-rtmp="updateRtmpUrl"
    />

    <ScenesPanel
      :scenes="scenes"
      @save="saveScene"
      @apply="applyScene"
      @remove="deleteScene"
    />

    <SettingsPanel
      v-if="systemInfo"
      :settings="systemInfo.settings"
      :lan-ips="systemInfo.lanIps"
      :effective-record-dir="systemInfo.recordDir"
      @update="updateSettings"
    />

    <PreviewPlayer
      v-if="preview"
      :hls-url="preview.hlsUrl"
      :webrtc-url="preview.webrtcUrl"
      :title="preview.filename"
      @close="preview = null"
    />

    <QrCodeModal
      v-if="qrStream"
      :url="qrStream.publicHlsUrl || qrStream.rtspUrl"
      :title="qrStream.streamName"
      @close="qrStream = null"
    />

    <CaptureSourceModal
      v-if="captureMode"
      :mode="captureMode"
      @close="captureMode = null"
      @confirm="onCaptureConfirm"
    />
  </div>
</template>
