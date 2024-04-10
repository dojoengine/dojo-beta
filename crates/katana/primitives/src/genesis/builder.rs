use cairo_lang_starknet::contract_class::ContractClass;

use crate::class::{ClassHash, DeprecatedCompiledClass, SierraClass};

use super::{Genesis, GenesisClass};
use std::{
    convert::{Infallible, TryInto},
    path::{Path, PathBuf},
};

#[derive(Debug, thiserror::Error)]
pub enum GenesisBuilderError {}

#[derive(Debug)]
pub struct Builder {}

impl Builder {
    pub fn parent_hash(self) -> Self {
        self
    }

    pub fn state_root(self) -> Self {
        self
    }

    pub fn number(self) -> Self {
        self
    }

    pub fn timestamp(self) -> Self {
        self
    }

    pub fn sequencer_address(self) -> Self {
        self
    }

    pub fn gas_prices(self) -> Self {
        self
    }

    pub fn fee_token(self) -> Self {
        self
    }

    pub fn with_classes(
        self,
        classes: Vec<(GenesisClassKind, Option<ClassHash>)>,
    ) -> Result<Self, GenesisBuilderError> {
        Ok(self)
    }

    pub fn with_allocations(self) -> Result<Self, GenesisBuilderError> {
        Ok(self)
    }

    pub fn build(self) -> Genesis {
        todo!()
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {}
    }
}

pub enum GenesisClassKind {
    Path(PathBuf),
    Contract(SierraClass),
    Deprecated(DeprecatedCompiledClass),
}

fn foo() -> Result<Genesis, GenesisBuilderError> {
    let classes = vec![
        (GenesisClassKind::Deprecated(DeprecatedCompiledClass::default()), None),
        (GenesisClassKind::Path(PathBuf::default()), Some(ClassHash::default())),
    ];
    let genesis = Builder::default().parent_hash().with_classes(classes)?.build();
    Ok(genesis)
}
