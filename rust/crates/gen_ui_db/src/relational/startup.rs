// TJ-ARCH-MOB-001 compliant
//! Typestate startup orchestration: migrations -> seeds -> sync attach.

use std::{marker::PhantomData, sync::Arc};

use gen_ui_types::sync::SyncTransport;

use super::{RelationalResult, SeedBundle};

pub struct Uninitialized;
pub struct Migrated;
pub struct Ready;

#[async_trait::async_trait]
pub trait StartupStore: Send + Sync {
    async fn migrate(&self) -> RelationalResult<()>;
    async fn execute_seed(&self, sql: &str) -> RelationalResult<()>;
}

pub struct Startup<S, State> {
    store: S,
    http: reqwest::Client,
    _state: PhantomData<State>,
}

impl<S> Startup<S, Uninitialized>
where
    S: StartupStore,
{
    pub fn new(store: S) -> Self {
        Self {
            store,
            http: reqwest::Client::new(),
            _state: PhantomData,
        }
    }

    pub async fn migrate(self) -> RelationalResult<Startup<S, Migrated>> {
        self.store.migrate().await?;
        Ok(Startup {
            store: self.store,
            http: self.http,
            _state: PhantomData,
        })
    }
}

impl<S> Startup<S, Migrated>
where
    S: StartupStore,
{
    pub async fn seed_and_attach(
        self,
        bundles: &[SeedBundle],
        sync: Arc<dyn SyncTransport>,
    ) -> RelationalResult<Startup<S, Ready>> {
        for bundle in bundles {
            let sql = bundle.sql(&self.http).await?;
            self.store.execute_seed(&sql).await?;
        }
        sync.start()
            .await
            .map_err(|error| super::RelationalError::Sync(error.to_string()))?;
        Ok(Startup {
            store: self.store,
            http: self.http,
            _state: PhantomData,
        })
    }
}

impl<S> Startup<S, Ready> {
    pub fn into_store(self) -> S {
        self.store
    }
}
