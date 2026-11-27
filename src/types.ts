export type TrackRow = {
  id: string
  path: string
  fileName: string
  title: string
  artist: string
  album: string
  genre: string
  year: string
  albumArtist: string
  trackNumber: string
  tracksTotal: string
  comment: string
  tagFormat: string
  dirty: boolean
  hasArtwork: boolean
  pendingName?: string | null
  selected?: boolean
}

export type ColumnKey = keyof Pick<
  TrackRow,
  | 'title'
  | 'artist'
  | 'album'
  | 'genre'
  | 'year'
  | 'albumArtist'
  | 'trackNumber'
  | 'tracksTotal'
  | 'comment'
  | 'fileName'
  | 'tagFormat'
>

export type ColumnId = ColumnKey | 'artwork'

/** Default column widths. */
export const COLUMN_META: {
  key: ColumnId
  label: string
  defaultWidth: string
  align?: 'left' | 'center'
  readonly?: boolean
}[] = [
  { key: 'artwork', label: 'Artwork', defaultWidth: '80px', align: 'center' },
  { key: 'title', label: 'Title', defaultWidth: '127px' },
  { key: 'artist', label: 'Artist', defaultWidth: '114px' },
  { key: 'album', label: 'Album', defaultWidth: '134px' },
  { key: 'genre', label: 'Genre', defaultWidth: '109px' },
  { key: 'year', label: 'Year', defaultWidth: '65px', align: 'center' },
  { key: 'albumArtist', label: 'Album Artist', defaultWidth: '105px' },
  { key: 'trackNumber', label: 'Track #', defaultWidth: '75px', align: 'center' },
  { key: 'tracksTotal', label: 'Tracks Total', defaultWidth: '100px', align: 'center' },
  { key: 'comment', label: 'Comment', defaultWidth: '90px' },
  { key: 'fileName', label: 'File Name', defaultWidth: '250px' },
  { key: 'tagFormat', label: 'Tag Format', defaultWidth: '140px', readonly: true },
]

/** Tag format labels — ID3v2.3.0 is the most common default. */
export const TAG_FORMAT_OPTIONS = [
  'ID3v2.3.0',
  'ID3v2.4.0',
  'ID3v2.3.0, Info',
  'ID3v2.4.0, Info',
] as const

export const DEFAULT_TAG_FORMAT = TAG_FORMAT_OPTIONS[0]

/** Hover hints — Info = RIFF INFO (Rekordbox WAV). */
export const TAG_FORMAT_HINTS: Record<string, string> = {
  'ID3v2.3.0': 'ID3v2.3 only — widest player support',
  'ID3v2.4.0': 'ID3v2.4 only',
  'ID3v2.3.0, Info': 'Dual-write ID3v2.3 + RIFF INFO (Rekordbox WAV)',
  'ID3v2.4.0, Info': 'Dual-write ID3v2.4 + RIFF INFO (Rekordbox WAV)',
}

export const DEFAULT_COLUMN_LAYOUT = COLUMN_META.map((c) => ({
  key: c.key,
  visible: true,
  width: c.defaultWidth,
}))

export type ArtworkChange = {
  path: string
  action: 'set' | 'remove'
  dataBase64?: string
  mime?: string
}

export type ArtworkPayload = {
  mime: string
  dataBase64: string
}
