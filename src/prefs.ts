import { computed, ref, watch } from 'vue'
import type { ColumnId } from './types'
import { formatColWidth, parseColWidth } from './column-width'
import { COLUMN_META, DEFAULT_COLUMN_LAYOUT, DEFAULT_GENRE_LIST } from './types'

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
  columnLists: Partial<Record<ColumnId, string[]>>
  layouts: ColumnLayout[]
  activeLayoutId: string
}

const STORAGE_KEY = 'tag-editor-prefs-v5'

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
  columnLists: { genre: [...DEFAULT_GENRE_LIST] },
  layouts: [defaultLayout()],
  activeLayoutId: 'default',
})

const normalizeLayout = (layout: ColumnLayout): ColumnLayout => {
  const byKey = new Map(layout.columns.map((c) => [c.key, c]))
  return {
    ...layout,
    // Keep COLUMN_META order; preserve user widths/visibility.
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
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return defaultPrefs()
    const parsed = JSON.parse(raw) as AppPrefs
    if (!parsed.layouts?.length) parsed.layouts = [defaultLayout()]
    if (!parsed.activeLayoutId) parsed.activeLayoutId = parsed.layouts[0].id
    parsed.layouts = parsed.layouts.map(normalizeLayout)
    return { ...defaultPrefs(), ...parsed }
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

  const listForColumn = (key: ColumnId) => prefs.value.columnLists[key] ?? []

  const setLayoutActive = (id: string) => {
    prefs.value.activeLayoutId = id
  }

  const saveLayoutAs = (name: string, columns: ColumnLayoutColumn[]) => {
    const id = crypto.randomUUID()
    prefs.value.layouts.push({ id, name, columns: columns.map((c) => ({ ...c })) })
    prefs.value.activeLayoutId = id
    return id
  }

  const updateActiveLayout = (columns: ColumnLayoutColumn[]) => {
    const layout = prefs.value.layouts.find((l) => l.id === prefs.value.activeLayoutId)
    if (layout) layout.columns = columns.map((c) => ({ ...c }))
  }

  const setColumnWidth = (key: ColumnId, widthPx: number) => {
    const layout = prefs.value.layouts.find((l) => l.id === prefs.value.activeLayoutId)
    const col = layout?.columns.find((c) => c.key === key)
    if (col) col.width = `${Math.max(40, Math.round(widthPx))}px`
  }

  const deleteLayout = (id: string) => {
    if (prefs.value.layouts.length <= 1) return
    prefs.value.layouts = prefs.value.layouts.filter((l) => l.id !== id)
    if (prefs.value.activeLayoutId === id) {
      prefs.value.activeLayoutId = prefs.value.layouts[0].id
    }
  }

  return {
    prefs,
    activeLayout,
    visibleColumns,
    listForColumn,
    setLayoutActive,
    saveLayoutAs,
    updateActiveLayout,
    deleteLayout,
    setColumnWidth,
  }
}