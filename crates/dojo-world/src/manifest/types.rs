use std::fs;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use smol_str::SmolStr;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet::core::types::contract::AbiEntry;
use starknet::core::types::Felt;

use crate::manifest::AbstractManifestError;

// Collection of different types of `Manifest`'s which are used by dojo compiler/sozo
// For example:
//   - `BaseManifest` is generated by the compiler and wrote to `manifests/base` folder of project
//   - `DeploymentManifest` is generated by sozo which represents the future onchain state after a
//     successful migration
//   - `OverlayManifest` is used by sozo to override values of specific manifest of `BaseManifest`
//     thats generated by compiler

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct BaseManifest {
    pub world: Manifest<Class>,
    pub base: Manifest<Class>,
    pub contracts: Vec<Manifest<DojoContract>>,
    pub models: Vec<Manifest<DojoModel>>,
    pub events: Vec<Manifest<DojoEvent>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct DeploymentManifest {
    pub world: Manifest<WorldContract>,
    pub base: Manifest<Class>,
    // NOTE: `writes` field in contracts is of String but we read the values which are resource
    // hashes from the events, so needs to be handled accordingly
    pub contracts: Vec<Manifest<DojoContract>>,
    pub models: Vec<Manifest<DojoModel>>,
    pub events: Vec<Manifest<DojoEvent>>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct OverlayManifest {
    pub world: Option<OverlayClass>,
    pub base: Option<OverlayClass>,
    pub contracts: Vec<OverlayDojoContract>,
    pub models: Vec<OverlayDojoModel>,
    pub events: Vec<OverlayDojoEvent>,
}

#[derive(Clone, Serialize, Default, Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Manifest<T>
where
    T: ManifestMethods,
{
    #[serde(flatten)]
    pub inner: T,

    // name of the manifest which is used as filename
    pub manifest_name: String,
}

// Utility methods thats needs to be implemented by manifest types
pub trait ManifestMethods {
    type OverlayType;
    fn abi(&self) -> Option<&AbiFormat>;
    fn set_abi(&mut self, abi: Option<AbiFormat>);
    fn class_hash(&self) -> &Felt;
    fn set_class_hash(&mut self, class_hash: Felt);
    fn original_class_hash(&self) -> &Felt;

    /// This method is called when during compilation base manifest file already exists.
    /// Manifest generated during compilation won't contains properties manually updated by users
    /// (like calldata) so this method should override those fields
    fn merge(&mut self, old: Self::OverlayType);
}

impl<T> Manifest<T>
where
    T: ManifestMethods,
{
    pub fn new(inner: T, manifest_name: String) -> Self {
        Self { inner, manifest_name }
    }
}

#[serde_as]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "kind")]
pub struct DojoContract {
    #[serde_as(as = "Option<UfeHex>")]
    pub address: Option<Felt>,
    #[serde_as(as = "UfeHex")]
    pub class_hash: Felt,
    #[serde_as(as = "UfeHex")]
    pub original_class_hash: Felt,
    // base class hash used to deploy the contract
    #[serde_as(as = "UfeHex")]
    pub base_class_hash: Felt,
    pub abi: Option<AbiFormat>,
    #[serde(default)]
    pub reads: Vec<String>,
    #[serde(default)]
    pub writes: Vec<String>,
    #[serde(default)]
    pub init_calldata: Vec<String>,
    pub tag: String,
    pub systems: Vec<String>,
}

/// Represents a declaration of a model.
#[serde_as]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "kind")]
pub struct DojoModel {
    pub members: Vec<Member>,
    #[serde_as(as = "UfeHex")]
    pub class_hash: Felt,
    #[serde_as(as = "UfeHex")]
    pub original_class_hash: Felt,
    pub abi: Option<AbiFormat>,
    pub tag: String,
    pub qualified_path: String,
}

/// Represents a declaration of an event.
#[serde_as]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "kind")]
pub struct DojoEvent {
    pub members: Vec<Member>,
    #[serde_as(as = "UfeHex")]
    pub class_hash: Felt,
    #[serde_as(as = "UfeHex")]
    pub original_class_hash: Felt,
    pub abi: Option<AbiFormat>,
    pub tag: String,
    pub qualified_path: String,
}

#[serde_as]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "kind")]
pub struct WorldContract {
    #[serde_as(as = "UfeHex")]
    pub class_hash: Felt,
    #[serde_as(as = "UfeHex")]
    pub original_class_hash: Felt,
    pub abi: Option<AbiFormat>,
    #[serde_as(as = "Option<UfeHex>")]
    pub address: Option<Felt>,
    #[serde_as(as = "Option<UfeHex>")]
    pub transaction_hash: Option<Felt>,
    pub block_number: Option<u64>,
    pub seed: String,
    pub metadata: Option<WorldMetadata>,
}

#[serde_as]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "kind")]
pub struct Class {
    #[serde_as(as = "UfeHex")]
    pub class_hash: Felt,
    #[serde_as(as = "UfeHex")]
    pub original_class_hash: Felt,
    pub abi: Option<AbiFormat>,
    pub tag: String,
}

#[serde_as]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct OverlayDojoContract {
    pub tag: String,
    pub original_class_hash: Option<Felt>,
    pub reads: Option<Vec<String>>,
    pub writes: Option<Vec<String>>,
    pub init_calldata: Option<Vec<String>>,
}

#[serde_as]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct OverlayDojoModel {
    pub tag: String,
    pub original_class_hash: Option<Felt>,
}

#[serde_as]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct OverlayDojoEvent {
    pub tag: String,
    pub original_class_hash: Option<Felt>,
}

#[serde_as]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct OverlayContract {
    pub name: SmolStr,
    pub original_class_hash: Option<Felt>,
}

#[serde_as]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct OverlayClass {
    pub tag: String,
    pub original_class_hash: Option<Felt>,
}

// Types used by manifest

/// Represents a model member.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Member {
    /// Name of the member.
    pub name: String,
    /// Type of the member.
    #[serde(rename = "type")]
    pub ty: String,
    pub key: bool,
}

impl From<dojo_types::schema::Member> for Member {
    fn from(m: dojo_types::schema::Member) -> Self {
        Self { name: m.name, ty: m.ty.name(), key: m.key }
    }
}

/// System input ABI.
#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Input {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
}

/// System Output ABI.
#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Output {
    #[serde(rename = "type")]
    pub ty: String,
}

/// Format of the ABI into the manifest.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AbiFormat {
    /// Only a relative path to the ABI file is stored.
    Path(Utf8PathBuf),
    /// The full ABI is embedded.
    Embed(Vec<AbiEntry>),
}

impl AbiFormat {
    /// Get the [`Utf8PathBuf`] if the ABI is stored as a path.
    pub fn to_path(&self) -> Option<&Utf8PathBuf> {
        match self {
            AbiFormat::Path(p) => Some(p),
            AbiFormat::Embed(_) => None,
        }
    }

    /// Loads an ABI from the path or embedded entries.
    ///
    /// # Arguments
    ///
    /// * `root_dir` - The root directory of the ABI file.
    pub fn load_abi_string(&self, root_dir: &Utf8PathBuf) -> Result<String, AbstractManifestError> {
        match self {
            AbiFormat::Path(abi_path) => Ok(fs::read_to_string(root_dir.join(abi_path))?),
            AbiFormat::Embed(abi) => Ok(serde_json::to_string(&abi)?),
        }
    }

    /// Convert to embed variant.
    ///
    /// # Arguments
    ///
    /// * `root_dir` - The root directory for the abi file resolution.
    pub fn to_embed(&self, root_dir: &Utf8PathBuf) -> Result<AbiFormat, AbstractManifestError> {
        if let AbiFormat::Path(abi_path) = self {
            let mut abi_file = std::fs::File::open(root_dir.join(abi_path))?;
            Ok(serde_json::from_reader(&mut abi_file)?)
        } else {
            Ok(self.clone())
        }
    }
}

#[cfg(test)]
impl PartialEq for AbiFormat {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AbiFormat::Path(p1), AbiFormat::Path(p2)) => p1 == p2,
            (AbiFormat::Embed(e1), AbiFormat::Embed(e2)) => {
                // Currently, [`AbiEntry`] does not implement [`PartialEq`] so we cannot compare
                // them directly.
                let e1_json = serde_json::to_string(e1).expect("valid JSON from ABI");
                let e2_json = serde_json::to_string(e2).expect("valid JSON from ABI");
                e1_json == e2_json
            }
            _ => false,
        }
    }
}

#[serde_as]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct WorldMetadata {
    pub profile_name: String,
    pub rpc_url: String,
}
