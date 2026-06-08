// Ambient types for the vite-plugin-pwa virtual modules. `…/solid` types the
// `useRegisterSW` hook used by PwaUpdater.tsx; `…/info` types the build-time
// PWA metadata. These modules only exist at build time (injected by the
// plugin), so they need an explicit reference for tsc.
/// <reference types="vite-plugin-pwa/solid" />
/// <reference types="vite-plugin-pwa/info" />
