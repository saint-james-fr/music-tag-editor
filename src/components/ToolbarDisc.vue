<script setup lang="ts">
const props = defineProps<{
  label: string
  disabled?: boolean
  dimmed?: boolean
  title?: string
  hasMenu?: boolean
  /** Toolbar cell width (pt). */
  widthPx?: number
}>()

const emit = defineEmits<{ click: [MouseEvent] }>()
</script>

<template>
  <button
    type="button"
    class="tb"
    :disabled="disabled && !hasMenu"
    :class="{ dimmed: dimmed || (disabled && hasMenu) }"
    :style="props.widthPx ? { width: `${props.widthPx}px` } : undefined"
    :title="title ?? label"
    @click="emit('click', $event)"
  >
    <span class="glyph-wrap">
      <slot />
      <span v-if="hasMenu" class="menu-mark" aria-hidden="true" />
    </span>
    <span class="lb">{{ label }}</span>
  </button>
</template>

<style scoped>
.tb {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  gap: 2px;
  width: var(--te-toolbar-item-w, 46px);
  min-height: var(--te-toolbar-item-h, 51px);
  padding: 2px 0 1px;
  border: none;
  background: transparent;
  cursor: default;
  -webkit-font-smoothing: antialiased;
  flex-shrink: 0;
}
.tb:disabled,
.tb.dimmed {
  opacity: 0.38;
  cursor: default;
}
.tb.dimmed:not(:disabled) {
  opacity: 0.48;
}
.glyph-wrap {
  position: relative;
  width: var(--te-toolbar-glyph, 22px);
  height: var(--te-toolbar-glyph, 22px);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: center / 100% 100% no-repeat url('/icons/disc-bg.png');
  box-shadow:
    0 1px 1px rgba(0, 0, 0, 0.45),
    inset 0 1px 0 rgba(255, 255, 255, 0.12);
}
.glyph-wrap :deep(.te-icon) {
  width: 28px;
  height: 28px;
  object-fit: contain;
  transform: scale(var(--te-toolbar-icon-scale, 0.52));
  filter: brightness(0) invert(1);
  opacity: 1;
  pointer-events: none;
}
.menu-mark {
  position: absolute;
  right: -1px;
  bottom: -1px;
  width: 0;
  height: 0;
  border-style: solid;
  border-width: 0 0 4px 4px;
  border-color: transparent transparent rgba(230, 230, 230, 0.85) transparent;
}
.tb:not(:disabled):hover .glyph-wrap {
  background: radial-gradient(circle at 36% 28%, #777 0%, #4e4e4e 55%, #414141 100%);
}
.tb:not(:disabled):active .glyph-wrap {
  transform: scale(0.97);
  box-shadow:
    0 1px 0 rgba(0, 0, 0, 0.35),
    inset 0 1px 3px rgba(0, 0, 0, 0.45);
}
.lb {
  font-size: var(--te-toolbar-label, 10px);
  line-height: 1.05;
  color: var(--te-label-muted);
  text-align: center;
  letter-spacing: 0;
  max-width: 100%;
  padding: 0 1px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.tb:not(:disabled):hover .lb {
  color: rgba(255, 255, 255, 0.72);
}
</style>