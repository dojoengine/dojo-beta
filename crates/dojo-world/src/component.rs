use starknet::core::types::{BlockId, FieldElement, FunctionCall};
use starknet::core::utils::{
    cairo_short_string_to_felt, get_selector_from_name, parse_cairo_short_string,
    CairoShortStringToFeltError, ParseCairoShortStringError,
};
use starknet::providers::{Provider, ProviderError};

use crate::manifest::Member;
use crate::world::{ContractReaderError, WorldContractReader};

#[cfg(test)]
#[path = "component_test.rs"]
mod test;

#[derive(Debug, thiserror::Error)]
pub enum ComponentError<P> {
    #[error(transparent)]
    ProviderError(ProviderError<P>),
    #[error("Invalid schema length")]
    InvalidSchemaLength,
    #[error(transparent)]
    ParseCairoShortStringError(ParseCairoShortStringError),
    #[error(transparent)]
    CairoShortStringToFeltError(CairoShortStringToFeltError),
    #[error("Converting felt")]
    ConvertingFelt,
    #[error(transparent)]
    ContractReaderError(ContractReaderError<P>),
}

pub struct ComponentReader<'a, P: Provider + Sync> {
    world: &'a WorldContractReader<'a, P>,
    class_hash: FieldElement,
}

impl<'a, P: Provider + Sync> ComponentReader<'a, P> {
    pub async fn new(
        world: &'a WorldContractReader<'a, P>,
        name: String,
        block_id: BlockId,
    ) -> Result<ComponentReader<'a, P>, ComponentError<P::Error>> {
        let res = world
            .provider
            .call(
                FunctionCall {
                    contract_address: world.address,
                    calldata: vec![
                        cairo_short_string_to_felt(&name)
                            .map_err(ComponentError::CairoShortStringToFeltError)?,
                    ],
                    entry_point_selector: get_selector_from_name("component").unwrap(),
                },
                block_id,
            )
            .await
            .map_err(ComponentError::ProviderError)?;

        Ok(Self { world, class_hash: res[0] })
    }

    pub fn class_hash(&self) -> FieldElement {
        self.class_hash
    }

    pub async fn schema(&self, block_id: BlockId) -> Result<Vec<Member>, ComponentError<P::Error>> {
        let entrypoint = get_selector_from_name("schema").unwrap();

        let res = self
            .world
            .call(
                "LibraryCall",
                vec![FieldElement::THREE, self.class_hash, entrypoint, FieldElement::ZERO],
                block_id,
            )
            .await
            .map_err(ComponentError::ContractReaderError)?;

        let mut members = vec![];
        for chunk in res[3..].chunks(4) {
            if chunk.len() != 4 {
                return Err(ComponentError::InvalidSchemaLength);
            }

            members.push(Member {
                name: parse_cairo_short_string(&chunk[0])
                    .map_err(ComponentError::ParseCairoShortStringError)?,
                ty: parse_cairo_short_string(&chunk[1])
                    .map_err(ComponentError::ParseCairoShortStringError)?,
                slot: chunk[2].try_into().map_err(|_| ComponentError::ConvertingFelt)?,
                offset: chunk[3].try_into().map_err(|_| ComponentError::ConvertingFelt)?,
            });
        }

        Ok(members)
    }
}
