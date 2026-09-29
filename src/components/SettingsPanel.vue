<script setup lang="ts">
import { ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import type { AppSettings } from "../types";

const props = defineProps<{
  settings: AppSettings;
  lanIps: string[];
  effectiveRecordDir: string;
}>();

const emit = defineEmits<{
  update: [
    payload: {
      maxConcurrent?: number;
      selectedLanIp?: string;
      rtspUsername?: string;
      rtspPassword?: string;
      recordDir?: string;
    },
  ];
}>();

const localMax = ref(props.settings.maxConcurrent);
const selectedLanIp = ref(props.settings.selectedLanIp ?? "");
const rtspUsername = ref(props.settings.rtspUsername ?? "");
const rtspPassword = ref(props.settings.rtspPassword ?? "");
const recordDir = ref(props.settings.recordDir ?? "");

watch(
  () => props.settings,
  (s) => {
    localMax.value = s.maxConcurrent;
    selectedLanIp.value = s.selectedLanIp ?? "";
    rtspUsername.value = s.rtspUsername ?? "";
    rtspPassword.value = s.rtspPassword ?? "";
    recordDir.value = s.recordDir ?? "";
  },
  { deep: true },
);

async function pickRecordDir() {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") {
    recordDir.value = selected;
  }
}

function apply() {
  emit("update", {
    maxConcurrent: localMax.value,
    selectedLanIp: selectedLanIp.value,
    rtspUsername: rtspUsername.value,
    rtspPassword: rtspPassword.value,
    recordDir: recordDir.value,
  });
}
</script>

<template>
  <section class="settings">
    <h2>设置</h2>

    <div class="grid">
      <label class="field">
        <span>最大并发推流数</span>
        <input v-model.number="localMax" type="number" min="1" max="32" />
      </label>

      <label class="field">
        <span>局域网 IP（分享地址用）</span>
        <select v-model="selectedLanIp">
          <option value="">自动检测</option>
          <option v-for="ip in lanIps" :key="ip" :value="ip">{{ ip }}</option>
        </select>
      </label>

      <label class="field">
        <span>RTSP/HLS 用户名</span>
        <input v-model="rtspUsername" type="text" placeholder="留空则不启用认证" />
      </label>

      <label class="field">
        <span>RTSP/HLS 密码</span>
        <input v-model="rtspPassword" type="password" placeholder="留空则不启用认证" />
      </label>

      <label class="field wide">
        <span>录制目录</span>
        <div class="dir-row">
          <input v-model="recordDir" type="text" :placeholder="recordDir || '默认应用数据目录'" />
          <button class="btn sm" type="button" @click="pickRecordDir">浏览</button>
        </div>
        <small>当前生效：{{ recordDir || effectiveRecordDir }}</small>
      </label>
    </div>

    <div class="actions">
      <button class="btn sm primary" type="button" @click="apply">应用设置</button>
    </div>

    <p class="note">
      局域网播放请在防火墙中放行 TCP/UDP 8554（RTSP）、TCP 8888（HLS）与 TCP 8889（WebRTC）。
      启用认证后，VLC 需使用带用户名密码的 RTSP 地址。
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
.grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 13px;
}
.field.wide {
  grid-column: 1 / -1;
}
.field input,
.field select {
  background: var(--surface-2);
  border: 1px solid var(--border);
  color: inherit;
  border-radius: 6px;
  padding: 8px;
}
.dir-row {
  display: flex;
  gap: 8px;
}
.dir-row input {
  flex: 1;
}
.field small {
  color: var(--text-muted);
  font-size: 11px;
}
.actions {
  margin-top: 12px;
}
.note {
  margin: 12px 0 0;
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.6;
}
</style>
