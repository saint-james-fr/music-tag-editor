<script setup lang="ts">
export type LogLine = { t: string; msg: string; level?: 'info' | 'error' }

const props = defineProps<{ open: boolean; lines: LogLine[] }>()
const emit = defineEmits<{ close: []; clear: [] }>()

const copyAll = async () => {
  const text = props.lines.map((l) => `${l.t}  ${l.msg}`).join('\n')
  await navigator.clipboard.writeText(text)
}
</script>

<template>
  <div v-if="open" class="backdrop scrim" @click.self="emit('close')">
    <div class="modal material-sheet" role="dialog" aria-label="App log">
      <header>App log</header>
      <pre v-if="lines.length" class="log"><span
          v-for="(line, i) in lines"
          :key="i"
          class="line"
          :class="line.level"
        >{{ line.t }}  {{ line.msg }}
</span></pre>
      <p v-else class="empty">No entries yet. Status messages and errors show up here.</p>
      <footer>
        <button type="button" :disabled="!lines.length" @click="copyAll">Copy</button>
        <button type="button" :disabled="!lines.length" @click="emit('clear')">Clear</button>
        <button type="button" @click="emit('close')">Close</button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  z-index: 60;
  display: flex;
  align-items: center;
  justify-content: center;
}
.modal {
  width: min(640px, 94vw);
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  border-radius: 12px;
  padding: 16px;
}
header {
  font-weight: 600;
  font-size: 16px;
  margin-bottom: 12px;
  flex-shrink: 0;
}
.log {
  flex: 1;
  min-height: 200px;
  max-height: 56vh;
  margin: 0 0 12px;
  padding: 10px 12px;
  overflow: auto;
  border: 1px solid #555;
  border-radius: 6px;
  background: #1a1a1a;
  font: 11px/1.45 ui-monospace, SFMono-Regular, Menlo, monospace;
  white-space: pre-wrap;
  word-break: break-word;
  user-select: text;
  -webkit-user-select: text;
  cursor: text;
}
.line.error {
  color: #ff8a8a;
}
.empty {
  margin: 0 0 12px;
  color: #888;
  font-size: 12px;
}
footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  flex-shrink: 0;
}
button {
  padding: 6px 14px;
  border: 1px solid #555;
  border-radius: 6px;
  background: #444;
  color: inherit;
  cursor: pointer;
}
button:disabled {
  opacity: 0.45;
  cursor: default;
}
</style>
