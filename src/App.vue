<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import AppHeader from "./components/AppHeader.vue";
import CaptureSourceModal from "./components/CaptureSourceModal.vue";
import DependencyBanner from "./components/DependencyBanner.vue";
import DropZone from "./components/DropZone.vue";
import PreviewPlayer from "./components/PreviewPlayer.vue";
import SettingsPanel from "./components/SettingsPanel.vue";
import StreamList from "./components/StreamList.vue";
import { useStreams } from "./composables/useStreams";
import type { StreamTaskInfo } from "./types";

const {
  streams,
  systemInfo,
  error,
  addPaths,
  addCamera,
  addDisplay,
  start,
  stop,
  startAll,
  stopAll,
  remove,
  toggleLoop,
  toggleCopyMode,
  updateMaxConcurrent,
} = useStreams();

const preview = ref<StreamTaskInfo | null>(null);
const captureMode = ref<"camera" | "display" | null>(null);
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

function onCaptureConfirm(
  videoDevice: string | null,
  audioDevice: string | null,
) {
  if (captureMode.value === "camera" && videoDevice) {
    addCamera(videoDevice, audioDevice);
  } else if (captureMode.value === "display") {
    addDisplay(audioDevice);
  }
  captureMode.value = null;
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

    <DropZone
      @add="addPaths"
      @add-camera="captureMode = 'camera'"
      @add-display="captureMode = 'display'"
    />

    <StreamList
      :streams="streams"
      @start="start"
      @stop="stop"
      @remove="remove"
      @preview="preview = $event"
      @toggle-loop="toggleLoop"
      @toggle-copy="toggleCopyMode"
    />

    <SettingsPanel
      v-if="systemInfo"
      :max-concurrent="systemInfo.settings.maxConcurrent"
      @update="updateMaxConcurrent"
    />

    <PreviewPlayer
      v-if="preview"
      :url="preview.hlsUrlLocal"
      :title="preview.filename"
      @close="preview = null"
    />

    <CaptureSourceModal
      v-if="captureMode"
      :mode="captureMode"
      @close="captureMode = null"
      @confirm="onCaptureConfirm"
    />
  </div>
</template>
