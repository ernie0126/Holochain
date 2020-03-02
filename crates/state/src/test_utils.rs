

use crate::{error::WorkspaceResult, env::create_lmdb_env};
use rkv::{Writer, Rkv};
use std::sync::{Arc, RwLock};
use tempdir::TempDir;

pub fn test_env() -> Arc<RwLock<Rkv>> {
    let tmpdir = TempDir::new("skunkworx").unwrap();
    create_lmdb_env(tmpdir.path())
}
