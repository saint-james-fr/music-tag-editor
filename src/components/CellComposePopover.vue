<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { ColumnId, ColumnKey, TrackRow } from '../types'
import { COLUMN_META } from '../types'

const props = defineProps<{
  col: ColumnId
  rows: TrackRow[]
  pattern: string
}>()

const emit = defineEmits<{
  close: []
  applyField: [value: string]
  applyPattern: [pattern: string]
  previewPattern: [pattern: string]
}>()

const label = computed(() => COLUMN_META.find((c) => c.key === props.col)?.label ?? props.col)
const isFileName = computed(() => props.col === 'fileName')
const isMultiline = computed(() => props.col === 'comment')

const fieldValue = ref('')
const localPattern = ref(props.pattern)

watch(
  () => [props.col, props.rows, props.pattern] as const,
  () => {
    localPattern.value = props.pattern
    if (!isFileName.value && props.rows.length === 1) {
      fieldValue.value = props.rows[0][props.col as ColumnKey] ?? ''
    } else {
      fieldValue.value = ''
    }
  },
  { immediate: true },
)

const title = computed(() =>
  props.rows.length === 1
    ? `Compose ${label.value}`
    : `Compose ${label.value} (${props.rows.length} files)`,
)
</script>

<template>
  <div class="backdrop scrim" @click.self="emit('close')">
    <div class="popover material-sheet" role="dialog" :aria-label="title">
      <header>{{ title }}</header>

      <template v-if="isFileName">
        <p class="hint">
          Tokens: &#123;&#123;tracknumber&#125;&#125;, &#123;&#123;artist&#125;&#125;, &#123;&#123;title&#125;&#125;,
          &#123;&#123;album&#125;&#125;, &#123;&#123;genre&#125;&#125;, &#123;&#123;year&#125;&#125;
        </p>
        <input v-model="localPattern" type="text" class="field" spellcheck="false" />
        <div class="actions">
          <button type="button" @click="emit('previewPattern', localPattern)">Preview</button>
          <button type="button" class="primary" @click="emit('applyPattern', localPattern)">Apply</button>
          <button type="button" @click="emit('close')">Done</button>
        </div>
      </template>

      <template v-else>
        <p class="hint">Value for all selected rows</p>
        <textarea v-if="isMultiline" v-model="fieldValue" class="field area" rows="5" />
        <input v-else v-model="fieldValue" type="text" class="field" />
        <div class="actions">
          <button type="button" class="primary" @click="emit('applyField', fieldValue)">Apply</button>
          <button type="button" @click="emit('close')">Done</button>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  z-index: 55;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 120px;
}
.popover {
  width: min(440px, 92vw);
  padding: 14px 16px;
  border-radius: 12px;
}
header {
  font-weight: 600;
  margin-bottom: 10px;
}
.hint {
  margin: 0 0 8px;
  font-size: 11px;
  opacity: 0.75;
}
.field {
  width: 100%;
  padding: 8px 10px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 6px;
  background: rgba(0, 0, 0, 0.35);
  color: inherit;
  font-size: 13px;
}
.field.area {
  resize: vertical;
  font-family: inherit;
}
.actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 12px;
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
  background: #3d6ea8;
}
</style>
