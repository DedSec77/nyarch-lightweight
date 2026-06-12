# nyarch-lightweight

A **lightweight** desktop shell for [nyarch](https://github.com/DedSec77/nyarch).

Unlike the full [`nyarch-client`](https://github.com/DedSec77/nyarch-client),
this build does **not** bundle the React frontend. It opens the **live website**
in a native WebView window. The result:

- **Tiny** installer/binary (no embedded frontend).
- **Always up to date** — shows the latest deployed site, no rebuild needed.
- **Background tray** — closing the window hides it to the system tray so push
  notifications keep working.

## How the site URL is configured

The URL is baked into the Rust binary at compile time from the `NYARCH_URL`
environment variable (falls back to the constant in `src-tauri/src/lib.rs`).

```bash
# local build
export NYARCH_URL="https://your-real-site"
npm install
npm run tauri build
```

In CI, set `NYARCH_URL` as a GitHub Actions repo **variable** (or secret):
Settings → Secrets and variables → Actions → Variables → `NYARCH_URL`.

## Releasing

Push a tag to trigger the build workflow:

```bash
git tag v0.1.0 && git push origin v0.1.0
```

GitHub Actions builds and attaches to a draft release:

- **Windows:** `.msi` + `.exe`
- **Linux:** `.deb` + `.AppImage` + raw binary

## Why is the WebView still WebKitGTK on Linux?

Both this and the full client use the system WebView (WebKitGTK on Linux,
WebView2 on Windows). On Linux we set `WEBKIT_DISABLE_DMABUF_RENDERER=1` and
`WEBKIT_DISABLE_COMPOSITING_MODE=1` by default to avoid crashes on wlroots
compositors and reduce scroll/animation stutter. Override them via real env
vars if your GPU handles compositing well.
