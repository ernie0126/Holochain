//! kdirect persist type

use crate::*;
use futures::future::BoxFuture;
use kitsune_p2p::event::MetricDatum;
use kitsune_p2p::event::MetricQuery;
use kitsune_p2p::event::MetricQueryAnswer;
use kitsune_p2p::KitsuneAgent;
use kitsune_p2p_types::dht_arc::DhtArc;
use kitsune_p2p_types::tls::TlsConfig;
use std::future::Future;
use types::kdagent::*;
use types::kdentry::KdEntry;
use types::kdhash::KdHash;

/// Trait representing a persistence store.
pub trait AsKdPersist: 'static + Send + Sync {
    /// Get a uniq val that assists with Eq/Hash of trait objects.
    fn uniq(&self) -> Uniq;

    /// Check if this persist instance has been closed
    fn is_closed(&self) -> bool;

    /// Explicitly close this persist instance
    fn close(&self) -> BoxFuture<'static, ()>;

    /// Get or create and get the singleton tls cert creds for this store.
    fn singleton_tls_config(&self) -> BoxFuture<'static, KitsuneResult<TlsConfig>>;

    /// Generate a signature keypair, returning the pub key as a KdHash.
    fn generate_signing_keypair(&self) -> BoxFuture<'static, KitsuneResult<KdHash>>;

    /// Sign arbitrary data with the secret key associated with given KdHash.
    fn sign(
        &self,
        pub_key: KdHash,
        data: &[u8],
    ) -> BoxFuture<'static, KitsuneResult<Arc<[u8; 64]>>>;

    /// Store agent info
    fn store_agent_info(&self, agent_info: KdAgentInfo) -> BoxFuture<'static, KitsuneResult<()>>;

    /// Get agent info
    fn get_agent_info(
        &self,
        root: KdHash,
        agent: KdHash,
    ) -> BoxFuture<'static, KitsuneResult<KdAgentInfo>>;

    /// Query agent info
    fn query_agent_info(&self, root: KdHash)
        -> BoxFuture<'static, KitsuneResult<Vec<KdAgentInfo>>>;

    /// Store agent info
    fn put_metric_datum(
        &self,
        agent: KitsuneAgent,
        datum: MetricDatum,
    ) -> BoxFuture<'static, KitsuneResult<()>>;

    /// Store agent info
    fn query_metrics(
        &self,
        query: MetricQuery,
    ) -> BoxFuture<'static, KitsuneResult<MetricQueryAnswer>>;

    /// Store entry
    fn store_entry(
        &self,
        root: KdHash,
        agent: KdHash,
        entry: KdEntry,
    ) -> BoxFuture<'static, KitsuneResult<()>>;

    /// Get entry
    fn get_entry(
        &self,
        root: KdHash,
        agent: KdHash,
        hash: KdHash,
    ) -> BoxFuture<'static, KitsuneResult<KdEntry>>;

    /// Get entry
    fn query_entries(
        &self,
        root: KdHash,
        agent: KdHash,
        created_at_start_s: f32,
        created_at_end_s: f32,
        dht_arc: DhtArc,
    ) -> BoxFuture<'static, KitsuneResult<Vec<KdEntry>>>;
}

/// Handle to a persistence store.
#[derive(Clone)]
pub struct KdPersist(pub Arc<dyn AsKdPersist>);

impl PartialEq for KdPersist {
    fn eq(&self, oth: &Self) -> bool {
        self.0.uniq().eq(&oth.0.uniq())
    }
}

impl Eq for KdPersist {}

impl std::hash::Hash for KdPersist {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.uniq().hash(state)
    }
}

impl KdPersist {
    /// Check if this persist instance has been closed
    pub fn is_closed(&self) -> bool {
        AsKdPersist::is_closed(&*self.0)
    }

    /// Explicitly close this persist instance
    pub fn close(&self) -> impl Future<Output = ()> + 'static + Send {
        AsKdPersist::close(&*self.0)
    }

    /// Get or create and get the singleton tls cert creds for this store.
    pub fn singleton_tls_config(
        &self,
    ) -> impl Future<Output = KitsuneResult<TlsConfig>> + 'static + Send {
        AsKdPersist::singleton_tls_config(&*self.0)
    }

    /// Generate a signature keypair, returning the pub key as a KdHash.
    pub fn generate_signing_keypair(
        &self,
    ) -> impl Future<Output = KitsuneResult<KdHash>> + 'static + Send {
        AsKdPersist::generate_signing_keypair(&*self.0)
    }

    /// Sign arbitrary data with the secret key associated with given KdHash.
    pub fn sign(
        &self,
        pub_key: KdHash,
        data: &[u8],
    ) -> impl Future<Output = KitsuneResult<Arc<[u8; 64]>>> + 'static + Send {
        AsKdPersist::sign(&*self.0, pub_key, data)
    }

    /// Store agent info
    pub fn store_agent_info(
        &self,
        agent_info: KdAgentInfo,
    ) -> impl Future<Output = KitsuneResult<()>> + 'static + Send {
        AsKdPersist::store_agent_info(&*self.0, agent_info)
    }

    /// Get agent info
    pub fn get_agent_info(
        &self,
        root: KdHash,
        agent: KdHash,
    ) -> impl Future<Output = KitsuneResult<KdAgentInfo>> + 'static + Send {
        AsKdPersist::get_agent_info(&*self.0, root, agent)
    }

    /// Query agent info
    pub fn query_agent_info(
        &self,
        root: KdHash,
    ) -> impl Future<Output = KitsuneResult<Vec<KdAgentInfo>>> + 'static + Send {
        AsKdPersist::query_agent_info(&*self.0, root)
    }

    /// Store entry
    pub fn store_entry(
        &self,
        root: KdHash,
        agent: KdHash,
        entry: KdEntry,
    ) -> impl Future<Output = KitsuneResult<()>> + 'static + Send {
        AsKdPersist::store_entry(&*self.0, root, agent, entry)
    }

    /// Get entry
    pub fn get_entry(
        &self,
        root: KdHash,
        agent: KdHash,
        hash: KdHash,
    ) -> impl Future<Output = KitsuneResult<KdEntry>> + 'static + Send {
        AsKdPersist::get_entry(&*self.0, root, agent, hash)
    }

    /// Get entry
    pub fn query_entries(
        &self,
        root: KdHash,
        agent: KdHash,
        created_at_start_s: f32,
        created_at_end_s: f32,
        dht_arc: DhtArc,
    ) -> impl Future<Output = KitsuneResult<Vec<KdEntry>>> + 'static + Send {
        AsKdPersist::query_entries(
            &*self.0,
            root,
            agent,
            created_at_start_s,
            created_at_end_s,
            dht_arc,
        )
    }
}
