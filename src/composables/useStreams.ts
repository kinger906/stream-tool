import { onMounted, onUnmounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { api } from "../api/tauri";
import type { StreamTaskInfo, SystemInfo } from "../types";

export function useStreams() {
  const streams = ref<StreamTaskInfo[]>([]);
  const systemInfo = ref<SystemInfo | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  let unlisten: (() => void) | null = null;
  let timer: ReturnType<typeof setInterval> | null = null;

  async function refresh() {
    try {
      const [list, info] = await Promise.all([
        api.listStreams(),
        api.getSystemInfo(),
      ]);
      streams.value = list;
      systemInfo.value = info;
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
    await api.updateStream(id, enabled, undefined);
    await refresh();
  }

  async function toggleCopyMode(id: string, copyMode: boolean) {
    await api.updateStream(id, undefined, copyMode);
    await refresh();
  }

  async function updateMaxConcurrent(value: number) {
    const settings = await api.updateSettings(value);
    if (systemInfo.value) {
      systemInfo.value.settings = settings;
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
    loading,
    error,
    refresh,
    addPaths,
    start,
    stop,
    startAll,
    stopAll,
    remove,
    toggleLoop,
    toggleCopyMode,
    updateMaxConcurrent,
  };
}
