<script setup lang="ts">
import { ref, watch } from 'vue'
import { usePrefs } from '../prefs'
import type { ColumnId } from '../types'
import { COLUMN_META } from '../types'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()

const { prefs, activeLayout, setLayoutActive, saveLayoutAs, updateActiveLayout, deleteLayout } = usePrefs()

const draft = ref(activeLayout.value.columns.map((c) => ({ ...c })))
const newName = ref('')

watch(
  () => props.open,
  (o) => {
    if (o) draft.value = activeLayout.value.columns.map((c) => ({ ...c }))
  },
)

const label = (key: ColumnId) => COLUMN_META.find((c) => c.key === key)?.label ?? key

const move = (index: number, dir: -1 | 1) => {
  const next = index + dir
  if (next < 0 || next >= draft.value.length) return
  const cols = [...draft.value]
  ;[cols[index], cols[next]] = [cols[next], cols[index]]
  draft.value = cols
}

const apply = () => {
  updateActiveLayout(draft.value)
  emit('close')
}

const saveAs = () => {
  const name = newName.value.trim() || `Layout ${prefs.value.layouts.length + 1}`
  saveLayoutAs(name, draft.value)
  newName.value = ''
  emit('close')
}

const onLayoutSwitch = (id: string) => {
  setLayoutActive(id)
  draft.value = activeLayout.value.columns.map((c) => ({ ...c }))
}
</script>

<template>
  <div v-if="open" class="backdrop scrim" @click.self="emit('close')">
    <div class="modal material-sheet">
      <header>Column layout</header>
      <label>
        Active layout
        <select
          :value="prefs.activeLayoutId"
          @change="onLayoutSwitch(($event.target as HTMLSelectElement).value)"
        >
          <option v-for="l in prefs.layouts" :key="l.id" :value="l.id">{{ l.name }}</option>
        </select>
      </label>
      <ul class="cols">
        <li v-for="(col, i) in draft" :key="col.key">
          <label>
            <input v-model="col.visible" type="checkbox" />
            {{ label(col.key) }}
          </label>
          <input v-model="col.width" type="text" class="width" title="grid width e.g. 64px or minmax(120px,1fr)" />
          <button type="button" :disabled="i === 0" @click="move(i, -1)">↑</button>
          <button type="button" :disabled="i === draft.length - 1" @click="move(i, 1)">↓</button>
        </li>
      </ul>
      <div class="save-as">
        <input v-model="newName" placeholder="Save as name…" />
        <button type="button" @click="saveAs">Save as</button>
        <button
          type="button"
          :disabled="prefs.layouts.length <= 1"
          @click="deleteLayout(prefs.activeLayoutId)"
        >
          Delete layout
        </button>
      </div>
      <footer>
        <button type="button" @click="emit('close')">Cancel</button>
        <button type="button" class="primary" @click="apply">Apply</button>
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
  width: min(520px, 94vw);
  max-height: 90vh;
  overflow: auto;
  border-radius: 12px;
  padding: 16px;
}
header {
  font-weight: 600;
  margin-bottom: 12px;
}
.cols {
  list-style: none;
  padding: 0;
  margin: 12px 0;
}
.cols li {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 6px;
}
.width {
  width: 140px;
  background: #1f1f1f;
  border: 1px solid #555;
  color: inherit;
  padding: 4px 6px;
}
.save-as {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 12px;
}
footer {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}
button {
  padding: 6px 12px;
  border: 1px solid #555;
  border-radius: 6px;
  background: #444;
  color: inherit;
}
button.primary {
  background: #3d6ea8;
}
select,
input {
  padding: 6px 8px;
  border: 1px solid #555;
  border-radius: 4px;
  background: #1f1f1f;
  color: inherit;
}
</style>