export const parseColWidth = (width: string) => {
  const px = width.match(/^(\d+(?:\.\d+)?)px$/)
  if (px) return Math.max(48, Math.round(parseFloat(px[1])))
  const min = width.match(/minmax\((\d+)px/)
  if (min) return Math.max(48, parseInt(min[1], 10))
  return 120
}

export const formatColWidth = (px: number) => `${Math.max(48, Math.round(px))}px`