<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useFileDrop } from '../composables/useFileDrop'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { open } from '@tauri-apps/plugin-dialog'
import { parseColWidth } from '../column-width'
import { startColumnResize } from '../composables/useColumnResize'
import { usePrefs } from '../prefs'
import type { EditorTab } from '../tabs'
import type { ArtworkChange, ArtworkPayload, ColumnId, ColumnKey, TrackRow } from '../types'
import { isBrowserPreview } from '../demo-tracks'
import {
  COLUMN_META,
  DEFAULT_TAG_FORMAT,
  TAG_FORMAT_HINTS,
  TAG_FORMAT_OPTIONS,
} from '../types'
import ArtworkPopover from './ArtworkPopover.vue'
import CellComposePopover from './CellComposePopover.vue'
import IconToolbar from './IconToolbar.vue'
import RenamePopover from './RenamePopover.vue'

const props = defineProps<{
  tab: EditorTab
}>()

const emit = defineEmits<{
  status: [string]
  'update:showPrefs': [boolean]
}>()

const { prefs, visibleColumns, setColumnWidth } = usePrefs()

const busy = ref(false)
const filterEnabled = ref(true)
const replaceEnabled = ref(false)
const replaceText = ref('')
const artworkPending = ref<ArtworkChange[]>([])
const thumbCache = ref<Record<string, string>>({})
const artPopoverRows = ref<TrackRow[] | null>(null)
const artPreview = ref<ArtworkPayload | null>(null)
const showRename = ref(false)
const dropActive = ref(false)
const focusCell = ref<{ rowId: string; col: ColumnId } | null>(null)
const composeOpen = ref<{ col: ColumnId; rows: TrackRow[] } | null>(null)
const tagFormatMenu = ref<{ x: number; y: number; current: string; rows: TrackRow[] } | null>(null)
const sortKey = ref<ColumnId | 'num'>('fileName')
const sortAsc = ref(true)
/** Excel-style cell range (filtered row index × visible col index). */
const rangeAnchor = ref<{ ri: number; ci: number } | null>(null)
const rangeEnd = ref<{ ri: number; ci: number } | null>(null)

const renamePattern = computed({
  get: () => prefs.value.renamePattern,
  set: (v) => {
    prefs.value.renamePattern = v
  },
})

const ROW_INDEX_W = 50
const gridTemplate = computed(() => {
  const cols = visibleColumns.value.map((c) => {
    const px = parseColWidth(c.width)
    if (c.key === 'artwork') return `${px}px`
    return `minmax(${px}px, 1fr)`
  })
  return `${ROW_INDEX_W}px ${cols.join(' ')}`
})

const cellValue = (row: TrackRow, key: ColumnId) => {
  if (key === 'artwork') return row.hasArtwork ? '1' : ''
  if (key === 'fileName') return row.pendingName ?? row.fileName
  return String(row[key as ColumnKey] ?? '')
}

const filtered = computed(() => {
  const q = props.tab.query.trim().toLowerCase()
  let rows = props.tab.tracks
  if (filterEnabled.value && q) {
    rows = rows.filter((row) =>
      visibleColumns.value.some((col) => {
        if (col.key === 'artwork') return row.hasArtwork
        return cellValue(row, col.key).toLowerCase().includes(q)
      }),
    )
  }
  const key = sortKey.value
  const dir = sortAsc.value ? 1 : -1
  return [...rows].sort((a, b) => {
    if (key === 'num') {
      return (props.tab.tracks.indexOf(a) - props.tab.tracks.indexOf(b)) * dir
    }
    return cellValue(a, key).localeCompare(cellValue(b, key), undefined, {
      numeric: true,
      sensitivity: 'base',
    }) * dir
  })
})

const toggleSort = (key: ColumnId) => {
  if (sortKey.value === key) sortAsc.value = !sortAsc.value
  else {
    sortKey.value = key
    sortAsc.value = true
  }
}

const dirtyCount = computed(() => props.tab.tracks.filter((t) => t.dirty).length)
const pendingRenames = computed(() => props.tab.tracks.filter((t) => t.pendingName).length)
const colIndexOf = (col: ColumnId) => visibleColumns.value.findIndex((c) => c.key === col)

const cellRange = computed(() => {
  if (!rangeAnchor.value || !rangeEnd.value) return null
  return {
    r0: Math.min(rangeAnchor.value.ri, rangeEnd.value.ri),
    r1: Math.max(rangeAnchor.value.ri, rangeEnd.value.ri),
    c0: Math.min(rangeAnchor.value.ci, rangeEnd.value.ci),
    c1: Math.max(rangeAnchor.value.ci, rangeEnd.value.ci),
  }
})

const isCellSelected = (ri: number, col: ColumnId) => {
  const range = cellRange.value
  if (!range) return false
  const ci = colIndexOf(col)
  if (ci < 0) return false
  return ri >= range.r0 && ri <= range.r1 && ci >= range.c0 && ci <= range.c1
}

const isRowSelected = (ri: number) => {
  const range = cellRange.value
  if (!range) return false
  return (
    ri >= range.r0 &&
    ri <= range.r1 &&
    range.c0 === 0 &&
    range.c1 === visibleColumns.value.length - 1
  )
}

const selectedRows = computed(() => {
  const range = cellRange.value
  if (!range) return [] as TrackRow[]
  return filtered.value.slice(range.r0, range.r1 + 1)
})
const artPendingCount = computed(() => artworkPending.value.length)

const windowTitle = computed(() => {
  const n = props.tab.tracks.length
  if (props.tab.folderLabel) return `${props.tab.folderLabel.toUpperCase()} (${n} files)`
  return n ? `Tag Editor (${n} files)` : 'Tag Editor'
})

watch(
  windowTitle,
  (t) => {
    if (isBrowserPreview()) return
    void getCurrentWindow().setTitle(t)
  },
  { immediate: true },
)

const loadFromPaths = async (paths: string[], merge = false) => {
  if (!paths.length) return
  busy.value = true
  try {
    const rows = await invoke<TrackRow[]>('load_paths', { paths })
    if (merge && props.tab.tracks.length) {
      const byId = new Map(props.tab.tracks.map((t) => [t.id, t]))
      rows.forEach((r) => byId.set(r.id, r))
      props.tab.tracks = [...byId.values()].sort((a, b) => a.path.localeCompare(b.path))
    } else {
      props.tab.tracks = rows
    }
    if (paths.length === 1) {
      props.tab.folderLabel = paths[0].split(/[/\\]/).filter(Boolean).pop() ?? paths[0]
    } else {
      const parents = new Set(paths.map((p) => p.replace(/[/\\][^/\\]+$/, '')))
      props.tab.folderLabel =
        parents.size === 1
          ? (parents.values().next().value?.split(/[/\\]/).pop() ?? `${paths.length} files`)
          : `${paths.length} files`
    }
    if (!props.tab.title || props.tab.title.startsWith('Tab ')) {
      props.tab.title = props.tab.folderLabel || props.tab.title
    }
    emit('status', `Importé : ${props.tab.tracks.length} fichier(s).`)
    preloadThumbs(rows.filter((r) => r.hasArtwork))
  } catch (e) {
    emit('status', String(e))
  } finally {
    busy.value = false
  }
}

const preloadThumbs = async (rows: TrackRow[]) => {
  for (const row of rows.slice(0, 80)) {
    if (thumbCache.value[row.path]) continue
    try {
      const payload = await invoke<ArtworkPayload>('get_artwork', { path: row.path })
      thumbCache.value[row.path] = `data:${payload.mime};base64,${payload.dataBase64}`
    } catch {
      /* none */
    }
  }
}

const reloadRows = async () => {
  const paths = props.tab.tracks.map((t) => t.path)
  if (!paths.length) return
  busy.value = true
  try {
    props.tab.tracks = await invoke<TrackRow[]>('load_paths', { paths })
    artworkPending.value = []
    emit('status', 'Reloaded from disk.')
    preloadThumbs(props.tab.tracks.filter((r) => r.hasArtwork))
  } catch (e) {
    emit('status', String(e))
  } finally {
    busy.value = false
  }
}

defineExpose({ loadPaths: loadFromPaths, reload: reloadRows })

const openFolder = async () => {
  const picked = await open({ directory: true, multiple: false, title: 'Open folder' })
  if (picked && typeof picked === 'string') await loadFromPaths([picked])
}

const importFiles = async () => {
  const picked = await open({ multiple: true, title: 'Import audio files' })
  if (!picked) return
  const paths = Array.isArray(picked) ? picked : [picked]
  await loadFromPaths(paths, true)
}

const removeSelected = () => {
  if (!selectedRows.value.length) return
  const ids = new Set(selectedRows.value.map((r) => r.id))
  props.tab.tracks = props.tab.tracks.filter((r) => !ids.has(r.id))
  emit('status', `Removed ${ids.size} row(s) from list.`)
}

const revertEdits = async () => {
  if (dirtyCount.value || artPendingCount.value) {
    if (!confirm('Discard unsaved tag and artwork changes?')) return
  }
  await reloadRows()
}

const setCellValue = (row: TrackRow, key: ColumnKey, value: string) => {
  if (key === 'fileName' || key === 'tagFormat') return
  row[key] = value
  row.dirty = true
  if (key === 'trackNumber' && prefs.value.updateTracksTotalOnEdit && !row.tracksTotal) {
    const nums = props.tab.tracks
      .map((t) => parseInt(t.trackNumber, 10))
      .filter((n) => !Number.isNaN(n))
    if (nums.length) row.tracksTotal = String(Math.max(...nums, props.tab.tracks.length))
  }
}

/** Excel-like: typing fills the active column across every selected row in the range. */
const onCellInput = (row: TrackRow, key: ColumnKey, value: string) => {
  const range = cellRange.value
  const ri = filtered.value.findIndex((r) => r.id === row.id)
  const inRange = range && ri >= range.r0 && ri <= range.r1
  const targets =
    inRange && selectedRows.value.length > 1 ? selectedRows.value : [row]
  targets.forEach((r) => setCellValue(r, key, value))
}

const save = async () => {
  busy.value = true
  try {
    const result = await invoke<{ ok: number; errors: string[] }>('save_tracks', {
      tracks: props.tab.tracks,
      artworkChanges: artworkPending.value,
    })
    props.tab.tracks.forEach((t) => {
      t.dirty = false
    })
    artworkPending.value = []
    emit(
      'status',
      result.errors.length === 0
        ? `Saved ${result.ok} change(s).`
        : `Save: ${result.ok} ok, ${result.errors.length} failed. ${result.errors[0]}`,
    )
  } catch (e) {
    emit('status', String(e))
  } finally {
    busy.value = false
  }
}

const previewRename = async () => {
  props.tab.tracks = await invoke<TrackRow[]>('preview_rename', {
    tracks: props.tab.tracks,
    pattern: renamePattern.value,
    padTrack: prefs.value.padTrackNumber,
  })
  emit('status', `${pendingRenames.value} pending rename(s).`)
}

const applyRename = async () => {
  if (prefs.value.confirmBeforeRename && !confirm('Apply file renames on disk?')) return
  busy.value = true
  try {
    props.tab.tracks = await invoke<TrackRow[]>('rename_tracks', {
      tracks: props.tab.tracks,
      pattern: renamePattern.value,
      padTrack: prefs.value.padTrackNumber,
    })
    emit('status', 'Rename complete.')
    showRename.value = false
  } catch (e) {
    emit('status', String(e))
  } finally {
    busy.value = false
  }
}

const fillColumn = (key: ColumnId) => {
  if (key === 'artwork' || isReadonlyCol(key)) return
  const rows = filtered.value
  if (!rows.length) return
  const first = rows[0][key as ColumnKey]
  rows.forEach((row) => setCellValue(row, key as ColumnKey, first))
}

/** Index column = Excel row header (full row). */
const selectRowHeader = (ri: number, e: MouseEvent) => {
  e.preventDefault()
  const last = Math.max(0, visibleColumns.value.length - 1)
  if (e.shiftKey && rangeAnchor.value) {
    rangeEnd.value = { ri, ci: last }
    rangeAnchor.value = { ri: rangeAnchor.value.ri, ci: 0 }
  } else {
    rangeAnchor.value = { ri, ci: 0 }
    rangeEnd.value = { ri, ci: last }
  }
  const row = filtered.value[ri]
  if (row) focusCell.value = { rowId: row.id, col: visibleColumns.value[0]?.key ?? 'title' }
}

/** Excel cell select: click = one cell, Shift+click = rectangle. */
const selectCellAt = (ri: number, col: ColumnId, e: MouseEvent) => {
  const ci = colIndexOf(col)
  if (ci < 0) return
  if (e.shiftKey && rangeAnchor.value) {
    e.preventDefault()
    rangeEnd.value = { ri, ci }
  } else if (e.metaKey || e.ctrlKey) {
    e.preventDefault()
    // ponytail: treat ⌘-click like a new anchor (no additive ranges)
    rangeAnchor.value = { ri, ci }
    rangeEnd.value = { ri, ci }
  } else {
    rangeAnchor.value = { ri, ci }
    rangeEnd.value = { ri, ci }
  }
  const row = filtered.value[ri]
  if (row) focusCell.value = { rowId: row.id, col }
}

const setFocus = (row: TrackRow, col: ColumnId, rowIndex?: number) => {
  focusCell.value = { rowId: row.id, col }
  const ri = rowIndex ?? filtered.value.findIndex((r) => r.id === row.id)
  const ci = colIndexOf(col)
  if (ri < 0 || ci < 0) return
  // Keyboard/focus without modifiers: keep range if cell already inside it.
  if (isCellSelected(ri, col)) return
  rangeAnchor.value = { ri, ci }
  rangeEnd.value = { ri, ci }
}

const onCellMouseDown = (row: TrackRow, ri: number, col: ColumnId, e: MouseEvent) => {
  if (e.shiftKey || e.metaKey || e.ctrlKey) e.preventDefault()
  // Keep multi-select when opening Tag Format inside the selection
  if (
    col === 'tagFormat' &&
    !e.shiftKey &&
    !e.metaKey &&
    !e.ctrlKey &&
    selectedRows.value.length > 1 &&
    isCellSelected(ri, col)
  ) {
    focusCell.value = { rowId: row.id, col }
    return
  }
  selectCellAt(ri, col, e)
}

const isFocused = (row: TrackRow, col: ColumnId) =>
  focusCell.value?.rowId === row.id && focusCell.value?.col === col

const composeTargets = (row: TrackRow) =>
  selectedRows.value.length > 1 ? selectedRows.value : [row]

const cellClipboard = ref('')

const editTargets = () => {
  if (selectedRows.value.length) return selectedRows.value
  const id = focusCell.value?.rowId
  const one = id ? props.tab.tracks.find((t) => t.id === id) : null
  return one ? [one] : []
}

const hasEditTarget = computed(() => selectedRows.value.length > 0 || !!focusCell.value)

const copyCells = async () => {
  const col = focusCell.value?.col
  if (!col || col === 'artwork') return
  const row =
    props.tab.tracks.find((t) => t.id === focusCell.value?.rowId) ?? editTargets()[0]
  if (!row) return
  const text = cellValue(row, col)
  cellClipboard.value = text
  try {
    await navigator.clipboard.writeText(text)
  } catch {
    /* ignore */
  }
  emit('status', `Copied “${text.slice(0, 40)}${text.length > 40 ? '…' : ''}”`)
}

const pasteCells = async () => {
  const col = focusCell.value?.col
  if (!col || col === 'artwork' || isReadonlyCol(col) || col === 'fileName') return
  let text = cellClipboard.value
  try {
    const clip = await navigator.clipboard.readText()
    if (clip !== '') text = clip
  } catch {
    /* keep buffer */
  }
  const targets = editTargets()
  if (!targets.length) return
  targets.forEach((r) => setCellValue(r, col as ColumnKey, text))
  emit('status', `Pasted → ${targets.length} row(s)`)
}

const clearCells = () => {
  const col = focusCell.value?.col
  if (!col || col === 'artwork' || isReadonlyCol(col) || col === 'fileName') return
  const targets = editTargets()
  targets.forEach((r) => setCellValue(r, col as ColumnKey, ''))
  emit('status', `Cleared ${colLabel(col)} on ${targets.length} row(s)`)
}

const cutCells = async () => {
  await copyCells()
  clearCells()
}

const hasTextSelection = () => {
  const el = document.activeElement
  if (!(el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement)) return false
  return el.selectionStart !== el.selectionEnd
}

const caretAtInputEdge = (dir: 'left' | 'right') => {
  const el = document.activeElement
  if (!(el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement)) return true
  const start = el.selectionStart ?? 0
  const end = el.selectionEnd ?? 0
  if (start !== end) return false
  if (dir === 'left') return start === 0
  return end === el.value.length
}

const focusDomCell = async (ri: number, col: ColumnId) => {
  await nextTick()
  const el = document.querySelector(
    `.sheet .grid.row[data-ri="${ri}"] [data-col="${CSS.escape(col)}"]`,
  ) as HTMLElement | null
  el?.focus({ preventScroll: false })
  el?.scrollIntoView({ block: 'nearest', inline: 'nearest' })
}

const moveActiveCell = (dri: number, dci: number, extend: boolean) => {
  const rows = filtered.value
  const cols = visibleColumns.value
  if (!rows.length || !cols.length) return
  let ri = 0
  let ci = 0
  if (focusCell.value) {
    ri = rows.findIndex((r) => r.id === focusCell.value!.rowId)
    ci = colIndexOf(focusCell.value.col)
  } else if (rangeEnd.value) {
    ri = rangeEnd.value.ri
    ci = rangeEnd.value.ci
  }
  if (ri < 0) ri = 0
  if (ci < 0) ci = 0
  const nri = Math.max(0, Math.min(rows.length - 1, ri + dri))
  const nci = Math.max(0, Math.min(cols.length - 1, ci + dci))
  const col = cols[nci].key
  if (extend && rangeAnchor.value) {
    rangeEnd.value = { ri: nri, ci: nci }
  } else {
    rangeAnchor.value = { ri: nri, ci: nci }
    rangeEnd.value = { ri: nri, ci: nci }
  }
  focusCell.value = { rowId: rows[nri].id, col }
  void focusDomCell(nri, col)
}

const onPaneKeydown = (e: KeyboardEvent) => {
  const meta = e.metaKey || e.ctrlKey
  if (meta) {
    if (e.key === 'c' && !hasTextSelection()) {
      e.preventDefault()
      void copyCells()
    } else if (e.key === 'v' && !hasTextSelection()) {
      e.preventDefault()
      void pasteCells()
    } else if (e.key === 'x' && !hasTextSelection()) {
      e.preventDefault()
      void cutCells()
    }
    return
  }

  const arrow = e.key === 'ArrowUp' || e.key === 'ArrowDown' || e.key === 'ArrowLeft' || e.key === 'ArrowRight'
  const tab = e.key === 'Tab'
  if (!arrow && !tab) return
  if (!focusCell.value && !rangeEnd.value) return

  if (e.key === 'ArrowLeft' && !caretAtInputEdge('left')) return
  if (e.key === 'ArrowRight' && !caretAtInputEdge('right')) return

  e.preventDefault()
  if (e.key === 'ArrowUp') moveActiveCell(-1, 0, e.shiftKey)
  else if (e.key === 'ArrowDown') moveActiveCell(1, 0, e.shiftKey)
  else if (e.key === 'ArrowLeft') moveActiveCell(0, -1, e.shiftKey)
  else if (e.key === 'ArrowRight') moveActiveCell(0, 1, e.shiftKey)
  else if (tab) moveActiveCell(0, e.shiftKey ? -1 : 1, false)
}

const colMeta = (key: ColumnId) => COLUMN_META.find((m) => m.key === key)
const colAlign = (key: ColumnId) => colMeta(key)?.align ?? 'left'
const isReadonlyCol = (key: ColumnId) => !!colMeta(key)?.readonly

const openCompose = (row: TrackRow, col: ColumnId) => {
  composeOpen.value = { col, rows: composeTargets(row) }
}

const openTagFormatMenu = (row: TrackRow, e: MouseEvent) => {
  const rows = composeTargets(row)
  const current =
    rows.length === 1 && rows[0].tagFormat
      ? rows[0].tagFormat
      : DEFAULT_TAG_FORMAT
  const menuW = 220
  const menuH = 4 * 28 + 12
  const x = Math.min(e.clientX, window.innerWidth - menuW - 8)
  const y = Math.min(e.clientY, window.innerHeight - menuH - 8)
  tagFormatMenu.value = { x: Math.max(8, x), y: Math.max(8, y), current, rows }
}

const applyTagFormat = (format: string) => {
  const menu = tagFormatMenu.value
  if (!menu) return
  menu.rows.forEach((r) => {
    r.tagFormat = format
    r.dirty = true
  })
  tagFormatMenu.value = null
  emit('status', `Tag format → ${format} (${menu.rows.length})`)
}

const mergeTrackPatches = (patched: TrackRow[]) => {
  const byId = new Map(patched.map((r) => [r.id, r]))
  props.tab.tracks = props.tab.tracks.map((r) => byId.get(r.id) ?? r)
}

const onComposeField = (value: string) => {
  if (!composeOpen.value) return
  const key = composeOpen.value.col as ColumnKey
  if (key === 'fileName' || key === 'tagFormat') return
  const n = composeOpen.value.rows.length
  composeOpen.value.rows.forEach((row) => setCellValue(row, key, value))
  composeOpen.value = null
  emit('status', `Updated ${n} row(s).`)
}

const onComposePreview = async (pattern: string) => {
  prefs.value.renamePattern = pattern
  const rows = composeOpen.value?.rows ?? []
  if (!rows.length) return
  const ids = new Set(rows.map((r) => r.id))
  const updated = await invoke<TrackRow[]>('preview_rename', {
    tracks: props.tab.tracks.filter((r) => ids.has(r.id)),
    pattern,
    padTrack: prefs.value.padTrackNumber,
  })
  mergeTrackPatches(updated)
  emit('status', 'Preview rename for selection.')
}

const onComposeApplyPattern = async (pattern: string) => {
  prefs.value.renamePattern = pattern
  const rows = composeOpen.value?.rows ?? []
  if (!rows.length) return
  busy.value = true
  try {
    const ids = new Set(rows.map((r) => r.id))
    const updated = await invoke<TrackRow[]>('rename_tracks', {
      tracks: props.tab.tracks.filter((r) => ids.has(r.id)),
      pattern,
      padTrack: prefs.value.padTrackNumber,
    })
    mergeTrackPatches(updated)
    composeOpen.value = null
    emit('status', 'Renamed selection.')
  } catch (e) {
    emit('status', String(e))
  } finally {
    busy.value = false
  }
}

const applyReplace = () => {
  const q = props.tab.query.trim()
  if (!q) return
  const targets = selectedRows.value.length ? selectedRows.value : filtered.value
  const col = focusCell.value?.col
  if (!col || col === 'artwork') return
  targets.forEach((row) => {
    const key = col as ColumnKey
    const cur = row[key]
    if (!cur.includes(q)) return
    setCellValue(row, key, cur.split(q).join(replaceText.value))
  })
}

const findDone = () => {
  props.tab.query = ''
  replaceEnabled.value = false
  replaceText.value = ''
}

const findStep = (delta: number) => {
  const q = props.tab.query.trim()
  if (!q) return
  const rows = filtered.value
  if (!rows.length) return
  const start = focusCell.value
    ? rows.findIndex((r) => r.id === focusCell.value?.rowId)
    : -1
  const next = (start + delta + rows.length) % rows.length
  const row = rows[next]
  const col = focusCell.value?.col ?? visibleColumns.value.find((c) => c.key !== 'artwork')?.key
  if (col) setFocus(row, col, next)
}

const findPrev = () => findStep(-1)
const findNext = () => findStep(1)

const onToolbarMenu = (menuId: string, itemId: string) => {
  switch (menuId) {
    case 'export':
      if (itemId === 'folder') void exportArt()
      break
    case 'load':
      if (itemId === 'file') emit('status', 'Load tags from file — pas encore implémenté.')
      break
    case 'action':
      if (itemId === 'prefs') emit('update:showPrefs', true)
      if (
        itemId === 'compose' &&
        selectedRows.value.length &&
        focusCell.value &&
        focusCell.value.col !== 'artwork'
      ) {
        openCompose(selectedRows.value[0], focusCell.value.col)
      }
      break
    case 'artwork':
      if (itemId === 'replace' || itemId === 'load-path') openArtworkToolbar()
      if (itemId === 'export') void exportArt()
      if (itemId === 'remove') emit('status', 'Remove artwork via Artwork popover.')
      break
    case 'rename':
      if (itemId === 'pattern') showRename.value = true
      break
    default:
      break
  }
}

const onResizeStart = (key: ColumnId, width: string, e: PointerEvent) => {
  e.preventDefault()
  e.stopPropagation()
  const el = e.currentTarget as HTMLElement
  startColumnResize(key, width, e.clientX, (px) => setColumnWidth(key, px), e.pointerId, el)
}

const openArtwork = async (row: TrackRow) => {
  const targets =
    selectedRows.value.length > 1 && selectedRows.value.some((r) => r.id === row.id)
      ? selectedRows.value
      : [row]
  artPopoverRows.value = targets
  artPreview.value = null
  if (targets.length === 1 && targets[0].hasArtwork) {
    try {
      artPreview.value = await invoke<ArtworkPayload>('get_artwork', { path: targets[0].path })
    } catch {
      artPreview.value = null
    }
  }
}

const openArtworkToolbar = () => {
  const targets = selectedRows.value.length ? selectedRows.value : props.tab.tracks
  if (!targets.length) return
  void openArtwork(targets[0])
}

const onArtworkApply = (changes: ArtworkChange[]) => {
  artworkPending.value = [
    ...artworkPending.value.filter((c) => !changes.some((ch) => ch.path === c.path)),
    ...changes,
  ]
  const byPath = new Map(changes.map((c) => [c.path, c]))
  props.tab.tracks.forEach((row) => {
    const ch = byPath.get(row.path)
    if (!ch) return
    if (ch.action === 'remove') {
      row.hasArtwork = false
      delete thumbCache.value[row.path]
    } else {
      row.hasArtwork = true
      if (ch.dataBase64 && ch.mime) {
        thumbCache.value[row.path] = `data:${ch.mime};base64,${ch.dataBase64}`
      }
    }
  })
}

const exportArt = async () => {
  const folder = await open({ directory: true, multiple: false, title: 'Export artwork folder' })
  if (!folder || typeof folder !== 'string') return
  const paths = (artPopoverRows.value ?? props.tab.tracks)
    .filter((r) => r.hasArtwork)
    .map((r) => r.path)
  const result = await invoke<{ ok: number; errors: string[] }>('export_artworks', {
    paths,
    destDir: folder,
  })
  emit('status', `Exported ${result.ok} image(s).`)
}

const colLabel = (key: ColumnId) => COLUMN_META.find((m) => m.key === key)?.label ?? key

watch(
  () => props.tab.tracks.length,
  () => preloadThumbs(props.tab.tracks.filter((r) => r.hasArtwork)),
)

const onDroppedPaths = (paths: string[]) => {
  const merge = props.tab.tracks.length > 0
  void loadFromPaths(paths, merge)
}

useFileDrop(dropActive, onDroppedPaths)

onMounted(() => {
  preloadThumbs(props.tab.tracks.filter((r) => r.hasArtwork))
  window.addEventListener('keydown', onPaneKeydown)
})
onUnmounted(() => window.removeEventListener('keydown', onPaneKeydown))
</script>

<template>
  <div class="pane" :class="{ busy, 'drop-active': dropActive }">
    <div v-if="isBrowserPreview()" class="title-rail" aria-hidden="true">
      <span>{{ windowTitle }}</span>
    </div>

    <IconToolbar
      :can-save="dirtyCount + artPendingCount > 0"
      :can-reload="tab.tracks.length > 0"
      :has-selection="hasEditTarget"
      @open="openFolder"
      @import="importFiles"
      @remove="removeSelected"
      @move="emit('status', 'Move — pas encore implémenté.')"
      @save="save"
      @reload="reloadRows"
      @revert="revertEdits"
      @cut="cutCells"
      @copy="copyCells"
      @paste="pasteCells"
      @clear="clearCells"
      @prefs="emit('update:showPrefs', true)"
      @menu="onToolbarMenu"
    />

    <div v-if="dropActive" class="drop-hint" aria-live="polite">Déposer fichiers ou dossiers audio</div>

    <div class="find-bar">
      <div class="find-shell">
        <img class="find-glyph" src="/icons/Find.png" alt="" />
        <input
          v-model="tab.query"
          type="search"
          class="find-input"
          placeholder="Text to find. Contains, case insensitive search."
          autocomplete="off"
        />
        <div class="find-trail">
          <label class="chk filter">
            <input v-model="filterEnabled" type="checkbox" />
            Filter
          </label>
          <button type="button" class="find-nav" title="Previous" @click="findPrev">‹</button>
          <button type="button" class="find-nav" title="Next" @click="findNext">›</button>
          <button type="button" class="find-done" @click="findDone">Done</button>
          <label class="chk">
            <input v-model="replaceEnabled" type="checkbox" />
            Replace
          </label>
        </div>
      </div>
      <div v-if="replaceEnabled" class="replace-row">
        <input v-model="replaceText" type="text" class="replace-input" placeholder="Replace with…" />
        <button type="button" class="tiny" @click="applyReplace">Replace</button>
      </div>
    </div>

    <div class="grid-wrap" @mousedown="(e) => (e.target === e.currentTarget ? (focusCell = null) : null)">
      <div class="sheet">
        <div class="grid head scroll-edge-under" :style="{ gridTemplateColumns: gridTemplate }">
          <div class="cell idx head-idx" />
          <div
            v-for="col in visibleColumns"
            :key="col.key"
            class="cell head-cell"
            :class="{ sorted: sortKey === col.key, [`align-${colAlign(col.key)}`]: true }"
            @click="toggleSort(col.key)"
            @dblclick.stop="fillColumn(col.key)"
          >
            <span class="head-label">{{ colLabel(col.key) }}</span>
            <span v-if="sortKey === col.key" class="sort-mark">{{ sortAsc ? '▴' : '▾' }}</span>
            <span
              class="resize"
              title="Drag to resize column"
              @pointerdown="onResizeStart(col.key, col.width, $event)"
              @click.stop
            />
          </div>
        </div>
        <hr class="sheet-line" />
        <div
          v-for="(row, ri) in filtered"
          :key="row.id"
          class="grid row"
          :data-ri="ri"
          :class="{ alt: ri % 2 === 1, 'row-pick': isRowSelected(ri) }"
          :style="{ gridTemplateColumns: gridTemplate }"
        >
          <button
            type="button"
            class="cell idx"
            :class="{ active: isRowSelected(ri) || (cellRange && ri >= cellRange.r0 && ri <= cellRange.r1) }"
            @mousedown="selectRowHeader(ri, $event)"
          >
            {{ tab.tracks.indexOf(row) + 1 }}
          </button>
          <template v-for="col in visibleColumns" :key="col.key">
            <button
              v-if="col.key === 'artwork'"
              type="button"
              class="cell art-btn"
              :data-col="col.key"
              :class="{
                focus: isFocused(row, col.key),
                'cell-sel': isCellSelected(ri, col.key),
              }"
              @mousedown="onCellMouseDown(row, ri, col.key, $event)"
              @click="setFocus(row, col.key, ri); openArtwork(row)"
            >
              <img v-if="thumbCache[row.path]" :src="thumbCache[row.path]" alt="" />
              <span v-else class="art-ph">{{ row.hasArtwork ? '♪' : '' }}</span>
            </button>
            <div
              v-else
              class="cell-wrap"
              :class="{
                'cell-sel': isCellSelected(ri, col.key),
                [`align-${colAlign(col.key)}`]: true,
              }"
              @mousedown="onCellMouseDown(row, ri, col.key, $event)"
            >
              <div
                v-if="col.key === 'fileName' || isReadonlyCol(col.key)"
                class="cell read"
                :data-col="col.key"
                :class="{
                  'file-name': col.key === 'fileName',
                  'tag-format': col.key === 'tagFormat',
                  focus: isFocused(row, col.key),
                  pending: col.key === 'fileName' && row.pendingName,
                  'cell-sel': isCellSelected(ri, col.key),
                }"
                tabindex="0"
                @focus="setFocus(row, col.key, ri)"
                @click="
                  setFocus(row, col.key, ri);
                  if (col.key === 'tagFormat') openTagFormatMenu(row, $event)
                "
              >
                {{
                  col.key === 'tagFormat'
                    ? cellValue(row, col.key) || DEFAULT_TAG_FORMAT
                    : cellValue(row, col.key)
                }}
              </div>
              <input
                v-else
                class="cell"
                :data-col="col.key"
                :class="{
                  focus: isFocused(row, col.key),
                  'cell-sel': isCellSelected(ri, col.key),
                }"
                :value="row[col.key as ColumnKey]"
                @focus="setFocus(row, col.key, ri)"
                @input="onCellInput(row, col.key as ColumnKey, ($event.target as HTMLInputElement).value)"
              />
            </div>
          </template>
        </div>
      </div>
    </div>

    <RenamePopover
      v-if="showRename"
      v-model:pattern="renamePattern"
      :pending="pendingRenames"
      @close="showRename = false"
      @preview="previewRename"
      @apply="applyRename"
    />

    <ArtworkPopover
      v-if="artPopoverRows"
      :rows="artPopoverRows"
      :preview="artPreview"
      @close="artPopoverRows = null"
      @apply="onArtworkApply"
      @export="exportArt"
    />

    <CellComposePopover
      v-if="composeOpen"
      :col="composeOpen.col"
      :rows="composeOpen.rows"
      :pattern="renamePattern"
      @close="composeOpen = null"
      @apply-field="onComposeField"
      @preview-pattern="onComposePreview"
      @apply-pattern="onComposeApplyPattern"
    />

    <div
      v-if="tagFormatMenu"
      class="tf-scrim"
      @mousedown.self="tagFormatMenu = null"
    >
      <div
        class="tf-menu"
        role="menu"
        :style="{ left: `${tagFormatMenu.x}px`, top: `${tagFormatMenu.y}px` }"
      >
        <button
          v-for="opt in TAG_FORMAT_OPTIONS"
          :key="opt"
          type="button"
          role="menuitemradio"
          :aria-checked="(tagFormatMenu.current || DEFAULT_TAG_FORMAT) === opt"
          class="tf-item"
          :class="{ checked: (tagFormatMenu.current || DEFAULT_TAG_FORMAT) === opt }"
          :title="TAG_FORMAT_HINTS[opt]"
          @click="applyTagFormat(opt)"
        >
          <span class="tf-check" aria-hidden="true">{{
            (tagFormatMenu.current || DEFAULT_TAG_FORMAT) === opt ? '✓' : ''
          }}</span>
          <span class="tf-label">{{ opt }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pane {
  display: flex;
  flex-direction: column;
  flex: 1 1 0;
  min-height: 0;
  overflow: hidden;
  background: var(--te-bg-pane, #2a2a2a);
  position: relative;
  color: var(--te-text, #f0f0f0);
}
.pane.drop-active {
  box-shadow: inset 0 0 0 2px #39d353;
}
.drop-hint {
  position: absolute;
  top: 88px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 20;
  padding: 8px 16px;
  background: rgba(20, 20, 20, 0.92);
  border: 1px solid #39d353;
  border-radius: 6px;
  color: #9ef0b0;
  font-size: 12px;
  pointer-events: none;
}
.pane.busy {
  cursor: progress;
}
.title-rail {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  min-height: var(--te-title-rail-h, 24px);
  padding: 4px 12px 3px;
  font-size: 13px;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: #f0f0f0;
  background: var(--te-bg-title, #3a3a3a);
  border-bottom: none;
}
.find-bar {
  padding: 4px 6px 5px;
  position: relative;
  z-index: 4;
  flex-shrink: 0;
  background: var(--te-bg-toolbar-bottom, #323232);
  box-shadow: inset 0 1px 0 rgba(0, 0, 0, 0.25);
}
.find-shell {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: var(--te-find-h, 28px);
  padding: 1px 6px 1px 6px;
  border: 1px solid #1e1e1e;
  border-radius: 3px;
  background: var(--te-bg-find, #2e2e2e);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.04),
    inset 0 1px 2px rgba(0, 0, 0, 0.35);
}
.find-glyph {
  width: 14px;
  height: 14px;
  opacity: 0.55;
  flex-shrink: 0;
  filter: brightness(0) invert(1);
  object-fit: contain;
}
.find-input {
  flex: 1;
  min-width: 0;
  padding: 3px 4px;
  border: none;
  border-radius: 0;
  background: transparent;
  color: #ececec;
  font-size: 12px;
}
.find-input:focus {
  outline: none;
}
.find-trail {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}
.find-nav {
  width: 19px;
  height: 22px;
  padding: 0;
  border: 1px solid #2f2f2f;
  border-radius: 3px;
  background: linear-gradient(180deg, #4a4a4a 0%, #3a3a3a 100%);
  color: #d8d8d8;
  font-size: 12px;
  line-height: 1;
  cursor: default;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);
}
.find-done {
  min-width: 40px;
  height: 22px;
  padding: 0 8px;
  border: 1px solid #2f2f2f;
  border-radius: 3px;
  background: linear-gradient(180deg, #4a4a4a 0%, #3a3a3a 100%);
  color: #d8d8d8;
  font-size: 11px;
  cursor: default;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);
}
.replace-row {
  display: flex;
  gap: 8px;
  padding: 4px 8px 0;
}
.chk {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: #c8c8c8;
  white-space: nowrap;
  height: 22px;
}
.chk input {
  width: 13px;
  height: 13px;
  margin: 0;
  accent-color: #0a84ff;
}
.replace-input {
  width: 140px;
  padding: 5px 8px;
  border: 1px solid #444;
  border-radius: 3px;
  background: #1a1a1a;
  color: inherit;
  font-size: 12px;
}
.tiny {
  padding: 5px 10px;
  border: 1px solid #555;
  border-radius: 4px;
  background: #404040;
  color: inherit;
  font-size: 12px;
  cursor: pointer;
}
.grid-wrap {
  flex: 1 1 0;
  min-height: 0;
  overflow: auto;
  background: var(--te-bg-grid, #2e2e2e);
}
.sheet {
  width: 100%;
  min-width: 100%;
  box-sizing: border-box;
}
.grid {
  display: grid;
  width: 100%;
  align-items: stretch;
  box-sizing: border-box;
}
.grid.head {
  position: sticky;
  top: 0;
  z-index: 3;
  background: var(--te-bg-head, #373737);
  height: var(--te-grid-head-h, 28px);
  min-height: var(--te-grid-head-h, 28px);
}
.sheet-line {
  border: none;
  border-top: 1px solid var(--te-border-subtle, #1a1a1a);
  margin: 0;
}
.cell-wrap {
  position: relative;
  min-width: 0;
  border-right: 1px solid var(--te-border-grid, #333);
  border-bottom: 1px solid #2a2a2a;
  height: var(--te-grid-row-h, 20px);
  display: flex;
  align-items: stretch;
}
.cell-wrap .cell {
  border: none;
  border-bottom: none;
  border-right: none;
  height: 100%;
}
.cell-wrap.align-center .cell,
.head-cell.align-center {
  text-align: center;
}
.cell {
  padding: 1px 6px;
  border-right: 1px solid var(--te-border-grid, #454545);
  border-bottom: 1px solid #2a2a2a;
  height: var(--te-grid-row-h, 20px);
  font-size: 12px;
  line-height: 18px;
  color: var(--te-text, #f0f0f0);
}
.head-cell {
  position: relative;
  display: flex;
  align-items: center;
  gap: 3px;
  font-weight: 600;
  font-size: 12px;
  color: #e8e8e8;
  user-select: none;
  padding: 0 10px 0 6px;
  height: 100%;
  cursor: default;
  border-right: 1px solid var(--te-border-grid, #454545);
  border-bottom: none;
  background: var(--te-bg-head, #3a3a3a);
}
.head-label {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}
.sort-mark {
  flex-shrink: 0;
  font-size: 8px;
  color: #c8c8c8;
  line-height: 1;
  transform: translateY(-1px);
}
.resize {
  position: absolute;
  top: 0;
  right: 0;
  width: 5px;
  height: 100%;
  cursor: col-resize;
}
.resize:hover {
  background: rgba(90, 143, 212, 0.35);
}
.idx {
  text-align: center;
  color: var(--te-idx-color, #cfcfcf);
  font-size: 11px;
  font-weight: 500;
  border-right: 1px solid var(--te-border-grid, #454545);
  border-bottom: 1px solid #2a2a2a;
  background: var(--te-bg-cell, #343434);
  border-top: none;
  border-left: none;
  cursor: pointer;
  padding: 0;
  height: var(--te-grid-row-h, 20px);
  line-height: var(--te-grid-row-h, 20px);
}
.idx.active {
  background: #3670b8;
  color: #fff;
}
/* Excel: only cells inside the selection rectangle. */
.grid .cell-wrap.cell-sel .cell,
.grid .cell.cell-sel,
.grid .art-btn.cell-sel {
  background: #2a4a78 !important;
}
.grid .cell.focus,
.grid input.cell.focus,
.grid .cell.read.focus {
  background: #1e3a5f !important;
  color: #fff;
}
.file-name.focus.pending {
  color: #ffd08a;
}
.cell.read.tag-format {
  cursor: pointer;
}
.head-idx {
  background: var(--te-bg-head, #3a3a3a);
  height: 100%;
  border-bottom: none;
}
.grid.row .cell:not(.idx),
.grid.row .cell-wrap .cell,
.grid.row .art-btn {
  background: var(--te-bg-cell, #343434);
}
.grid.row.alt .cell:not(.idx),
.grid.row.alt .cell-wrap .cell,
.grid.row.alt .art-btn {
  background: var(--te-bg-cell-alt, #383838);
}
.grid input.cell.focus {
  outline: none;
}
.cell.read.pending:not(.row-sel):not(.focus) {
  color: var(--te-pending, #f0a020);
}
.tf-scrim {
  position: fixed;
  inset: 0;
  z-index: 80;
}
.tf-menu {
  position: fixed;
  min-width: 220px;
  padding: 4px;
  border-radius: 8px;
  background: #343434;
  border: 1px solid rgba(255, 255, 255, 0.12);
  box-shadow: 0 8px 28px rgba(0, 0, 0, 0.45);
  display: flex;
  flex-direction: column;
  gap: 1px;
}
.tf-item {
  display: flex;
  align-items: center;
  gap: 6px;
  border: none;
  background: transparent;
  color: #f0f0f0;
  font: inherit;
  font-size: 12px;
  line-height: 1.3;
  text-align: left;
  padding: 5px 10px 5px 6px;
  border-radius: 5px;
  cursor: pointer;
  white-space: nowrap;
}
.tf-item:hover {
  background: #3d7fd6;
  color: #fff;
}
.tf-check {
  width: 14px;
  flex: 0 0 14px;
  text-align: center;
  font-size: 11px;
}
.tf-label {
  flex: 1;
}
.grid input.cell {
  width: 100%;
  border: none;
  border-radius: 0;
  margin: 0;
  color: inherit;
  font-family: inherit;
  font-size: 12px;
  line-height: 18px;
  background: transparent;
}
.grid input.cell:focus {
  outline: none;
}
.art-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1px;
  border: none;
  cursor: pointer;
  height: var(--te-grid-row-h, 20px);
  border-right: 1px solid var(--te-border-grid, #333);
  border-bottom: 1px solid #2a2a2a;
}
.art-btn img {
  width: 16px;
  height: 16px;
  object-fit: cover;
}
.art-ph {
  font-size: 10px;
  opacity: 0.45;
}
.cell.read {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: default;
  width: 100%;
  box-sizing: border-box;
}
</style>