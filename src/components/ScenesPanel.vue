<script setup lang="ts">
import { ref } from "vue";
import type { ScenePreset } from "../types";

defineProps<{ scenes: ScenePreset[] }>();

const emit = defineEmits<{
  save: [name: string];
  apply: [id: string];
  remove: [id: string];
}>();

const name = ref("");

function onSave() {
  const n = name.value.trim();
  if (!n) return;
  emit("save", n);
  name.value = "";
}
</script>

<template>
  <section class="scenes">
    <h2>场景预设</h2>
    <div class="row">
      <input
        v-model="name"
        type="text"
        placeholder="保存当前任务列表为场景…"
        @keyup.enter="onSave"
      />
      <button class="btn sm primary" type="button" @click="onSave">保存场景</button>
    </div>

    <ul v-if="scenes.length" class="list">
      <li v-for="s in scenes" :key="s.id">
        <div>
          <strong>{{ s.name }}</strong>
          <span class="meta">{{ s.streams.length }} 路</span>
        </div>
        <div class="actions">
          <button class="btn sm" type="button" @click="emit('apply', s.id)">
            加载并替换
          </button>
          <button class="btn sm danger" type="button" @click="emit('remove', s.id)">
            删除
          </button>
        </div>
      </li>
    </ul>
    <p v-else class="empty">暂无场景。配置好几路推流后可一键保存。</p>
  </section>
</template>

<style scoped>
.scenes {
  margin-top: 16px;
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--surface);
}
.scenes h2 {
  margin: 0 0 12px;
  font-size: 14px;
}
.row {
  display: flex;
  gap: 8px;
}
.row input {
  flex: 1;
  background: var(--surface-2);
  border: 1px solid var(--border);
  color: inherit;
  border-radius: 6px;
  padding: 8px;
}
.list {
  list-style: none;
  margin: 12px 0 0;
  padding: 0;
}
.list li {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  padding: 10px 0;
  border-top: 1px solid var(--border);
}
.meta {
  margin-left: 8px;
  font-size: 12px;
  color: var(--text-muted);
}
.actions {
  display: flex;
  gap: 6px;
}
.empty {
  margin: 12px 0 0;
  font-size: 12px;
  color: var(--text-muted);
}
</style>
