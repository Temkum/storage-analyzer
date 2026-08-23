# Vue Cheatsheet

Conventions and commands specific to `apps/web` (Vue 3 + TypeScript + Vite).
All commands below assume you're in `apps/web/`.

## Package manager

This project uses **pnpm**, not npm/yarn — the lockfile is
`pnpm-lock.yaml`, and CI installs with `pnpm install --frozen-lockfile`.
Don't run `npm install` here; it'll create a conflicting lockfile.

```bash
pnpm install
```

---

## Running the app

### Frontend only (no Tauri shell, no C++ engine)

```bash
pnpm dev
```

Starts Vite on `http://localhost:5173`. Useful for pure UI work, but
`useScanner`/`analyzer.ts` calls will fail — there's no Tauri `invoke`
backend without the desktop shell running.

### Full desktop app (frontend + Rust shell + C++ sidecar)

```bash
pnpm tauri:dev
```

Requires the C++ engine already built and staged (see
`docs/CPP-CHEATSHEET.md` / `scripts/build-engine.sh`) — this does **not**
rebuild the sidecar for you.

### Production build

```bash
pnpm build          # type-check + build, via run-p
pnpm build-only      # build only, skips type-check (faster iteration)
pnpm preview         # serve the production build locally
```

---

## Type checking

```bash
pnpm type-check
```

Runs `vue-tsc --build`, which type-checks `.vue` SFCs in addition to plain
`.ts` — `tsc` alone won't catch errors inside `<script setup>` blocks.

---

## Linting & formatting

```bash
pnpm lint            # runs both of the below (run-s lint:*)
pnpm lint:oxlint      # oxlint . --fix   (fast, catches obvious issues)
pnpm lint:eslint      # eslint . --fix --cache

pnpm format           # prettier --write --experimental-cli src/
```

`oxlint` runs first and is much faster than ESLint — if you're only
checking for typos/unused vars during a quick loop, `pnpm lint:oxlint` alone
is often enough before falling back to the full `pnpm lint`.

---

## Testing (Vitest)

### Run all unit tests once

```bash
pnpm test:unit
```

### Watch mode (reruns on save)

```bash
pnpm exec vitest
```

### Run a single test file

```bash
pnpm exec vitest run src/utils/__tests__/format.spec.ts
```

### Run tests matching a name pattern

```bash
pnpm exec vitest run -t "formats kilobytes"
```

### Existing test conventions

Tests live in `__tests__/` next to the code they cover
(`src/utils/__tests__/format.spec.ts` tests `src/utils/format.ts`). Style:

```ts
import { describe, expect, it } from 'vitest'
import { formatBytes } from '@/utils/format'

describe('formatBytes', () => {
  it('formats zero as 0 B', () => {
    expect(formatBytes(0)).toBe('0 B')
  })
})
```

The Vitest environment is `jsdom` (set in `vitest.config.ts`), so DOM APIs
are available in tests without extra setup.

---

## Project conventions

### Composition API only, `<script setup lang="ts">`

Every component in this project uses `<script setup>` — no Options API, no
`defineComponent({...})` boilerplate:

```vue
<script setup lang="ts">
import { ref } from 'vue'

const count = ref(0)
</script>
```

### State lives in composables, not a store

There's no Pinia/Vuex in this project — shared state is plain composable
functions returning refs, e.g. `useScanner()`:

```ts
// composables/useScanner.ts
export function useScanner() {
  const result = ref<ScanResult | null>(null)
  const isScanning = ref(false)
  // ...
  return { result, isScanning, scan, cancel }
}
```

```vue
<!-- consumed in App.vue -->
<script setup lang="ts">
import { useScanner } from '@/composables/useScanner'

const { result, isScanning, scan, cancel } = useScanner()
</script>
```

If a future feature genuinely needs cross-tree shared state beyond what a
composable comfortably handles, that's the point to reach for Pinia — it's
not currently a dependency.

### `@/` import alias

`@/` resolves to `src/` (configured in `vite.config.ts` /
`tsconfig.json`). Prefer it over relative `../../` paths:

```ts
import { formatBytes } from '@/utils/format'
import type { ScanResult } from '@/types/scan'
```

### Typed props and emits

```vue
<script setup lang="ts">
defineProps<{
  path: string
  scanning: boolean
}>()

const emit = defineEmits<{
  'update:path': [value: string]
  scan: []
}>()
</script>
```

### Calling into the Tauri backend

All engine calls go through `src/services/analyzer.ts`, which wraps
`@tauri-apps/api/core`'s `invoke`. Don't call `invoke` directly from
components — go through the service layer, and through a composable
(`useScanner`) for anything stateful:

```ts
import { invoke } from '@tauri-apps/api/core'

export async function scanDirectory(path: string): Promise<ScanResult> {
  const raw = await invoke<string>('scan_directory', { path })
  return JSON.parse(raw) as ScanResult
}
```

### Listening for backend events

Scan progress streams from the Rust shell as a Tauri event, not a return
value — subscribe/unsubscribe around the async call:

```ts
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

let unlisten: UnlistenFn | null = null

unlisten = await listen<number>('scan-progress', (event) => {
  scannedEntries.value = event.payload
})

// later
unlisten?.()
```

---

## Debugging

### Vue DevTools

`vite-plugin-vue-devtools` is already wired into `vite.config.ts` — open
the app in a browser (via `pnpm dev`) and the DevTools panel is available
without a separate extension.

### Inspecting Tauri `invoke` calls

When running under `pnpm tauri:dev`, open the webview's dev console (right
click → Inspect, or the Tauri window's built-in devtools) to see
`invoke`/`listen` traffic and any errors surfaced from the Rust side.
