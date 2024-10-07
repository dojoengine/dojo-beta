use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use async_trait::async_trait;
use cainome::parser::tokens::Composite;

use crate::error::BindgenResult;
use crate::DojoData;

pub mod typescript;
pub mod typescript_v2;
pub mod unity;

#[derive(Debug)]
pub enum BuiltinPlugins {
    Typescript,
    Unity,
    TypeScriptV2,
}

impl fmt::Display for BuiltinPlugins {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuiltinPlugins::Typescript => write!(f, "typescript"),
            BuiltinPlugins::Unity => write!(f, "unity"),
            BuiltinPlugins::TypeScriptV2 => write!(f, "typescript_v2"),
        }
    }
}

#[async_trait]
pub trait BuiltinPlugin: Sync {
    /// Generates code by executing the plugin.
    ///
    /// # Arguments
    ///
    /// * `data` - Dojo data gathered from the compiled project.
    async fn generate_code(&self, data: &DojoData) -> BindgenResult<HashMap<PathBuf, Vec<u8>>>;
}

pub trait BindgenWriter: Sync {
    /// Writes the generated code to the specified path.
    ///
    /// # Arguments
    ///
    /// * `code` - The generated code.
    fn write(&self, path: &str, data: &DojoData) -> BindgenResult<(PathBuf, Vec<u8>)>;
    fn get_path(&self) -> &str;
}

pub trait BindgenGenerator: Sync {
    /// Generates code by executing the plugin.
    /// The generated code is written to the specified path.
    /// This will write file sequentially (for now) so we need one generator per part of the file.
    /// (header, type definitions, interfaces, functions and so on)
    /// TODO: add &mut ref to what's currently generated to place specific code at specific places.
    ///
    /// # Arguments
    ///
    ///
    fn generate(&self, token: &Composite, buffer: &mut Vec<String>) -> BindgenResult<String>;
}
