<script setup lang="ts">
import { ref } from 'vue'
import ToolbarDisc from './ToolbarDisc.vue'
import ToolbarMenuPanel, { type MenuItem } from './ToolbarMenuPanel.vue'

const props = defineProps<{
  canSave: boolean
  canReload: boolean
  hasSelection: boolean
}>()

type ToolbarEvent =
  | 'open'
  | 'import'
  | 'remove'
  | 'move'
  | 'save'
  | 'load'
  | 'artwork'
  | 'rename'
  | 'reload'
  | 'revert'
  | 'cut'
  | 'copy'
  | 'paste'
  | 'clear'
  | 'prefs'

type Item = {
  label: string
  icon: string
  title?: string
  action: ToolbarEvent | 'menu'
  menuId?: string
  /** Toolbar cell width. */
  widthPx: number
  disabled?: (p: { canSave: boolean; canReload: boolean; hasSelection: boolean }) => boolean
  sepBefore?: boolean
}

const emit = defineEmits<{
  open: []
  import: []
  remove: []
  move: []
  save: []
  load: []
  artwork: []
  rename: []
  reload: []
  revert: []
  cut: []
  copy: []
  paste: []
  clear: []
  prefs: []
  menu: [id: string, itemId: string]
}>()

const menus: Record<string, MenuItem[]> = {
  export: [
    { id: 'folder', label: 'Export Artwork to Folder…' },
  ],
  load: [
    { id: 'file', label: 'Load Tags from File…' },
    { id: 'paths', label: 'Load Tags from Paths…', disabled: true },
    { id: 'network', label: 'Load Tags from Network…', disabled: true },
  ],
  action: [
    { id: 'compose', label: 'Compose…' },
    { id: 'assign-tracks', label: 'Assign Track Numbers…', disabled: true },
    { id: 'transliterate', label: 'Transliterate…', disabled: true },
    { id: 'prefs', label: 'Preferences…' },
  ],
  artwork: [
    { id: 'load-path', label: 'Load from Path…' },
    { id: 'load-network', label: 'Load from Network…', disabled: true },
    { id: 'replace', label: 'Replace…' },
    { id: 'export', label: 'Export Artwork…' },
    { id: 'remove', label: 'Remove Artwork' },
  ],
  rename: [
    { id: 'pattern', label: 'Rename Files…' },
    { id: 'preview', label: 'Preview Rename', disabled: true },
  ],
}

const items: Item[] = [
  { label: 'Open', icon: 'Open', title: 'Open folder', action: 'open', widthPx: 46 },
  {
    label: 'Import',
    icon: 'ImportFromMusic',
    title: 'Import files',
    action: 'import',
    widthPx: 52,
  },
  {
    label: 'Remove',
    icon: 'Remove',
    action: 'remove',
    widthPx: 60,
    disabled: (p) => !p.hasSelection,
  },
  {
    label: 'Move',
    icon: 'Move',
    action: 'move',
    widthPx: 48,
    disabled: (p) => !p.hasSelection,
  },
  {
    label: 'Save',
    icon: 'Save',
    action: 'save',
    widthPx: 44,
    disabled: (p) => !p.canSave,
  },
  {
    label: 'Export',
    icon: 'ExportFolder',
    action: 'menu',
    menuId: 'export',
    widthPx: 52,
    disabled: (p) => !p.canReload,
  },
  {
    label: 'Load',
    icon: 'LoadTags',
    action: 'menu',
    menuId: 'load',
    widthPx: 48,
    sepBefore: true,
    disabled: (p) => !p.canReload,
  },
  {
    label: 'Action',
    icon: 'Action',
    action: 'menu',
    menuId: 'action',
    widthPx: 52,
  },
  {
    label: 'Artwork',
    icon: 'Artwork',
    action: 'menu',
    menuId: 'artwork',
    widthPx: 60,
    disabled: (p) => !p.canReload,
  },
  {
    label: 'Rename',
    icon: 'Rename',
    action: 'menu',
    menuId: 'rename',
    widthPx: 60,
    disabled: (p) => !p.canReload,
  },
  {
    label: 'Revert',
    icon: 'Revert',
    action: 'revert',
    widthPx: 52,
    disabled: (p) => !p.canReload,
  },
  {
    label: 'Reload',
    icon: 'Reload',
    action: 'reload',
    widthPx: 54,
    disabled: (p) => !p.canReload,
  },
  {
    label: 'Cut',
    icon: 'Cut',
    action: 'cut',
    widthPx: 44,
    sepBefore: true,
    disabled: (p) => !p.hasSelection,
  },
  {
    label: 'Copy',
    icon: 'Copy',
    action: 'copy',
    widthPx: 46,
    disabled: (p) => !p.hasSelection,
  },
  {
    label: 'Paste',
    icon: 'Paste',
    action: 'paste',
    widthPx: 48,
    disabled: (p) => !p.hasSelection,
  },
  {
    label: 'Clear',
    icon: 'Clear',
    action: 'clear',
    widthPx: 46,
    disabled: (p) => !p.hasSelection,
  },
  {
    label: 'Customize',
    icon: 'Appearance',
    action: 'prefs',
    widthPx: 74,
    sepBefore: true,
  },
]

const iconUrl = (name: string) => `/icons/${name}.png`

const menuOpen = ref<string | null>(null)
const menuPos = ref({ top: 0, left: 0 })

const openMenu = (menuId: string, el: EventTarget | null) => {
  const btn = el as HTMLElement | null
  if (!btn) return
  const r = btn.getBoundingClientRect()
  menuPos.value = { top: r.bottom + 2, left: r.left + 4 }
  menuOpen.value = menuOpen.value === menuId ? null : menuId
}

const closeMenu = () => {
  menuOpen.value = null
}

const fire = (action: ToolbarEvent) => {
  closeMenu()
  switch (action) {
    case 'open':
      emit('open')
      break
    case 'import':
      emit('import')
      break
    case 'remove':
      emit('remove')
      break
    case 'move':
      emit('move')
      break
    case 'save':
      emit('save')
      break
    case 'load':
      emit('load')
      break
    case 'artwork':
      emit('artwork')
      break
    case 'rename':
      emit('rename')
      break
    case 'reload':
      emit('reload')
      break
    case 'revert':
      emit('revert')
      break
    case 'cut':
      emit('cut')
      break
    case 'copy':
      emit('copy')
      break
    case 'paste':
      emit('paste')
      break
    case 'clear':
      emit('clear')
      break
    case 'prefs':
      emit('prefs')
      break
    default: {
      const _exhaustive: never = action
      void _exhaustive
    }
  }
}

const onClick = (item: Item, e: MouseEvent) => {
  if (item.action === 'menu' && item.menuId) {
    if (item.menuId === 'export' && item.disabled?.(props)) return
    openMenu(item.menuId, e.currentTarget)
    return
  }
  if (item.disabled?.(props)) return
  if (item.action !== 'menu') fire(item.action)
}

const onMenuPick = (itemId: string) => {
  const id = menuOpen.value
  if (!id) return
  emit('menu', id, itemId)
  closeMenu()
}
</script>

<template>
  <div class="icon-toolbar toolbar-strip">
    <template v-for="item in items" :key="item.icon">
      <span v-if="item.sepBefore" class="sep" aria-hidden="true" />
      <ToolbarDisc
        class="tb-menu"
        :class="{ 'menu-open': menuOpen === item.menuId }"
        :label="item.label"
        :title="item.title ?? item.label"
        :width-px="item.widthPx"
        :has-menu="item.action === 'menu'"
        :disabled="item.action !== 'menu' && !!item.disabled?.(props)"
        :dimmed="!!item.disabled?.(props)"
        @click="onClick(item, $event)"
      >
        <img class="te-icon" :src="iconUrl(item.icon)" alt="" />
      </ToolbarDisc>
    </template>
    <ToolbarMenuPanel
      v-if="menuOpen && menus[menuOpen]"
      :open="!!menuOpen"
      :items="menus[menuOpen]"
      :top="menuPos.top"
      :left="menuPos.left"
      @pick="onMenuPick"
      @close="closeMenu"
    />
  </div>
</template>

<style scoped>
.icon-toolbar {
  display: flex;
  flex-wrap: nowrap;
  overflow-x: auto;
  overflow-y: hidden;
  align-items: flex-start;
  gap: 0;
  padding: 3px 4px 1px 4px;
  min-height: var(--te-toolbar-strip-h, 51px);
  flex-shrink: 0;
  position: relative;
  z-index: 5;
  scrollbar-width: none;
}
.icon-toolbar::-webkit-scrollbar {
  display: none;
}
.sep {
  width: 1px;
  height: var(--te-toolbar-item-h, 51px);
  margin: 0 4px 0 3px;
  align-self: flex-start;
  flex-shrink: 0;
  background: rgba(255, 255, 255, 0.11);
}
.menu-open :deep(.glyph-wrap) {
  background: radial-gradient(circle at 38% 32%, #7a7a7a 0%, #555 52%, #484848 100%);
}
</style>