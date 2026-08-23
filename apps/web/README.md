# system-analyzer-web

Vue 3 frontend for **System Analyzer** — the Vue/TS layer of the
cross-platform Disk Analyzer desktop application.

> This is a sub-package of the monorepo. For full installation, building,
> and contribution instructions, see the **[root README](../../README.md)**.

## Local development

```sh
pnpm install
pnpm dev              # Vite dev server
pnpm tauri:dev        # full desktop app (Linux)
pnpm type-check       # vue-tsc
pnpm lint             # eslint + oxlint
pnpm test:unit        # vitest
pnpm build            # production build
```

## Where the frontend lives in the system

See the [Architecture section in the root README](../../README.md#architecture)
for how `apps/web` (Vue) → `apps/desktop` (Tauri) → `cpp/` (engine) connect.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Vue (Official)](https://marketplace.visualstudio.com/items?itemName=Vue.volar)
(and disable Vetur).

## Recommended Browser Setup

- Chromium-based browsers (Chrome, Edge, Brave, etc.):
  - [Vue.js devtools](https://chromewebstore.google.com/detail/vuejs-devtools/nhdogjmejiglipccpnnanbledajbpd)
  - [Turn on Custom Object Formatter in Chrome DevTools](http://bit.ly/object-formatters)
- Firefox:
  - [Vue.js devtools](https://addons.mozilla.org/en-US/firefox/addon/vue-js-devtools/)
  - [Turn on Custom Object Formatter in Firefox DevTools](https://fxdx.dev/firefox-devtools-custom-object-formatters/)

## Type Support for `.vue` Imports in TS

TypeScript cannot handle type information for `.vue` imports by default, so we
replace the `tsc` CLI with `vue-tsc` for type checking. In editors, we need
[Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) to make
the TypeScript language service aware of `.vue` types.
