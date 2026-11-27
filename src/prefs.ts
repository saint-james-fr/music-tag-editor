import { computed, ref, watch } from 'vue'
import type { ColumnId } from './types'
import { formatColWidth, parseColWidth } from './column-width'
import { COLUMN_META, DEFAULT_COLUMN_LAYOUT } from './types'

export type ColumnLayoutColumn = {
  key: ColumnId
  visible: boolean
  width: string
}

export type ColumnLayout = {
  id: string
  name: string
  columns: ColumnLayoutColumn[]
}

export type AppPrefs = {
  renamePattern: string
  padTrackNumber: boolean
  updateTracksTotalOnEdit: boolean
  confirmBeforeRename: boolean
  layouts: ColumnLayout[]
  activeLayoutId: string
}

const STORAGE_KEY = 'tag-editor-prefs-v6'

const defaultLayout = (): ColumnLayout => ({
  id: 'default',
  name: 'Default',
  columns: DEFAULT_COLUMN_LAYOUT.map((c) => ({ ...c })),
})

const defaultPrefs = (): AppPrefs => ({
  renamePattern: '{{tracknumber}} - {{artist}} - {{title}}',
  padTrackNumber: true,
  updateTracksTotalOnEdit: true,
  confirmBeforeRename: false,
  layouts: [defaultLayout()],
  activeLayoutId: 'default',
})

const normalizeLayout = (layout: ColumnLayout): ColumnLayout => {
  const byKey = new Map(layout.columns.map((c) => [c.key, c]))
  return {
    ...layout,
    columns: COLUMN_META.map((meta) => {
      const existing = byKey.get(meta.key)
      const width = existing?.width
        ? formatColWidth(parseColWidth(existing.width))
        : meta.defaultWidth
      return {
        key: meta.key,
        visible: existing?.visible ?? true,
        width,
      }
    }),
  }
}

const loadPrefs = (): AppPrefs => {
  try {
    const raw = localStorage.getItem(STORAGE_KEY) ?? localStorage.getItem('tag-editor-prefs-v5')
    if (!raw) return defaultPrefs()
    const parsed = JSON.parse(raw) as AppPrefs & { columnLists?: unknown }
    if (!parsed.layouts?.length) parsed.layouts = [defaultLayout()]
    if (!parsed.activeLayoutId) parsed.activeLayoutId = parsed.layouts[0].id
    parsed.layouts = parsed.layouts.map(normalizeLayout)
    const { columnLists: _drop, ...rest } = parsed
    return { ...defaultPrefs(), ...rest }
  } catch {
    return defaultPrefs()
  }
}

const prefs = ref<AppPrefs>(loadPrefs())

watch(
  prefs,
  (p) => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(p))
  },
  { deep: true },
)

export const usePrefs = () => {
  const activeLayout = computed(
    () =>
      prefs.value.layouts.find((l) => l.id === prefs.value.activeLayoutId) ??
      prefs.value.layouts[0],
  )

  const visibleColumns = computed(() =>
    activeLayout.value.columns.filter((c) => c.visible),
  )

  const setColumnWidth = (key: ColumnId, widthPx: number) => {
    const layout = prefs.value.layouts.find((l) => l.id === prefs.value.activeLayoutId)
    const col = layout?.columns.find((c) => c.key === key)
    if (col) col.width = `${Math.max(40, Math.round(widthPx))}px`
  }

  return {
    prefs,
    activeLayout,
    visibleColumns,
    setColumnWidth,
  }
}
