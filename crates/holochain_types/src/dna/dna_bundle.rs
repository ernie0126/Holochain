use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

use crate::prelude::*;
use futures::StreamExt;
use holo_hash::*;
use mr_bundle::{Location, ResourceBytes};

#[cfg(test)]
mod test;

/// A bundle of Wasm zomes, respresented as a file.
#[derive(
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    SerializedBytes,
    shrinkwraprs::Shrinkwrap,
    derive_more::From,
)]
pub struct DnaBundle(mr_bundle::Bundle<ValidatedDnaManifest>);

impl DnaBundle {
    /// Constructor
    pub fn new(
        manifest: ValidatedDnaManifest,
        resources: Vec<(PathBuf, Vec<u8>)>,
        root_dir: PathBuf,
    ) -> DnaResult<Self> {
        Ok(mr_bundle::Bundle::new(manifest, resources, root_dir)?.into())
    }

    /// Convert to a DnaFile, and return what the hash of the Dna *would* have
    /// been without the provided phenotype overrides
    pub async fn into_dna_file(
        self,
        uid: Option<Uid>,
        properties: Option<YamlProperties>,
    ) -> DnaResult<(DnaFile, DnaHash)> {
        let (integrity, coordinator, wasms) = self.inner_maps().await?;
        let (dna_def, original_hash) = self.to_dna_def(integrity, coordinator, uid, properties)?;

        Ok((DnaFile::from_parts(dna_def, wasms), original_hash))
    }

    /// Construct from raw bytes
    pub fn decode(bytes: &[u8]) -> DnaResult<Self> {
        mr_bundle::Bundle::decode(bytes)
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Read from a bundle file
    pub async fn read_from_file(path: &Path) -> DnaResult<Self> {
        mr_bundle::Bundle::read_from_file(path)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    async fn inner_maps(&self) -> DnaResult<(IntegrityZomes, CoordinatorZomes, WasmMap)> {
        let mut resources = self.resolve_all_cloned().await?;
        let data = match &self.manifest().0 {
            DnaManifest::V1(manifest) => {
                let integrity =
                    hash_bytes(manifest.integrity.zomes.iter().cloned(), &mut resources).await?;
                let coordinator =
                    hash_bytes(manifest.coordinator.zomes.iter().cloned(), &mut resources).await?;
                [integrity, coordinator]
            }
        };

        let integrity_zomes = data[0]
            .iter()
            .map(|(zome_name, hash, _, dependencies)| {
                (
                    zome_name.clone(),
                    ZomeDef::Wasm(WasmZome {
                        wasm_hash: hash.clone(),
                        dependencies: dependencies.clone(),
                    })
                    .into(),
                )
            })
            .collect();
        let coordinator_zomes = data[1]
            .iter()
            .map(|(zome_name, hash, _, dependencies)| {
                (
                    zome_name.clone(),
                    ZomeDef::Wasm(WasmZome {
                        wasm_hash: hash.clone(),
                        dependencies: dependencies.clone(),
                    })
                    .into(),
                )
            })
            .collect();
        let code: BTreeMap<_, _> = data
            .into_iter()
            .flatten()
            .map(|(_, hash, wasm, _)| (hash, wasm))
            .collect();

        let wasms = WasmMap::from(code);

        Ok((integrity_zomes, coordinator_zomes, wasms))
    }

    /// Convert to a DnaDef
    pub fn to_dna_def(
        &self,
        integrity_zomes: IntegrityZomes,
        coordinator_zomes: CoordinatorZomes,
        uid: Option<Uid>,
        properties: Option<YamlProperties>,
    ) -> DnaResult<(DnaDefHashed, DnaHash)> {
        match &self.manifest().0 {
            DnaManifest::V1(manifest) => {
                let mut dna_def = DnaDef {
                    name: manifest.name.clone(),
                    uid: manifest.integrity.uid.clone().unwrap_or_default(),
                    properties: SerializedBytes::try_from(
                        manifest.integrity.properties.clone().unwrap_or_default(),
                    )?,
                    origin_time: manifest.integrity.origin_time.into(),
                    integrity_zomes,
                    coordinator_zomes,
                };

                if uid.is_none() && properties.is_none() {
                    // If no phenotype overrides, then the original hash is the same as the current hash
                    let ddh = DnaDefHashed::from_content_sync(dna_def);
                    let original_hash = ddh.as_hash().clone();
                    Ok((ddh, original_hash))
                } else {
                    // Otherwise, record the original hash first, for version comparisons.
                    let original_hash = DnaHash::with_data_sync(&dna_def);

                    let props = manifest.integrity.properties.as_ref();
                    let properties: SerializedBytes = properties
                        .as_ref()
                        .or(props)
                        .map(SerializedBytes::try_from)
                        .unwrap_or_else(|| SerializedBytes::try_from(()))?;
                    let uid = uid
                        .or_else(|| manifest.integrity.uid.clone())
                        .unwrap_or_default();

                    dna_def.uid = uid;
                    dna_def.properties = properties;
                    Ok((DnaDefHashed::from_content_sync(dna_def), original_hash))
                }
            }
        }
    }

    /// Build a bundle from a DnaFile. Useful for tests.
    #[cfg(feature = "test_utils")]
    pub async fn from_dna_file(dna_file: DnaFile) -> DnaResult<Self> {
        let DnaFile { dna, code, .. } = dna_file;
        let manifest = Self::manifest_from_dna_def(dna.into_content())?;
        let resources = code
            .into_iter()
            .map(|(hash, wasm)| (PathBuf::from(hash.to_string()), wasm.code.to_vec()))
            .collect();
        DnaBundle::new(manifest.try_into()?, resources, PathBuf::from("."))
    }

    #[cfg(feature = "test_utils")]
    fn manifest_from_dna_def(dna_def: DnaDef) -> DnaResult<DnaManifest> {
        let integrity = dna_def
            .integrity_zomes
            .into_iter()
            .filter_map(|(name, zome)| {
                let dependencies = zome
                    .as_any_zome_def()
                    .dependencies()
                    .iter()
                    .cloned()
                    .map(|name| ZomeDependency { name })
                    .collect();
                zome.wasm_hash(&name).ok().map(|hash| {
                    let hash = WasmHashB64::from(hash);
                    let filename = format!("{}", hash);
                    ZomeManifest {
                        name,
                        hash: Some(hash),
                        location: Location::Bundled(PathBuf::from(filename)),
                        dependencies: Some(dependencies),
                    }
                })
            })
            .collect();
        let coordinator = dna_def
            .coordinator_zomes
            .into_iter()
            .filter_map(|(name, zome)| {
                let dependencies = zome
                    .as_any_zome_def()
                    .dependencies()
                    .iter()
                    .cloned()
                    .map(|name| ZomeDependency { name })
                    .collect();
                zome.wasm_hash(&name).ok().map(|hash| {
                    let hash = WasmHashB64::from(hash);
                    let filename = format!("{}", hash);
                    ZomeManifest {
                        name,
                        hash: Some(hash),
                        location: Location::Bundled(PathBuf::from(filename)),
                        dependencies: Some(dependencies),
                    }
                })
            })
            .collect();
        Ok(DnaManifestCurrent {
            name: dna_def.name,
            integrity: IntegrityManifest {
                uid: Some(dna_def.uid),
                properties: Some(dna_def.properties.try_into().map_err(|e| {
                    DnaError::DnaFileToBundleConversionError(format!(
                        "DnaDef properties were not YAML-deserializable: {}",
                        e
                    ))
                })?),
                origin_time: dna_def.origin_time.into(),
                zomes: integrity,
            },
            coordinator: CoordinatorManifest { zomes: coordinator },
        }
        .into())
    }
}

async fn hash_bytes(
    zomes: impl Iterator<Item = ZomeManifest>,
    resources: &mut HashMap<Location, ResourceBytes>,
) -> DnaResult<Vec<(ZomeName, WasmHash, DnaWasm, Vec<ZomeName>)>> {
    let iter = zomes.map(|z| {
        let bytes = resources
            .remove(&z.location)
            .expect("resource referenced in manifest must exist");
        let zome_name = z.name;
        let expected_hash = z.hash.map(WasmHash::from);
        let wasm = DnaWasm::from(bytes);
        let dependencies = z.dependencies.map_or(Vec::with_capacity(0), |deps| {
            deps.into_iter().map(|d| d.name).collect()
        });
        async move {
            let hash = wasm.to_hash().await;
            if let Some(expected) = expected_hash {
                if hash != expected {
                    return Err(DnaError::WasmHashMismatch(expected, hash));
                }
            }
            DnaResult::Ok((zome_name, hash, wasm, dependencies))
        }
    });
    futures::stream::iter(iter)
        .buffered(10)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn dna_bundle_to_dna_file() {
        let path1 = PathBuf::from("1");
        let path2 = PathBuf::from("2");
        let wasm1 = vec![1, 2, 3];
        let wasm2 = vec![4, 5, 6];
        let hash1 = DnaWasm::from(wasm1.clone()).to_hash().await;
        let hash2 = DnaWasm::from(wasm2.clone()).to_hash().await;
        let mut manifest = DnaManifestCurrent {
            name: "name".into(),
            integrity: IntegrityManifest {
                uid: Some("original uid".to_string()),
                properties: Some(serde_yaml::Value::Null.into()),
                origin_time: Timestamp::HOLOCHAIN_EPOCH.into(),
                zomes: vec![
                    ZomeManifest {
                        name: "zome1".into(),
                        hash: None,
                        location: mr_bundle::Location::Bundled(path1.clone()),
                        dependencies: Default::default(),
                    },
                    ZomeManifest {
                        name: "zome2".into(),
                        // Intentional wrong hash
                        hash: Some(hash1.clone().into()),
                        location: mr_bundle::Location::Bundled(path2.clone()),
                        dependencies: Default::default(),
                    },
                ],
            },
            coordinator: CoordinatorManifest { zomes: vec![] },
        };
        let resources = vec![(path1, wasm1), (path2, wasm2)];

        // - Show that conversion fails due to hash mismatch
        let bad_bundle: DnaBundle = mr_bundle::Bundle::new_unchecked(
            manifest.clone().try_into().unwrap(),
            resources.clone(),
        )
        .unwrap()
        .into();
        matches::assert_matches!(
            bad_bundle.into_dna_file(None, None).await,
            Err(DnaError::WasmHashMismatch(h1, h2))
            if h1 == hash1 && h2 == hash2
        );

        // - Correct the hash and try again
        manifest.integrity.zomes[1].hash = Some(hash2.into());
        let bundle: DnaBundle = mr_bundle::Bundle::new_unchecked(
            manifest.clone().try_into().unwrap(),
            resources.clone(),
        )
        .unwrap()
        .into();
        let dna_file: DnaFile = bundle.into_dna_file(None, None).await.unwrap().0;
        assert_eq!(dna_file.dna_def().integrity_zomes.len(), 2);
        assert_eq!(dna_file.code().len(), 2);

        // - Check that properties and UUID can be overridden
        let properties: YamlProperties = serde_yaml::Value::from(42).into();
        let bundle: DnaBundle =
            mr_bundle::Bundle::new_unchecked(manifest.try_into().unwrap(), resources)
                .unwrap()
                .into();
        let dna_file: DnaFile = bundle
            .into_dna_file(Some("uid".into()), Some(properties.clone()))
            .await
            .unwrap()
            .0;
        assert_eq!(dna_file.dna.uid, "uid".to_string());
        assert_eq!(
            dna_file.dna.properties,
            SerializedBytes::try_from(properties).unwrap()
        );
    }
}
