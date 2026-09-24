<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import Hls from "hls.js";

const props = defineProps<{ url: string; title: string }>();
const emit = defineEmits<{ close: [] }>();

const videoRef = ref<HTMLVideoElement | null>(null);
const playerError = ref<string | null>(null);
let hls: Hls | null = null;

function destroy() {
  hls?.destroy();
  hls = null;
}

async function setupPlayer(url: string) {
  destroy();
  playerError.value = null;
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
      playerError.value = `HLS 加载失败：${data.details ?? data.type}`;
      if (data.type === Hls.ErrorTypes.NETWORK_ERROR) {
        hls?.startLoad();
      }
    });

    hls.on(Hls.Events.MANIFEST_PARSED, () => {
      video.play().catch(() => {
        playerError.value = "自动播放被阻止，请点击播放按钮";
      });
    });

    hls.loadSource(url);
    hls.attachMedia(video);
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

onMounted(() => {
  setupPlayer(props.url);
});

watch(
  () => props.url,
  (url) => {
    setupPlayer(url);
  },
);

onBeforeUnmount(destroy);
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="panel">
      <div class="panel-head">
        <h3>{{ title }}</h3>
        <button class="btn" @click="emit('close')">关闭</button>
      </div>
      <video ref="videoRef" controls autoplay muted playsinline class="video" />
      <p v-if="playerError" class="error">{{ playerError }}</p>
      <p class="url">{{ url }}</p>
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
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
}
.panel-head h3 {
  margin: 0;
  font-size: 15px;
}
.video {
  width: 100%;
  aspect-ratio: 16 / 9;
  background: #000;
}
.error {
  margin: 0;
  padding: 8px 16px 0;
  font-size: 12px;
  color: #ff7875;
}
.url {
  margin: 0;
  padding: 10px 16px;
  font-size: 12px;
  color: var(--text-muted);
  word-break: break-all;
}
</style>
