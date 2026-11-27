<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'

export type MenuItem = {
  id: string
  label: string
  disabled?: boolean
}

defineProps<{
  items: MenuItem[]
  open: boolean
  top: number
  left: number
}>()

const emit = defineEmits<{
  pick: [id: string]
  close: []
}>()

const onDoc = (e: MouseEvent) => {
  const t = e.target as HTMLElement
  if (t.closest?.('.menu-panel') || t.closest?.('.tb-menu')) return
  emit('close')
}

onMounted(() => document.addEventListener('mousedown', onDoc, true))
onUnmounted(() => document.removeEventListener('mousedown', onDoc, true))
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="menu-panel"
      role="menu"
      :style="{ top: `${top}px`, left: `${left}px` }"
    >
      <button
        v-for="item in items"
        :key="item.id"
        type="button"
        class="menu-item"
        role="menuitem"
        :disabled="item.disabled"
        @click="emit('pick', item.id); emit('close')"
      >
        {{ item.label }}
      </button>
    </div>
  </Teleport>
</template>

<style scoped>
.menu-panel {
  position: fixed;
  z-index: 1000;
  min-width: 220px;
  padding: 4px 0;
  background: rgba(58, 58, 58, 0.98);
  border: 1px solid rgba(0, 0, 0, 0.65);
  border-radius: 6px;
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.45),
    inset 0 1px 0 rgba(255, 255, 255, 0.08);
}
.menu-item {
  display: block;
  width: 100%;
  padding: 4px 22px;
  border: none;
  background: transparent;
  color: #eee;
  font-size: 13px;
  text-align: left;
  cursor: default;
}
.menu-item:hover:not(:disabled) {
  background: #3d7fd6;
  color: #fff;
}
.menu-item:disabled {
  opacity: 0.45;
}
</style>