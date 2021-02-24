//! Functions dealing with obtaining and referencing singleton LMDB environments

use crate::prelude::*;
use derive_more::Into;
use holochain_keystore::KeystoreSender;
use holochain_zome_types::cell::CellId;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use rkv::EnvironmentFlags;
use shrinkwraprs::Shrinkwrap;
use std::collections::hash_map;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

const DEFAULT_INITIAL_MAP_SIZE: usize = 100 * 1024 * 1024; // 100MB
const MAX_DBS: u32 = 32;

lazy_static! {
    static ref ENVIRONMENTS: RwLock<HashMap<PathBuf, EnvironmentWrite>> = {
        // This is just a convenient place that we know gets initialized
        // both in the final binary holochain && in all relevant tests
        //
        // Holochain (and most binaries) are left in invalid states
        // if a thread panic!s - switch to failing fast in that case.
        //
        // We tried putting `panic = "abort"` in the Cargo.toml,
        // but somehow that breaks the wasmer / test_utils integration.

        let orig_handler = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            // print the panic message
            eprintln!("FATAL PANIC {:#?}", panic_info);
            // invoke the original handler
            orig_handler(panic_info);
            // // Abort the process
            // // TODO - we need a better solution than this, but if there is
            // // no better solution, we can uncomment the following line:
            // std::process::abort();
        }));

        RwLock::new(HashMap::new())
    };
}

/// A read-only version of [EnvironmentWrite].
/// This environment can only generate read-only transactions, never read-write.
#[derive(Clone)]
pub struct EnvironmentRead {
    kind: EnvironmentKind,
    path: PathBuf,
    keystore: KeystoreSender,
}

impl EnvironmentRead {
    #[deprecated = "remove this identity function"]
    pub fn guard(&self) -> Self {
        self.clone()
    }

    #[deprecated = "remove this identity function"]
    pub fn inner(&self) -> Self {
        self.clone()
    }

    /// Accessor for the [EnvironmentKind] of the EnvironmentWrite
    pub fn kind(&self) -> &EnvironmentKind {
        &self.kind
    }

    /// Request access to this conductor's keystore
    pub fn keystore(&self) -> &KeystoreSender {
        &self.keystore
    }

    /// The environments path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// SHIM
    pub fn open_single<'s, T>(
        &self,
        name: T,
        opts: rkv::StoreOptions,
    ) -> Result<SingleStore, StoreError>
    where
        T: Into<Option<&'s str>>,
    {
        todo!("this is a shim")
    }

    /// SHIM
    pub fn open_integer<'s, T>(
        &self,
        name: T,
        mut opts: rkv::StoreOptions,
    ) -> Result<IntegerStore, StoreError>
    where
        T: Into<Option<&'s str>>,
    {
        todo!("this is a shim")
    }

    /// SHIM
    pub fn open_multi<'s, T>(
        &self,
        name: T,
        mut opts: rkv::StoreOptions,
    ) -> Result<MultiStore, StoreError>
    where
        T: Into<Option<&'s str>>,
    {
        todo!("this is a shim")
    }

    // /// SHIM
    // pub fn open_multi_integer<'s, T, K: PrimitiveInt>(
    //     &self,
    //     name: T,
    //     mut opts: StoreOptions,
    // ) -> Result<MultiIntegerStore<K>, StoreError>
    // where
    //     T: Into<Option<&'s str>>,
    // {
    //     todo!("this is a shim")
    // }
}

impl GetDb for EnvironmentRead {}
impl GetDb for EnvironmentWrite {}

/// The canonical representation of a (singleton) LMDB environment.
/// The wrapper contains methods for managing transactions
/// and database connections,
#[derive(Clone, Shrinkwrap, Into, derive_more::From)]
pub struct EnvironmentWrite(EnvironmentRead);

impl EnvironmentWrite {
    /// Create an environment,
    pub fn new(
        path_prefix: &Path,
        kind: EnvironmentKind,
        keystore: KeystoreSender,
    ) -> DatabaseResult<EnvironmentWrite> {
        let mut map = ENVIRONMENTS.write();
        let path = path_prefix.join(kind.path());
        if !path.is_dir() {
            std::fs::create_dir(path.clone())
                .map_err(|_e| DatabaseError::EnvironmentMissing(path.clone()))?;
        }
        let env: EnvironmentWrite = match map.entry(path.clone()) {
            hash_map::Entry::Occupied(e) => e.get().clone(),
            hash_map::Entry::Vacant(e) => e
                .insert({
                    tracing::debug!("Initializing databases for path {:?}", path);
                    initialize_databases(&path, &kind)?;
                    EnvironmentWrite(EnvironmentRead {
                        kind,
                        keystore,
                        path,
                    })
                })
                .clone(),
        };
        Ok(env)
    }

    /// Create a Cell environment (slight shorthand)
    pub fn new_cell(
        path_prefix: &Path,
        cell_id: CellId,
        keystore: KeystoreSender,
    ) -> DatabaseResult<Self> {
        Self::new(path_prefix, EnvironmentKind::Cell(cell_id), keystore)
    }

    #[deprecated = "remove this identity function"]
    pub fn guard(&self) -> Self {
        self.clone()
    }

    /// Remove the db and directory
    pub async fn remove(self) -> DatabaseResult<()> {
        todo!();

        // let mut map = ENVIRONMENTS.write();
        // map.remove(&self.0.path);

        // remove the directory
        std::fs::remove_dir_all(&self.0.path)?;
        Ok(())
    }
}

/// The various types of LMDB environment, used to specify the list of databases to initialize
#[derive(Clone)]
pub enum EnvironmentKind {
    /// Specifies the environment used by each Cell
    Cell(CellId),
    /// Specifies the environment used by a Conductor
    Conductor,
    /// Specifies the environment used to save wasm
    Wasm,
    /// State of the p2p network
    P2p,
}

impl EnvironmentKind {
    /// Constuct a partial Path based on the kind
    fn path(&self) -> PathBuf {
        match self {
            EnvironmentKind::Cell(cell_id) => PathBuf::from(cell_id.to_string()),
            EnvironmentKind::Conductor => PathBuf::from("conductor"),
            EnvironmentKind::Wasm => PathBuf::from("wasm"),
            EnvironmentKind::P2p => PathBuf::from("p2p"),
        }
    }
}

/// Implementors are able to create a new read-only LMDB transaction
pub trait ReadManager<'e> {
    /// Create a new read-only LMDB transaction
    fn reader(&'e self) -> DatabaseResult<Reader<'e>>;

    /// Run a closure, passing in a new read-only transaction
    fn with_reader<E, R, F: Send>(&self, f: F) -> Result<R, E>
    where
        E: From<DatabaseError>,
        F: FnOnce(Reader) -> Result<R, E>;
}

/// Implementors are able to create a new read-write LMDB transaction
pub trait WriteManager<'e> {
    /// Run a closure, passing in a mutable reference to a read-write
    /// transaction, and commit the transaction after the closure has run.
    /// If there is a LMDB error, recover from it and re-run the closure.
    // FIXME: B-01566: implement write failure detection
    fn with_commit<E, R, F: Send>(&self, f: F) -> Result<R, E>
    where
        E: From<DatabaseError>,
        F: FnOnce(&mut Writer) -> Result<R, E>;
}

impl<'e> ReadManager<'e> for EnvironmentRead {
    fn reader(&'e self) -> DatabaseResult<Reader<'e>> {
        todo!("probably no longer makes sense")
        // let reader = Reader::from(self.rkv.read()?);
        // Ok(reader)
    }

    fn with_reader<E, R, F: Send>(&self, f: F) -> Result<R, E>
    where
        E: From<DatabaseError>,
        F: FnOnce(Reader) -> Result<R, E>,
    {
        f(self.reader()?)
    }
}

impl<'e> ReadManager<'e> for EnvironmentWrite {
    fn reader(&'e self) -> DatabaseResult<Reader<'e>> {
        todo!("probably no longer makes sense")
        // let reader = Reader::from(self.rkv.read()?);
        // Ok(reader)
    }

    fn with_reader<E, R, F: Send>(&self, f: F) -> Result<R, E>
    where
        E: From<DatabaseError>,
        F: FnOnce(Reader) -> Result<R, E>,
    {
        f(self.reader()?)
    }
}

impl<'e> WriteManager<'e> for EnvironmentWrite {
    fn with_commit<E, R, F: Send>(&self, f: F) -> Result<R, E>
    where
        E: From<DatabaseError>,
        F: FnOnce(&mut Writer) -> Result<R, E>,
    {
        todo!("probably no longer makes sense")
    }
}
