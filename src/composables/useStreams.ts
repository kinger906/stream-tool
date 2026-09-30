import { onMounted, onUnmounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { api } from "../api/tauri";
import type {
  CaptureRegion,
  QualityPreset,
  ScenePreset,
  SettingsUpdate,
  StreamTaskInfo,
  SystemInfo,
} from "../types";

export function useStreams() {
  const streams = ref<StreamTaskInfo[]>([]);
  const systemInfo = ref<SystemInfo | null>(null);
  const scenes = ref<ScenePreset[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  let unlisten: (() => void) | null = null;
  let timer: ReturnType<typeof setInterval> | null = null;

  async function refreshScenes() {
    try {
      scenes.value = await api.listScenes();
    } catch {
      /* ignore */
    }
  }

  async function refresh() {
    try {
      const [list, info] = await Promise.all([
        api.listStreams(),
        api.getSystemInfo(),
      ]);
      streams.value = list;
      systemInfo.value = info;
      await refreshScenes();
    } catch (e) {
      error.value = String(e);
    }
  }

  async function addPaths(paths: string[]) {
    if (!paths.length) return;
    loading.value = true;
    error.value = null;
    try {
      await api.addStreams(paths);
      await refresh();
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function addCamera(videoDevice: string, audioDevice: string | null) {
    error.value = null;
    try {
      await api.addCameraStream(videoDevice, audioDevice ?? undefined);
      await refresh();
    } catch (e) {
      error.value = String(e);
    }
  }

  async function addDisplay(audioDevice: string | null) {
    error.value = null;
    try {
      await api.addDisplayStream(audioDevice ?? undefined);
      await refresh();
    } catch (e) {
      error.value = String(e);
    }
  }

  async function addWindow(windowTitle: string, audioDevice: string | null) {
    error.value = null;
    try {
      await api.addWindowStream(windowTitle, audioDevice ?? undefined);
      await refresh();
    } catch (e) {
      error.value = String(e);
    }
  }

  async function addRegion(region: CaptureRegion, audioDevice: string | null) {
    error.value = null;
    try {
      await api.addRegionStream(region, audioDevice ?? undefined);
      await refresh();
    } catch (e) {
      error.value = String(e);
    }
  }

  async function start(id: string) {
    error.value = null;
    try {
      await api.startStream(id);
      await refresh();
    } catch (e) {
      error.value = String(e);
      await refresh();
    }
  }

  async function stop(id: string) {
    await api.stopStream(id);
    await refresh();
  }

  async function startAll() {
    error.value = null;
    try {
      await api.startAllStreams();
      await refresh();
    } catch (e) {
      error.value = String(e);
      await refresh();
    }
  }

  async function stopAll() {
    await api.stopAllStreams();
    await refresh();
  }

  async function remove(id: string) {
    await api.removeStream(id);
    await refresh();
  }

  async function toggleLoop(id: string, enabled: boolean) {
    await api.updateStream(id, { loopEnabled: enabled });
    await refresh();
  }

  async function toggleCopyMode(id: string, copyMode: boolean) {
    await api.updateStream(id, { copyMode });
    await refresh();
  }

  async function updateStreamName(id: string, streamName: string) {
    error.value = null;
    try {
      await api.updateStream(id, { streamName });
      await refresh();
    } catch (e) {
      error.value = String(e);
    }
  }

  async function toggleRecord(id: string, enabled: boolean) {
    await api.updateStream(id, { recordEnabled: enabled });
    await refresh();
  }

  async function toggleReconnect(id: string, enabled: boolean) {
    await api.updateStream(id, { autoReconnect: enabled });
    await refresh();
  }

  async function updateQuality(id: string, qualityPreset: QualityPreset) {
    await api.updateStream(id, { qualityPreset });
    await refresh();
  }

  async function updateRtmpUrl(id: string, rtmpUrl: string) {
    error.value = null;
    try {
      await api.updateStream(id, { rtmpUrl });
      await refresh();
    } catch (e) {
      error.value = String(e);
    }
  }

  async function updateSettings(update: SettingsUpdate) {
    error.value = null;
    try {
      const settings = await api.updateSettings(update);
      if (systemInfo.value) {
        systemInfo.value.settings = settings;
      }
      await refresh();
    } catch (e) {
      error.value = String(e);
    }
  }

  async function saveScene(name: string) {
    error.value = null;
    try {
      await api.saveScene(name);
      await refreshScenes();
    } catch (e) {
      error.value = String(e);
    }
  }

  async function deleteScene(id: string) {
    await api.deleteScene(id);
    await refreshScenes();
  }

  async function applyScene(id: string, replace = true) {
    error.value = null;
    try {
      await api.applyScene(id, replace);
      await refresh();
    } catch (e) {
      error.value = String(e);
    }
  }

  onMounted(async () => {
    await refresh();
    unlisten = await listen("streams-changed", () => {
      refresh();
    });
    timer = setInterval(refresh, 3000);
  });

  onUnmounted(() => {
    if (timer) clearInterval(timer);
    unlisten?.();
  });

  return {
    streams,
    systemInfo,
    scenes,
    loading,
    error,
    refresh,
    addPaths,
    addCamera,
    addDisplay,
    addWindow,
    addRegion,
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
  };
}
