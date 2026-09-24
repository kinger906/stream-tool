<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{ maxConcurrent: number }>();
const emit = defineEmits<{ update: [value: number] }>();

const localMax = ref(props.maxConcurrent);

watch(
  () => props.maxConcurrent,
  (v) => {
    localMax.value = v;
  },
);

function apply() {
  emit("update", localMax.value);
}
</script>

<template>
  <section class="settings">
    <h2>设置</h2>
    <div class="row">
      <label for="max-concurrent">最大并发推流数</label>
      <input
        id="max-concurrent"
        v-model.number="localMax"
        type="number"
        min="1"
        max="32"
      />
      <button class="btn sm" @click="apply">应用</button>
    </div>
    <p class="note">
      局域网播放请在防火墙中放行 TCP/UDP 8554（RTSP）与 TCP 8888（HLS）。
      VLC 打开 RTSP 地址即可实时观看。
    </p>
  </section>
</template>

<style scoped>
.settings {
  margin-top: 20px;
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--surface);
}
.settings h2 {
  margin: 0 0 12px;
  font-size: 14px;
}
.row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.row input {
  width: 72px;
}
.note {
  margin: 12px 0 0;
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.6;
}
</style>
