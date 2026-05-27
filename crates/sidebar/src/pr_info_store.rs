use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use git::{
    GitHostingProviderRegistry, ParsedGitRemote, PullRequestInfo, parse_git_remote_url,
};
use gpui::{App, AppContext as _, Context, Entity, Global};
use http_client::HttpClient;


const CACHE_TTL: Duration = Duration::from_secs(5 * 60);

struct GlobalPrInfoStore(Entity<PrInfoStore>);
impl Global for GlobalPrInfoStore {}

#[derive(Clone)]
struct CacheEntry {
    pull_requests: Vec<PullRequestInfo>,
    fetched_at: Instant,
}

impl CacheEntry {
    fn is_fresh(&self) -> bool {
        self.fetched_at.elapsed() < CACHE_TTL
    }
}

/// Cache key: (owner/repo, branch).
type CacheKey = (String, String);

pub struct PrInfoStore {
    cache: HashMap<CacheKey, CacheEntry>,
    /// Tracks in-flight fetches so we don't issue duplicate requests.
    pending: HashMap<CacheKey, ()>,
}

impl PrInfoStore {
    pub fn init(cx: &mut App) {
        if cx.has_global::<GlobalPrInfoStore>() {
            return;
        }
        let store = cx.new(|_cx| Self {
            cache: HashMap::new(),
            pending: HashMap::new(),
        });
        cx.set_global(GlobalPrInfoStore(store));
    }

    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalPrInfoStore>().0.clone()
    }

    pub fn try_global(cx: &App) -> Option<Entity<Self>> {
        cx.try_global::<GlobalPrInfoStore>()
            .map(|store| store.0.clone())
    }

    /// Synchronous cache read. Returns cloned PR data if the entry exists and
    /// has not expired.
    pub fn lookup(&self, owner_repo: &str, branch: &str) -> Option<Vec<PullRequestInfo>> {
        let key = (owner_repo.to_string(), branch.to_string());
        self.cache
            .get(&key)
            .filter(|entry| entry.is_fresh())
            .map(|entry| entry.pull_requests.clone())
    }

    /// Parse the remote URL, check the cache, and if stale/missing spawn a
    /// background fetch. Does nothing if the remote URL cannot be parsed or if
    /// a fetch for the same key is already in flight.
    pub fn request_fetch(
        &mut self,
        remote_url: &str,
        branch: &str,
        head_owner: Option<&str>,
        http_client: Arc<dyn HttpClient>,
        cx: &mut Context<Self>,
    ) {
        let Some(provider_registry) = GitHostingProviderRegistry::try_global(cx) else {
            return;
        };

        let Some((provider, parsed_remote)) =
            parse_git_remote_url(provider_registry, remote_url)
        else {
            return;
        };

        let owner_repo = format!("{}/{}", parsed_remote.owner, parsed_remote.repo);
        let key: CacheKey = (owner_repo, branch.to_string());

        if self
            .cache
            .get(&key)
            .is_some_and(|entry| entry.is_fresh())
        {
            return;
        }

        if self.pending.contains_key(&key) {
            return;
        }

        self.pending.insert(key.clone(), ());

        let branch_owned = branch.to_string();
        let head_owner_owned = head_owner.map(|s| s.to_string());
        let remote_for_fetch = ParsedGitRemote {
            owner: parsed_remote.owner.clone(),
            repo: parsed_remote.repo.clone(),
        };

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    provider
                        .pull_requests_for_branch(
                            &remote_for_fetch,
                            &branch_owned,
                            head_owner_owned.as_deref(),
                            http_client,
                        )
                        .await
                })
                .await;

            this.update(cx, |store, cx| {
                store.pending.remove(&key);
                match result {
                    Ok(pull_requests) => {
                        log::info!("PR fetch: got {} PRs for {:?}", pull_requests.len(), key);
                        store.cache.insert(
                            key,
                            CacheEntry {
                                pull_requests,
                                fetched_at: Instant::now(),
                            },
                        );
                        cx.notify();
                    }
                    Err(err) => {
                        log::warn!("PR fetch: failed for {:?}: {err:#}", key);
                    }
                }
            })
            .ok();
        })
        .detach();
    }
}
