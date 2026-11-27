<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import type { ArtworkChange, ArtworkPayload, TrackRow } from '../types'

const props = defineProps<{
  rows: TrackRow[]
  preview: ArtworkPayload | null
}>()

const emit = defineEmits<{
  close: []
  apply: [changes: ArtworkChange[]]
  export: []
}>()

const loading = ref(false)
const localPreview = ref<ArtworkPayload | null>(props.preview)

watch(
  () => props.preview,
  (p) => {
    localPreview.value = p
  },
)

const title = computed(() =>
  props.rows.length === 1 ? props.rows[0].fileName : `${props.rows.length} files`,
)

const pickReplace = async () => {
  const picked = await open({
    multiple: false,
    title: 'Replace artwork (image or audio file)',
  })
  if (!picked || typeof picked !== 'string') return
  loading.value = true
  try {
    localPreview.value = await invoke<ArtworkPayload>('load_artwork_file', {
      sourcePath: picked,
    })
  } finally {
    loading.value = false
  }
}

const confirm = () => {
  if (!localPreview.value) {
    emit('close')
    return
  }
  const changes: ArtworkChange[] = props.rows.map((r) => ({
    path: r.path,
    action: 'set',
    dataBase64: localPreview.value!.dataBase64,
    mime: localPreview.value!.mime,
  }))
  emit('apply', changes)
  emit('close')
}

const remove = () => {
  emit(
    'apply',
    props.rows.map((r) => ({ path: r.path, action: 'remove' as const })),
  )
  localPreview.value = null
  emit('close')
}
</script>

<template>
  <div class="backdrop scrim" @click.self="emit('close')">
    <div class="popover material-sheet" role="dialog" aria-label="Artwork">
      <header>{{ title }}</header>
      <div class="preview" @dblclick="pickReplace">
        <img v-if="localPreview" :src="`data:${localPreview.mime};base64,${localPreview.dataBase64}`" alt="" />
        <span v-else class="empty">No artwork</span>
      </div>
      <div class="actions">
        <button type="button" :disabled="loading" @click="pickReplace">Replace</button>
        <button type="button" @click="remove">Delete</button>
        <button type="button" @click="emit('export')">Export</button>
        <button type="button" class="primary" @click="confirm">Done</button>
      </div>
      <p class="hint">Changes apply on Save.</p>
    </div>
  </div>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 50;
}
.popover {
  border-radius: 12px;
  padding: 14px;
  width: min(360px, 92vw);
}
header {
  font-weight: 600;
  margin-bottom: 10px;
}
.preview {
  height: 220px;
  background: #222;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  overflow: hidden;
  cursor: pointer;
}
.preview img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}
.empty {
  opacity: 0.5;
}
.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
}
button {
  padding: 6px 10px;
  border: 1px solid #555;
  border-radius: 6px;
  background: #444;
  color: inherit;
  cursor: pointer;
}
button.primary {
  background: #3d6ea8;
  margin-left: auto;
}
.hint {
  margin: 10px 0 0;
  font-size: 11px;
  opacity: 0.7;
}
</style>