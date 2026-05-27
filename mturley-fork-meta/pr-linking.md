# PR Linking in Thread Sidebar

## What it does

When an agent or terminal thread is associated with a worktree whose branch has a pull request on GitHub, shows a PR row in the sidebar thread entry with a rich hover tooltip. Clicking opens the PR in the browser.

### PR Row (sidebar)

- Own row in thread entry, between message preview and worktree/timestamp metadata
- Shows: state-specific icon (open/merged/closed), `#number` in state color (blue/purple/red), PR title snippet (truncated), author initials
- Clickable — opens PR URL in default browser
- Shown for both agent threads and terminal threads
- Supports open, closed, and merged PRs (uses `state=all` in API query)

### Hover Tooltip (GitHub-style)

- Repo name + "on [date]" (e.g. "opendatahub-io/odh-dashboard on May 6")
- Full PR title with `#number` (wrapping)
- Author avatar (loaded from GitHub avatar URL) + username + state icon + state label
- State colors: green (open), red (closed), purple (merged), muted (draft)
- Description (truncated to 200 chars, newlines collapsed)
- Base ← head branch names in accent color
- Labels as colored chip/pill elements (hex color from GitHub parsed to border/background)

### PR Lookup Flow

The lookup happens at **render time** in `render_thread()` and `render_terminal()`, not during `rebuild_contents()`. This is because thread entries may not be fully loaded (drafts filtered by `retain`) during `rebuild_contents`, but are always available when rendered.

1. For each thread's worktree path, find the matching repository snapshot
2. Determine the branch: use `snapshot.branch` for main worktree, `wt.branch_name()` for linked worktrees
3. Resolve upstream tracking: look up the branch in `snapshot.branch_list` to find its `upstream` tracking info. Use the upstream branch name (not the local branch name) for the API query
4. Derive the fork owner: if the tracking remote is "origin" or "upstream", parse the owner from `remote_origin_url`. Otherwise, use the remote name as the GitHub username (heuristic — works for the common fork workflow)
5. Check the `PrInfoStore` cache. If cached and fresh, return the PR data
6. If not cached, trigger a background fetch via `PrInfoStore::request_fetch()`
7. When fetch completes, `PrInfoStore` notifies observers → sidebar rebuilds → render picks up cached data

### Authentication

- Uses `GITHUB_TOKEN` env var if present (5000 req/hour)
- Falls back to unauthenticated GitHub API (60 req/hour)
- Matches existing Zed pattern: `.when_some(std::env::var("GITHUB_TOKEN").ok(), ...)`
- Private repos require a token; without one, PR row silently not shown

### Data Freshness

- Cache TTL: 5 minutes per (owner/repo, branch) pair
- Duplicate request prevention via `pending` set in `PrInfoStore`
- Sidebar observes `PrInfoStore` — rebuilds when new PR data arrives

## Files changed

### `crates/git/src/hosting_provider.rs`
- Added `PullRequestInfo`, `PrState`, `CiStatus`, `PrLabel` types
- Added `pull_requests_for_branch()` method to `GitHostingProvider` trait with `head_owner` parameter

### `crates/git_hosting_providers/src/providers/github.rs`
- Implemented `pull_requests_for_branch()` for GitHub
- API: `GET /repos/{owner}/{repo}/pulls?state=all&head={fork_owner}:{branch}&per_page=5`
- Parses response into `PullRequestInfo` with labels, author avatar URL, etc.

### `crates/icons/src/icons.rs`
- Added `PullRequestMerged` and `PullRequestClosed` icon variants

### `assets/icons/pull_request_merged.svg` and `pull_request_closed.svg`
- Custom SVG icons for merged and closed PR states

### `crates/ui/src/components/ai/thread_item.rs`
- Added `ThreadItemPrInfo`, `ThreadItemPrLabel`, `ThreadItemCiStatus` display types
- Added `pr_info` field and builder method on `ThreadItem`
- PR row rendering with state-specific icons and colors
- Rich tooltip with avatar, labels, branches, description
- `parse_hex_color()` helper for label chip colors

### `crates/sidebar/src/sidebar.rs`
- `lookup_pr_info_for_worktree_paths()` — render-time PR data lookup with upstream tracking branch resolution
- `trigger_pr_fetches()` — proactive fetch for threads missing PR data
- `to_thread_item_pr_info()` — converts `PullRequestInfo` to `ThreadItemPrInfo`
- `lookup_pr_info()` — cache lookup helper
- PR row wired into both `render_thread()` and `render_terminal()`
- `PrInfoStore` initialized and observed in `Sidebar::new()`

### `crates/sidebar/src/pr_info_store.rs` (new file)
- Global GPUI entity caching PR data keyed by (owner/repo, branch)
- `request_fetch()` — spawns background GitHub API call, updates cache, notifies observers
- 5-minute cache TTL, duplicate request prevention

### `crates/sidebar/Cargo.toml`
- Added `client` and `http_client` dependencies

## How to verify

1. Set `GITHUB_TOKEN` env var (optional but recommended for private repos)
2. Build with `script/bundle-mac -do`
3. Open a project with a git repo that has a branch with a PR on GitHub
4. Start an agent thread or terminal thread in that project
5. Sidebar should show PR row with state-specific icon, number, title, author initials
6. Hover PR row → tooltip with avatar, title, state, description, branches, labels
7. Click PR row → opens PR in browser
8. Merged PRs show purple icon/number, closed show red, open show blue
9. Thread without a PR branch → no PR row shown
10. Linked worktrees with tracking branches → PR resolved via upstream branch name
