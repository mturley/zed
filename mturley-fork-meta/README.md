# mturley's Zed Fork — "Zed Extended"

This folder tracks the purpose and design of changes made in this fork of Zed, making them easier to understand and retain across rebases onto upstream.

Each feature has its own markdown file with enough detail to reimplement it from scratch if a rebase destroys the changes.

## Fork Identity

The fork is renamed to "Zed Extended" with a separate macOS bundle identifier (`dev.zed.Zed-Extended-*`) and separate data directories (`~/Library/Application Support/Zed Extended/` on macOS). This allows it to coexist alongside upstream Zed.

### Files changed for the rename
- `crates/paths/src/paths.rs` — `APP_NAME` → `"Zed Extended"`
- `crates/zed/src/main.rs` — Relaxed binary name assertion (binary stays `zed` but APP_NAME has a space)
- `crates/zed/Cargo.toml` — Bundle names/identifiers/URL schemes
- `crates/zed/src/zed/app_menus.rs` — Menu labels (About, Hide, Quit)

## Features

1. **Message preview subtitle** — shows the first message of each thread in the sidebar (#2)
2. **PR linking in sidebar** — see `pr-linking.md` (#4)
