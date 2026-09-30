<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import Hls from "hls.js";
import type { PreviewMode } from "../types";

const props = defineProps<{
  hlsUrl: string;
  webrtcUrl: string;
  title: string;
}>();
const emit = defineEmits<{ close: [] }>();

const mode = ref<PreviewMode>("hls");
const videoRef = ref<HTMLVideoElement | null>(null);
const playerError = ref<string | null>(null);
const waitingHint = ref<string | null>(null);
let hls: Hls | null = null;
let manifestRetryTimer: ReturnType<typeof setInterval> | null = null;
let manifestRetryLeft = 0;

const activeUrl = computed(() =>
  mode.value === "hls" ? props.hlsUrl : props.webrtcUrl,
);

function clearManifestRetry() {
  if (manifestRetryTimer) {
    clearInterval(manifestRetryTimer);
    manifestRetryTimer = null;
  }
  waitingHint.value = null;
}

function destroy() {
  clearManifestRetry();
  hls?.destroy();
  hls = null;
}

function scheduleManifestRetry() {
  if (manifestRetryTimer || manifestRetryLeft <= 0) return;
  waitingHint.value = `HLS 清单尚未就绪，正在重试（剩余 ${manifestRetryLeft} 次）…`;
  manifestRetryTimer = setInterval(() => {
    if (manifestRetryLeft <= 0) {
      clearManifestRetry();
      return;
    }
    manifestRetryLeft -= 1;
    waitingHint.value = `HLS 清单尚未就绪，正在重试（剩余 ${manifestRetryLeft} 次）…`;
    hls?.startLoad();
  }, 2000);
}

async function setupHls(url: string) {
  destroy();
  playerError.value = null;
  manifestRetryLeft = 8;
  await nextTick();

  const video = videoRef.value;
  if (!video || !url) return;

  if (Hls.isSupported()) {
    hls = new Hls({
      enableWorker: true,
      lowLatencyMode: false,
      xhrSetup(xhr) {
        xhr.withCredentials = false;
      },
    });

    hls.on(Hls.Events.ERROR, (_event, data) => {
      if (!data.fatal) return;
      const detail = data.details ?? data.type;
      if (
        detail === "manifestLoadError" ||
        detail === "manifestParsingError" ||
        data.type === Hls.ErrorTypes.NETWORK_ERROR
      ) {
        if (manifestRetryLeft > 0) {
          scheduleManifestRetry();
          return;
        }
      }
      clearManifestRetry();
      playerError.value = `HLS 加载失败：${detail}。请确认任务为「推流中」，且路径与列表中的 HLS 地址一致。`;
    });

    hls.on(Hls.Events.MANIFEST_PARSED, () => {
      clearManifestRetry();
      video.play().catch(() => {
        playerError.value = "自动播放被阻止，请点击播放按钮";
      });
    });

    hls.loadSource(url);
    hls.attachMedia(video);
    scheduleManifestRetry();
    return;
  }

  if (video.canPlayType("application/vnd.apple.mpegurl")) {
    video.src = url;
    video.play().catch(() => {
      playerError.value = "自动播放被阻止，请点击播放按钮";
    });
    return;
  }

  playerError.value = "当前环境不支持 HLS 播放";
}

watch(mode, (m) => {
  if (m === "hls") {
    setupHls(props.hlsUrl);
  } else {
    destroy();
    playerError.value = null;
  }
});

watch(
  () => props.hlsUrl,
  (url) => {
    if (mode.value === "hls") {
      setupHls(url);
    }
  },
);

onMounted(() => {
  if (mode.value === "hls") {
    setupHls(props.hlsUrl);
  }
});

onBeforeUnmount(destroy);
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="panel">
      <div class="panel-head">
        <h3>{{ title }}</h3>
        <div class="mode-tabs">
          <button
            class="btn sm"
            :class="{ active: mode === 'hls' }"
            type="button"
            @click="mode = 'hls'"
          >
            HLS
          </button>
          <button
            class="btn sm"
            :class="{ active: mode === 'webrtc' }"
            type="button"
            @click="mode = 'webrtc'"
          >
            WebRTC
          </button>
        </div>
        <button class="btn" type="button" @click="emit('close')">关闭</button>
      </div>

      <template v-if="mode === 'hls'">
        <video ref="videoRef" controls autoplay muted playsinline class="video" />
        <p v-if="waitingHint" class="hint">{{ waitingHint }}</p>
        <p v-if="playerError" class="error">{{ playerError }}</p>
      </template>
      <template v-else>
        <iframe
          :src="webrtcUrl"
          class="webrtc-frame"
          allow="autoplay; camera; microphone; fullscreen"
          title="WebRTC preview"
        />
        <p class="hint">WebRTC 低延迟预览（MediaMTX 内置播放器）</p>
      </template>

      <p class="url">{{ activeUrl }}</p>
      <p class="hint foot">
        预览使用与列表相同的局域网地址；本机访问 127.0.0.1 与局域网 IP 等价，失败通常表示尚未推流成功。
      </p>
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
  width: min(860px, 92vw);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 12px;
  overflow: hidden;
}
.panel-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
}
.panel-head h3 {
  margin: 0;
  font-size: 15px;
  flex: 1;
}
.mode-tabs {
  display: flex;
  gap: 6px;
}
.mode-tabs .active {
  border-color: var(--accent);
  color: #69b1ff;
}
.video,
.webrtc-frame {
  width: 100%;
  aspect-ratio: 16 / 9;
  background: #000;
  border: none;
}
.error,
.hint {
  margin: 0;
  padding: 8px 16px 0;
  font-size: 12px;
}
.hint.foot {
  padding-bottom: 12px;
}
.error {
  color: #ff7875;
}
.hint {
  color: var(--text-muted);
}
.url {
  margin: 0;
  padding: 10px 16px;
  font-size: 12px;
  color: var(--text-muted);
  word-break: break-all;
}
</style>
