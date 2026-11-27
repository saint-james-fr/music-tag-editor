<script setup lang="ts">
import { ref, watch } from 'vue'
import { usePrefs } from '../prefs'
import type { ColumnId } from '../types'
import { COLUMN_META } from '../types'

defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()

const { prefs, listForColumn } = usePrefs()
const listKey = ref<ColumnId>('genre')
const listText = ref(listForColumn('genre').join('\n'))

watch(listKey, (k) => {
  listText.value = listForColumn(k).join('\n')
})

const syncListFromText = () => {
  const items = listText.value
    .split('\n')
    .map((s) => s.trim())
    .filter(Boolean)
  const unique = [...new Set(items)].slice(0, 512)
  prefs.value.columnLists[listKey.value] = unique
  listText.value = unique.join('\n')
}
</script>

<template>
  <div v-if="open" class="backdrop scrim" @click.self="emit('close')">
    <div class="modal material-sheet" role="dialog" aria-label="Preferences">
      <header>Preferences</header>
      <section>
        <h3>General</h3>
        <label class="row">
          <input v-model="prefs.padTrackNumber" type="checkbox" />
          Pad track number in rename (01, 02…)
        </label>
        <label class="row">
          <input v-model="prefs.updateTracksTotalOnEdit" type="checkbox" />
          Suggest tracks total when editing track #
        </label>
        <label class="row">
          <input v-model="prefs.confirmBeforeRename" type="checkbox" />
          Confirm before applying rename
        </label>
      </section>
      <section>
        <h3>Default rename pattern</h3>
        <input v-model="prefs.renamePattern" type="text" spellcheck="false" class="wide" />
      </section>
      <section>
        <h3>Column dropdown list</h3>
        <select v-model="listKey">
          <option v-for="c in COLUMN_META.filter((x) => x.list)" :key="c.key" :value="c.key">
            {{ c.label }}
          </option>
        </select>
        <textarea v-model="listText" rows="6" @blur="syncListFromText" />
        <p class="hint">One item per line (max 512). Used for Genre suggestions.</p>
      </section>
      <footer>
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
  width: min(480px, 94vw);
  max-height: 90vh;
  overflow: auto;
  border-radius: 12px;
  padding: 16px;
}
header {
  font-weight: 600;
  font-size: 16px;
  margin-bottom: 12px;
}
section {
  margin-bottom: 16px;
}
h3 {
  margin: 0 0 8px;
  font-size: 13px;
}
.row {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 6px;
}
.wide {
  width: 100%;
}
textarea {
  width: 100%;
  margin-top: 8px;
  background: #1f1f1f;
  border: 1px solid #555;
  color: inherit;
  border-radius: 4px;
  padding: 8px;
}
select,
input[type='text'] {
  padding: 6px 8px;
  border: 1px solid #555;
  border-radius: 4px;
  background: #1f1f1f;
  color: inherit;
}
.hint {
  font-size: 11px;
  opacity: 0.7;
}
footer {
  display: flex;
  justify-content: flex-end;
}
button {
  padding: 6px 12px;
  border: 1px solid #555;
  border-radius: 6px;
  background: #444;
  color: inherit;
}
</style>