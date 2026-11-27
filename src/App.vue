<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import EditorPane from './components/EditorPane.vue'
import LayoutModal from './components/LayoutModal.vue'
import PrefsModal from './components/PrefsModal.vue'
import { DEMO_TRACKS, shouldLoadDemo } from './demo-tracks'
import { useTabs } from './tabs'

const { tabs, activeTabId, activeTab, addTab, closeTab } = useTabs()
const status = ref('Glissez des fichiers audio ou utilisez Open.')
const showPrefs = ref(false)
const showLayout = ref(false)
const tab = computed(() => activeTab())

const onStatus = (msg: string) => {
  status.value = msg
}

onMounted(async () => {
  if (shouldLoadDemo()) {
    const t = tab.value
    t.tracks = DEMO_TRACKS.map((r) => ({ ...r }))
    t.folderLabel = 'MASTERS VINYLS'
    t.title = 'MASTERS VINYLS'
    status.value = ''
    return
  }
  // Cold start empty — re-open folder to re-read tags from disk.
  // (Session restore felt like stale/empty tags.)
  localStorage.removeItem('tag-editor-tabs-v1')
  status.value = 'Open a folder or drop audio files.'
})
</script>

<template>
  <div class="app">
    <div v-if="tabs.length > 1" class="tab-strip material-bar">
      <button
        v-for="t in tabs"
        :key="t.id"
        type="button"
        class="tab"
        :class="{ active: t.id === activeTabId }"
        @click="activeTabId = t.id"
      >
        {{ t.title || 'Untitled' }}
        <span class="x" @click.stop="closeTab(t.id)">×</span>
      </button>
      <button type="button" class="tab-add" title="New tab" @click="addTab">+</button>
    </div>

    <EditorPane
      v-if="tab"
      :key="tab.id"
      :tab="tab"
      @status="onStatus"
      @update:show-layout="showLayout = $event"
      @update:show-prefs="showPrefs = $event"
    />

    <footer class="status-bar">
      <span class="status-msg">{{ status }}</span>
      <span v-if="tab" class="status-count">{{ tab.tracks.length }}*</span>
    </footer>

    <PrefsModal :open="showPrefs" @close="showPrefs = false" />
    <LayoutModal :open="showLayout" @close="showLayout = false" />
  </div>
</template>

<style>
:root {
  font-family:
    -apple-system,
    BlinkMacSystemFont,
    'Helvetica Neue',
    sans-serif;
  font-size: 13px;
  color-scheme: dark;
}
*,
*::before,
*::after {
  box-sizing: border-box;
}
html,
body {
  height: 100%;
  margin: 0;
  overflow: hidden;
  background: var(--te-bg-pane, #2a2a2a);
  color: var(--te-text, #f0f0f0);
}
#app {
  height: 100%;
  overflow: hidden;
}
.app {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  background: var(--te-bg-pane, #2a2a2a);
}
.tab-strip {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 4px 8px 0;
  min-height: 28px;
  z-index: 6;
}
.tab {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px 6px;
  border: 1px solid transparent;
  border-bottom: none;
  border-radius: 6px 6px 0 0;
  background: #323232;
  color: #ccc;
  font-size: 11px;
  cursor: pointer;
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.tab.active {
  background: #2c2c2c;
  color: #fff;
  border-color: #3a3a3a;
}
.tab .x {
  opacity: 0.55;
  font-size: 14px;
  line-height: 1;
}
.tab-add {
  padding: 2px 10px;
  border: none;
  background: transparent;
  color: #888;
  cursor: pointer;
  font-size: 16px;
}
.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--te-status-h, 17px);
  min-height: var(--te-status-h, 17px);
  flex-shrink: 0;
  padding: 0 10px;
  font-size: 11px;
  color: #9a9a9a;
  background: var(--te-status-bg, #2a2a2a);
  border-top: 1px solid var(--te-border-subtle, #1f1f1f);
}
.status-msg {
  opacity: 0.85;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.status-count {
  color: #b0b0b0;
  font-weight: 600;
  font-size: 12px;
  flex-shrink: 0;
}
</style>