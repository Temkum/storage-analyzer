import { describe, expect, it } from 'vitest'

import { categoryGlyph, categoryLabel, fileCategory, fileExtension } from '@/utils/fileCategory'

describe('fileExtension', () => {
  it('returns the lowercase extension', () => {
    expect(fileExtension('/a/b/photo.PNG')).toBe('png')
    expect(fileExtension('/a/b/archive.tar.gz')).toBe('gz')
  })

  it('returns empty for files without an extension', () => {
    expect(fileExtension('/a/b/Makefile')).toBe('')
    expect(fileExtension('/a/b/.hidden')).toBe('')
  })
})

describe('fileCategory', () => {
  it('classifies a known document extension', () => {
    expect(fileCategory('/docs/report.pdf')).toBe('document')
  })

  it('classifies image, video and audio', () => {
    expect(fileCategory('/photos/trip.jpg')).toBe('image')
    expect(fileCategory('/movies/clip.mp4')).toBe('video')
    expect(fileCategory('/music/song.mp3')).toBe('audio')
  })

  it('classifies archives and code', () => {
    expect(fileCategory('/dl/bundle.zip')).toBe('archive')
    expect(fileCategory('/app/main.ts')).toBe('code')
  })

  it('returns other for unknown or extensionless paths', () => {
    expect(fileCategory('/misc/blob.xyz')).toBe('other')
    expect(fileCategory('/misc/README')).toBe('other')
  })
})

describe('category label and glyph', () => {
  it('returns sensible text and glyph for a category', () => {
    expect(categoryLabel('image')).toBe('Image')
    expect(categoryGlyph('image')).toBe('🖼️')
  })

  it('falls back for unknown categories', () => {
    expect(categoryLabel('nope' as never)).toBe('Other')
    expect(categoryGlyph('nope' as never)).toBe('📎')
  })
})
