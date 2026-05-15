# mturley's Zed Fork — "Zed Extended"

This folder tracks the purpose and design of changes made in this fork of Zed, making them easier to understand and retain across rebases onto upstream.

## Fork Identity

The fork is renamed to "Zed Extended" with a separate macOS bundle identifier (`dev.zed.Zed-Extended-*`) and separate data directories (`~/Library/Application Support/Zed Extended/` on macOS). This allows it to coexist alongside upstream Zed.

### Files changed for the rename
- `crates/paths/src/paths.rs` — `APP_NAME` → `"Zed Extended"`
- `crates/zed/src/main.rs` — Relaxed binary name assertion (binary stays `zed` but APP_NAME has a space)
- `crates/zed/Cargo.toml` — Bundle names/identifiers/URL schemes
- `crates/zed/src/zed/app_menus.rs` — Menu labels (About, Hide, Quit)

## Goals

1. **Rename/annotate threads** — Custom names or annotations on agent threads that persist, so threads are distinguishable beyond auto-generated titles.

2. **Link threads to PRs** — If a thread's worktree has a branch with an open PR, show a link to that PR in the sidebar with metadata (repo, author, merge status, CI status).

## Architecture Notes

See the plan file for a map of the relevant source files and data flow.
