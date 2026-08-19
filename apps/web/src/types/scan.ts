export enum FileType {
  File = 0,
  Directory = 1,
  Symlink = 2,
  Other = 3,
}

export interface FileEntry {
  path: string
  type: FileType
  size: number
}

export interface DirectorySize {
  path: string
  size: number
}

export interface ScanResult {
  entries: FileEntry[]
  directories: DirectorySize[]
}
