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

export interface ScanError {
  path: string
  message: string
}

export interface ScanResult {
  rootPath: string
  totalSize: number
  fileCount: number
  directoryCount: number
  entries: FileEntry[]
  directories: DirectorySize[]
  durationMs: number
  errors: ScanError[]
}
