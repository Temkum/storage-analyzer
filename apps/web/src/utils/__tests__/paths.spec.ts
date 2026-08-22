import { describe, expect, it } from 'vitest'

import { basename, isDirectChild, normalizePath } from '@/utils/paths'

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

describe('basename', () => {
  it('returns the final path segment', () => {
    expect(basename('/a/b/c.txt')).toBe('c.txt')
    expect(basename('C:\\Users\\me\\report.pdf')).toBe('report.pdf')
    expect(basename('/a/b/')).toBe('b')
  })
})