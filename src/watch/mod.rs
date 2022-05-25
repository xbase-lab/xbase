mod event;
mod serialize;

pub use event::{Event, EventKind};

use crate::{client::Client, constants::DAEMON_STATE, state::State, Result};
use async_trait::async_trait;
use notify::{Config, RecommendedWatcher, RecursiveMode::Recursive, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::sync::mpsc::channel;
use tokio::{sync::MutexGuard, task::JoinHandle};
use tracing::{debug, error, info, trace};

#[derive(derive_deref_rs::Deref)]
pub struct WatchService {
    #[deref]
    pub listeners: HashMap<String, Box<(dyn Watchable + Send + Sync + 'static)>>,
    pub handler: JoinHandle<Result<()>>,
}

pub struct InternalState {
    debounce: Arc<Mutex<SystemTime>>,
    last_path: Arc<Mutex<PathBuf>>,
}

/// Trait to make an object react to filesystem changes.
///
/// ToString is required in order to store watchable in HashMap
#[async_trait]
#[cfg(feature = "daemon")]
pub trait Watchable: ToString + Send + Sync + 'static {
    /// Trigger Restart of Watchable.
    async fn trigger(&self, state: &MutexGuard<State>, event: &Event) -> Result<()>;

    /// A function that controls whether a a Watchable should restart
    async fn should_trigger(&self, state: &MutexGuard<State>, event: &Event) -> bool;

    /// A function that controls whether a watchable should be dropped
    async fn should_discard(&self, state: &MutexGuard<State>, event: &Event) -> bool;

    /// Drop watchable for watching a given file system
    async fn discard(&self, state: &MutexGuard<State>) -> Result<()>;
}

#[cfg(feature = "daemon")]
impl WatchService {
    pub async fn new(client: Client, ignore_pattern: Vec<String>) -> Result<Self> {
        let listeners = Default::default();

        async fn try_to_recompile<'a>(
            event: &Event,
            client: &Client,
            state: &mut MutexGuard<'a, State>,
        ) {
            let recompile = event.is_create_event()
                || event.is_remove_event()
                || (event.is_content_update_event() && event.file_name().eq("project.yml"))
                || event.is_rename_event() && !(event.path().exists() || event.is_seen());

            if recompile {
                let ref name = client.abbrev_root();

                client.echo_msg(state, name, "recompiling ..").await;

                let ensure = client.ensure_server_support(state, Some(event.path()));

                if let Err(e) = ensure.await {
                    let ref msg = format!("Fail to recompile {e}");
                    client.echo_err(state, name, msg).await;
                } else {
                    client.echo_msg(state, name, "recompiled").await;
                    debug!("[WatchService] project {name:?} recompiled successfully");
                }
            };
        }

        let handler = tokio::spawn(async move {
            let mut discards = vec![];
            let ref root = client.root;
            let internal_state = InternalState::default();

            let (tx, mut rx) = channel::<notify::Event>(1);
            let mut w = <RecommendedWatcher as Watcher>::new(move |res| {
                if let Ok(event) = res {
                    tx.blocking_send(event).unwrap()
                }
            })?;
            w.watch(&client.root, Recursive)?;
            w.configure(Config::NoticeEvents(true))?;

            let ignore_pattern = ignore_pattern
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>();

            let ignore = wax::any::<wax::Glob, _>(ignore_pattern).unwrap();

            while let Some(event) = rx.recv().await {
                let ref event = match Event::new(&ignore, &internal_state, event) {
                    Some(e) => e,
                    None => continue,
                };

                let state = DAEMON_STATE.clone();
                let ref mut state = state.lock().await;

                try_to_recompile(event, &client, state).await;

                let watcher = match state.watcher.get(root) {
                    Ok(w) => w,
                    Err(err) => {
                        error!(r#"[WatchService] unable to get watcher for {root:?}: {err}"#);
                        info!(r#"[WatchService] dropping watcher for {root:?}: {err}"#);
                        break;
                    }
                };

                for (key, listener) in watcher.listeners.iter() {
                    if listener.should_discard(state, event).await {
                        if let Err(err) = listener.discard(state).await {
                            error!("[WatchService] `{key}` discard errored!: {err}");
                        }
                        discards.push(key.to_string());
                    } else if listener.should_trigger(state, event).await {
                        if let Err(err) = listener.trigger(state, event).await {
                            error!("[WatchService] `{key}` trigger errored!: {err}");
                        }
                    }
                }
                let watcher = state.watcher.get_mut(root).unwrap();

                for key in discards.iter() {
                    info!("[WatchService] remove(\"{key}\")");
                    watcher.listeners.remove(key);
                }

                discards.clear();
                internal_state.update_debounce();

                info!("[WatchService] processed ({event})");
            }

            info!("[WatchService] {:?} dropped", client.root);

            Ok(())
        });

        Ok(Self { handler, listeners })
    }

    pub fn add<W: Watchable>(&mut self, watchable: W) -> Result<()> {
        let key = watchable.to_string();
        info!(r#"[WatchService] add("{key}")"#);

        let other = self.listeners.insert(key, box (watchable));
        if let Some(watchable) = other {
            let key = watchable.to_string();
            error!("[WatchService] Watchable with key `{key}` already exists!")
        }

        Ok(())
    }

    pub fn remove(&mut self, key: &String) -> Result<()> {
        info!("[WatchService] remove `{key}`");
        self.listeners.remove(key);
        Ok(())
    }
}

impl Default for InternalState {
    fn default() -> Self {
        Self {
            debounce: Arc::new(Mutex::new(SystemTime::now())),
            last_path: Default::default(),
        }
    }
}

impl InternalState {
    pub fn update_debounce(&self) {
        let mut debounce = self.debounce.lock().unwrap();
        *debounce = SystemTime::now();
        trace!("[WatchService] debounce updated!");
    }

    pub fn last_run(&self) -> u128 {
        self.debounce.lock().unwrap().elapsed().unwrap().as_millis()
    }

    /// Get a reference to the internal state's last path.
    #[must_use]
    pub fn last_path(&self) -> Arc<Mutex<PathBuf>> {
        self.last_path.clone()
    }
}
