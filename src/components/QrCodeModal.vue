<script setup lang="ts">
import { onMounted, ref } from "vue";
import QRCode from "qrcode";

const props = defineProps<{ url: string; title: string }>();
const emit = defineEmits<{ close: [] }>();

const canvasRef = ref<HTMLCanvasElement | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  if (!canvasRef.value) return;
  try {
    await QRCode.toCanvas(canvasRef.value, props.url, {
      width: 240,
      margin: 2,
    });
  } catch (e) {
    error.value = String(e);
  }
});
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="panel">
      <div class="panel-head">
        <h3>扫码播放 — {{ title }}</h3>
        <button class="btn" type="button" @click="emit('close')">关闭</button>
      </div>
      <div class="body">
        <canvas ref="canvasRef" />
        <p v-if="error" class="error">{{ error }}</p>
        <p class="url">{{ url }}</p>
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
  z-index: 110;
}
.panel {
  width: min(360px, 92vw);
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
  font-size: 14px;
}
.body {
  padding: 20px 16px;
  text-align: center;
}
.error {
  color: #ff7875;
  font-size: 12px;
}
.url {
  margin: 12px 0 0;
  font-size: 11px;
  color: var(--text-muted);
  word-break: break-all;
}
</style>
