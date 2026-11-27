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
  width: var(--te-toolbar-glyph, 28px);
  height: var(--te-toolbar-glyph, 28px);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: linear-gradient(
    165deg,
    rgba(255, 255, 255, 0.22) 0%,
    rgba(255, 255, 255, 0.08) 42%,
    rgba(0, 0, 0, 0.12) 100%
  );
  border: 0.5px solid rgba(255, 255, 255, 0.22);
  backdrop-filter: blur(12px) saturate(1.4);
  -webkit-backdrop-filter: blur(12px) saturate(1.4);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.35),
    inset 0 -0.5px 0 rgba(0, 0, 0, 0.2),
    0 1px 2px rgba(0, 0, 0, 0.22);
  transition:
    background 120ms ease-out,
    box-shadow 120ms ease-out,
    border-color 120ms ease-out,
    transform 80ms ease-out;
}
.glyph-wrap :deep(.te-icon) {
  width: 28px;
  height: 28px;
  object-fit: contain;
  transform: scale(var(--te-toolbar-icon-scale, 0.58));
  filter: brightness(0) invert(1);
  opacity: 0.95;
  pointer-events: none;
}
.menu-mark {
  position: absolute;
  right: 0;
  bottom: 0;
  width: 0;
  height: 0;
  border-style: solid;
  border-width: 0 0 5px 5px;
  border-color: transparent transparent rgba(240, 240, 240, 0.9) transparent;
  filter: drop-shadow(0 0 0.5px rgba(0, 0, 0, 0.5));
}
.tb:not(:disabled):hover .glyph-wrap {
  background: linear-gradient(
    165deg,
    rgba(255, 255, 255, 0.32) 0%,
    rgba(255, 255, 255, 0.12) 45%,
    rgba(0, 0, 0, 0.1) 100%
  );
  border-color: rgba(255, 255, 255, 0.32);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.45),
    inset 0 -0.5px 0 rgba(0, 0, 0, 0.15),
    0 1px 3px rgba(0, 0, 0, 0.28);
}
.tb:not(:disabled):active .glyph-wrap {
  transform: scale(0.94);
  background: linear-gradient(
    165deg,
    rgba(255, 255, 255, 0.1) 0%,
    rgba(0, 0, 0, 0.22) 100%
  );
  border-color: rgba(255, 255, 255, 0.12);
  box-shadow:
    inset 0 1px 4px rgba(0, 0, 0, 0.45),
    inset 0 1px 0 rgba(255, 255, 255, 0.1),
    0 0.5px 0 rgba(255, 255, 255, 0.06);
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
@media (prefers-reduced-motion: reduce) {
  .glyph-wrap {
    transition: none;
  }
  .tb:not(:disabled):active .glyph-wrap {
    transform: none;
  }
}
@media (prefers-reduced-transparency: reduce) {
  .glyph-wrap {
    background: #555;
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
  }
  .tb:not(:disabled):hover .glyph-wrap {
    background: #636363;
  }
  .tb:not(:disabled):active .glyph-wrap {
    background: #404040;
  }
}
</style>