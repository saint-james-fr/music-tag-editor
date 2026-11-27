<script setup lang="ts">
const pattern = defineModel<string>('pattern', { required: true })

defineProps<{
  pending: number
}>()

const emit = defineEmits<{
  close: []
  preview: []
  apply: []
}>()
</script>

<template>
  <div class="backdrop scrim" @click.self="emit('close')">
    <div class="pop material-sheet">
      <header>Rename files</header>
      <input v-model="pattern" type="text" spellcheck="false" class="pattern" />
      <div class="actions">
        <button type="button" @click="emit('preview')">Preview</button>
        <button type="button" :disabled="!pending" class="primary" @click="emit('apply')">
          Rename{{ pending ? ` (${pending})` : '' }}
        </button>
        <button type="button" @click="emit('close')">Done</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  z-index: 40;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 100px;
}
.pop {
  border-radius: 12px;
  padding: 14px 16px;
  min-width: 420px;
}
header {
  font-weight: 600;
  margin-bottom: 10px;
}
.pattern {
  width: 100%;
  padding: 8px 10px;
  border: 1px solid #39d353;
  border-radius: 4px;
  background: #1a1a1a;
  color: #eee;
}
.actions {
  display: flex;
  gap: 8px;
  margin-top: 12px;
  justify-content: flex-end;
}
button {
  padding: 6px 12px;
  border: 1px solid #555;
  border-radius: 6px;
  background: #444;
  color: inherit;
  cursor: pointer;
}
button.primary {
  background: #3670b8;
}
button:disabled {
  opacity: 0.45;
}
</style>