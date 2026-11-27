import { onUnmounted, type Ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

export const useFileDrop = (active: Ref<boolean>, onPaths: (paths: string[]) => void) => {
  let unlisten: (() => void) | undefined

  const register = async () => {
    unlisten = await getCurrentWindow().onDragDropEvent((event) => {
      const payload = event.payload
      if (payload.type === 'enter' || payload.type === 'over') {
        active.value = true
        return
      }
      if (payload.type === 'leave') {
        active.value = false
        return
      }
      if (payload.type === 'drop') {
        active.value = false
        const paths = payload.paths.filter(Boolean)
        if (paths.length) onPaths(paths)
      }
    })
  }

  void register()

  onUnmounted(() => {
    unlisten?.()
  })
}