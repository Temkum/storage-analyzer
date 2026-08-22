import { describe, expect, it } from 'vitest'

import { basename, isDirectChild, normalizePath, relativePath } from '@/utils/paths'

describe('normalizePath', () => {
  it('converts windows separators and strips trailing slashes', () => {
    expect(normalizePath('C:\\Users\\me\\')).toBe('C:/Users/me')
    expect(normalizePath('/home/user//')).toBe('/home/user')
  })
})

describe('isDirectChild', () => {
  const root = '/tmp/system-analyzer-test'

  it('excludes the root itself', () => {
    expect(isDirectChild(root, root)).toBe(false)
  })

  it('includes immediate children only', () => {
    expect(isDirectChild(`${root}/Documents`, root)).toBe(true)
  })

  it('excludes nested descendants (they are double-counted parents)', () => {
    expect(isDirectChild(`${root}/Documents/Work`, root)).toBe(false)
  })

  it('excludes paths that merely share a prefix', () => {
    expect(isDirectChild(`${root}-extra`, root)).toBe(false)
  })
})

describe('relativePath', () => {
  const root = '/tmp/system-analyzer-test'

  it('returns the segment beneath the root', () => {
    expect(relativePath(`${root}/Documents/report.pdf`, root)).toBe('Documents/report.pdf')
  })

  it('keeps the root itself unchanged', () => {
    expect(relativePath(`${root}`, root)).toBe('/tmp/system-analyzer-test')
  })

  it('returns the normalized path when outside the root', () => {
    expect(relativePath('/elsewhere/file.txt', root)).toBe('/elsewhere/file.txt')
  })
})

describe('basename', () => {
  it('returns the final path segment', () => {
    expect(basename('/a/b/c.txt')).toBe('c.txt')
    expect(basename('C:\\Users\\me\\report.pdf')).toBe('report.pdf')
    expect(basename('/a/b/')).toBe('b')
  })
})
