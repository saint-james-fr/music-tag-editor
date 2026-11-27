import { ref } from 'vue'
import type { TrackRow } from './types'

export type EditorTab = {
  id: string
  title: string
  folderLabel: string
  query: string
  tracks: TrackRow[]
}

const newTab = (title = 'Untitled'): EditorTab => ({
  id: crypto.randomUUID(),
  title,
  folderLabel: '',
  query: '',
  tracks: [],
})

const tabs = ref<EditorTab[]>([newTab('Tab 1')])
const activeTabId = ref(tabs.value[0].id)

export const useTabs = () => {
  const activeTab = () => tabs.value.find((t) => t.id === activeTabId.value) ?? tabs.value[0]

  const addTab = () => {
    const t = newTab(`Tab ${tabs.value.length + 1}`)
    tabs.value.push(t)
    activeTabId.value = t.id
    return t
  }

  const closeTab = (id: string) => {
    if (tabs.value.length <= 1) {
      const t = tabs.value[0]
      t.tracks = []
      t.folderLabel = ''
      t.query = ''
      t.title = 'Tab 1'
      return
    }
    const idx = tabs.value.findIndex((t) => t.id === id)
    if (idx < 0) return
    tabs.value.splice(idx, 1)
    if (activeTabId.value === id) {
      activeTabId.value = tabs.value[Math.max(0, idx - 1)].id
    }
  }

  return { tabs, activeTabId, activeTab, addTab, closeTab }
}
