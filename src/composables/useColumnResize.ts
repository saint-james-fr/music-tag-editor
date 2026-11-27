import type { ColumnId } from '../types'
import { parseColWidth } from '../column-width'

export const startColumnResize = (
  _key: ColumnId,
  startWidth: string,
  startX: number,
  onWidth: (px: number) => void,
  pointerId?: number,
  target?: HTMLElement,
) => {
  const base = parseColWidth(startWidth)
  const min = 48
  const max = 720

  const apply = (clientX: number) => {
    const raw = base + (clientX - startX)
    let next = Math.round(raw)
    if (next < min) {
      const overshoot = min - next
      next = Math.round(min - Math.min(overshoot * 0.35, 20))
    } else if (next > max) {
      const overshoot = next - max
      next = Math.round(max + Math.min(overshoot * 0.35, 20))
    }
    onWidth(Math.max(min, Math.min(max, next)))
  }

  const onMove = (e: MouseEvent | PointerEvent) => apply(e.clientX)

  const onUp = () => {
    document.removeEventListener('mousemove', onMove as EventListener)
    document.removeEventListener('mouseup', onUp as EventListener)
    document.removeEventListener('pointermove', onMove as EventListener)
    document.removeEventListener('pointerup', onUp as EventListener)
    if (target && pointerId != null) {
      try {
        target.releasePointerCapture(pointerId)
      } catch {
        /* ignore */
      }
    }
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
  }

  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  if (target && pointerId != null && 'setPointerCapture' in target) {
    target.setPointerCapture(pointerId)
    document.addEventListener('pointermove', onMove)
    document.addEventListener('pointerup', onUp)
  } else {
    document.addEventListener('mousemove', onMove)
    document.addEventListener('mouseup', onUp)
  }
}