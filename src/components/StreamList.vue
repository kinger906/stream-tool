<script setup lang="ts">
import StreamRow from "./StreamRow.vue";
import type { StreamTaskInfo } from "../types";

defineProps<{ streams: StreamTaskInfo[] }>();

defineEmits<{
  start: [id: string];
  stop: [id: string];
  remove: [id: string];
  preview: [stream: StreamTaskInfo];
  toggleLoop: [id: string, enabled: boolean];
  toggleCopy: [id: string, copyMode: boolean];
}>();
</script>

<template>
  <div class="list-wrap">
    <table v-if="streams.length" class="table">
      <thead>
        <tr>
          <th>视频</th>
          <th>状态</th>
          <th>进度</th>
          <th>流地址（局域网）</th>
          <th>选项</th>
          <th>操作</th>
        </tr>
      </thead>
      <tbody>
        <StreamRow
          v-for="s in streams"
          :key="s.id"
          :stream="s"
          @start="$emit('start', s.id)"
          @stop="$emit('stop', s.id)"
          @remove="$emit('remove', s.id)"
          @preview="$emit('preview', s)"
          @toggle-loop="$emit('toggleLoop', s.id, $event)"
          @toggle-copy="$emit('toggleCopy', s.id, $event)"
        />
      </tbody>
    </table>
    <div v-else class="empty">暂无推流任务，请添加视频文件</div>
  </div>
</template>

<style scoped>
.list-wrap {
  margin-top: 16px;
}
.table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}
th,
td {
  padding: 10px 8px;
  border-bottom: 1px solid var(--border);
  text-align: left;
  vertical-align: top;
}
th {
  color: var(--text-muted);
  font-weight: 500;
  font-size: 12px;
}
.empty {
  padding: 40px;
  text-align: center;
  color: var(--text-muted);
  border: 1px dashed var(--border);
  border-radius: 12px;
}
</style>
