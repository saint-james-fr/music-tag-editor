<script setup lang="ts">
import { usePrefs } from '../prefs'

defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()

const { prefs } = usePrefs()
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
input[type='text'] {
  padding: 6px 8px;
  border: 1px solid #555;
  border-radius: 4px;
  background: #1f1f1f;
  color: inherit;
}
footer {
  display: flex;
  justify-content: flex-end;
}
button {
  padding: 6px 14px;
  border: 1px solid #555;
  border-radius: 6px;
  background: #444;
  color: inherit;
  cursor: pointer;
}
</style>
