export type FileCategory =
  'document' | 'image' | 'video' | 'audio' | 'archive' | 'code' | 'text' | 'disk' | 'other'

interface CategoryConfig {
  label: string
  glyph: string
  extensions: readonly string[]
}

const CATEGORIES: Record<FileCategory, CategoryConfig> = {
  document: {
    label: 'Document',
    glyph: '📄',
    extensions: [
      'pdf',
      'doc',
      'docx',
      'odt',
      'rtf',
      'ppt',
      'pptx',
      'xls',
      'xlsx',
      'csv',
      'md',
      'epub',
      'tex',
    ],
  },
  image: {
    label: 'Image',
    glyph: '🖼️',
    extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'bmp', 'tiff', 'ico', 'heic'],
  },
  video: {
    label: 'Video',
    glyph: '🎬',
    extensions: ['mp4', 'mkv', 'mov', 'avi', 'webm', 'flv', 'wmv', 'm4v'],
  },
  audio: {
    label: 'Audio',
    glyph: '🎵',
    extensions: ['mp3', 'wav', 'flac', 'aac', 'ogg', 'opus', 'm4a', 'wma'],
  },
  archive: {
    label: 'Archive',
    glyph: '📦',
    extensions: ['zip', 'tar', 'gz', 'bz2', 'xz', '7z', 'rar', 'tgz'],
  },
  code: {
    label: 'Code',
    glyph: '📝',
    extensions: [
      'js',
      'ts',
      'tsx',
      'jsx',
      'vue',
      'html',
      'css',
      'scss',
      'py',
      'rs',
      'c',
      'cpp',
      'h',
      'hpp',
      'java',
      'go',
      'rb',
      'php',
      'sh',
      'json',
      'yml',
      'yaml',
      'toml',
      'xml',
    ],
  },
  text: {
    label: 'Text',
    glyph: '📃',
    extensions: ['txt', 'log', 'ini', 'cfg', 'conf'],
  },
  disk: {
    label: 'Disk image',
    glyph: '💿',
    extensions: ['iso', 'img'],
  },
  other: {
    label: 'Other',
    glyph: '📎',
    extensions: [],
  },
}

export function fileExtension(path: string): string {
  const filename = path.split('/').pop() ?? ''
  const dotIndex = filename.lastIndexOf('.')

  if (dotIndex <= 0) {
    return ''
  }

  return filename.slice(dotIndex + 1).toLowerCase()
}

export function fileCategory(path: string): FileCategory {
  const extension = fileExtension(path)

  if (!extension) {
    return 'other'
  }

  for (const key of Object.keys(CATEGORIES) as FileCategory[]) {
    if (CATEGORIES[key]?.extensions.includes(extension)) {
      return key
    }
  }

  return 'other'
}

export function categoryLabel(category: FileCategory): string {
  return CATEGORIES[category]?.label ?? 'Other'
}

export function categoryGlyph(category: FileCategory): string {
  return CATEGORIES[category]?.glyph ?? '📎'
}
