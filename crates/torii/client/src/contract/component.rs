use std::vec;

use crypto_bigint::U256;
use dojo_types::component::Member;
use starknet::core::types::{BlockId, FieldElement, FunctionCall};
use starknet::core::utils::{
    cairo_short_string_to_felt, get_selector_from_name, parse_cairo_short_string,
    CairoShortStringToFeltError, ParseCairoShortStringError,
};
use starknet::macros::short_string;
use starknet::providers::{Provider, ProviderError};
use starknet_crypto::poseidon_hash_many;

use crate::contract::world::{ContractReaderError, WorldContractReader};

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
    #[error("Unpacking entity")]
    UnpackingEntity,
    #[error(transparent)]
    ContractReaderError(ContractReaderError<P>),
}

pub struct ComponentReader<'a, P: Provider + Sync> {
    world: &'a WorldContractReader<'a, P>,
    class_hash: FieldElement,
    name: FieldElement,
}

impl<'a, P: Provider + Sync> ComponentReader<'a, P> {
    pub async fn new(
        world: &'a WorldContractReader<'a, P>,
        name: String,
        block_id: BlockId,
    ) -> Result<ComponentReader<'a, P>, ComponentError<P::Error>> {
        let name = cairo_short_string_to_felt(&name)
            .map_err(ComponentError::CairoShortStringToFeltError)?;
        let res = world
            .provider
            .call(
                FunctionCall {
                    contract_address: world.address,
                    calldata: vec![name],
                    entry_point_selector: get_selector_from_name("component").unwrap(),
                },
                block_id,
            )
            .await
            .map_err(ComponentError::ProviderError)?;

        Ok(Self { world, class_hash: res[0], name })
    }

    pub fn class_hash(&self) -> FieldElement {
        self.class_hash
    }

    pub async fn schema(&self, block_id: BlockId) -> Result<Vec<Member>, ComponentError<P::Error>> {
        let entrypoint = get_selector_from_name("schema").unwrap();

        let res = self
            .world
            .call(
                "library_call",
                vec![FieldElement::THREE, self.class_hash, entrypoint, FieldElement::ZERO],
                block_id,
            )
            .await
            .map_err(ComponentError::ContractReaderError)?;

        let mut members = vec![];
        for chunk in res[3..].chunks(3) {
            if chunk.len() != 3 {
                return Err(ComponentError::InvalidSchemaLength);
            }

            let is_key: u8 = chunk[2].try_into().map_err(|_| ComponentError::ConvertingFelt)?;

            members.push(Member {
                name: parse_cairo_short_string(&chunk[0])
                    .map_err(ComponentError::ParseCairoShortStringError)?,
                ty: parse_cairo_short_string(&chunk[1])
                    .map_err(ComponentError::ParseCairoShortStringError)?,
                key: is_key == 1,
            });
        }

        Ok(members)
    }

    pub async fn size(&self, block_id: BlockId) -> Result<FieldElement, ComponentError<P::Error>> {
        let entrypoint = get_selector_from_name("size").unwrap();

        let res = self
            .world
            .call(
                "library_call",
                vec![FieldElement::THREE, self.class_hash, entrypoint, FieldElement::ZERO],
                block_id,
            )
            .await
            .map_err(ComponentError::ContractReaderError)?;

        Ok(res[2])
    }

    pub async fn layout(
        &self,
        block_id: BlockId,
    ) -> Result<Vec<FieldElement>, ComponentError<P::Error>> {
        let entrypoint = get_selector_from_name("layout").unwrap();

        let res = self
            .world
            .call(
                "library_call",
                vec![FieldElement::THREE, self.class_hash, entrypoint, FieldElement::ZERO],
                block_id,
            )
            .await
            .map_err(ComponentError::ContractReaderError)?;

        Ok(res[3..].into())
    }

    pub async fn entity(
        &self,
        keys: Vec<FieldElement>,
        block_id: BlockId,
    ) -> Result<Vec<FieldElement>, ComponentError<P::Error>> {
        let size: u8 = self.size(block_id).await?.try_into().unwrap();
        let layout = self.layout(block_id).await?;

        let key = poseidon_hash_many(&keys);
        let key = poseidon_hash_many(&[short_string!("dojo_storage"), self.name, key]);

        let mut packed = vec![];
        for slot in 0..size {
            let value = self
                .world
                .provider
                .get_storage_at(self.world.address, key + slot.into(), block_id)
                .await
                .map_err(ComponentError::ProviderError)?;

            packed.push(value);
        }

        let unpacked = unpack::<P>(packed, layout)?;
        println!("{:?}", unpacked);
        Ok(unpacked)
    }
}

/// Unpacks a vector of packed values according to a given layout.
///
/// # Arguments
///
/// * `packed_values` - A vector of FieldElement values that are packed.
/// * `layout` - A vector of FieldElement values that describe the layout of the packed values.
///
/// # Returns
///
/// * `Result<Vec<FieldElement>, ComponentError<P::Error>>` - A Result containing a vector of
///   unpacked FieldElement values if successful, or an error if unsuccessful.
pub fn unpack<P: Provider>(
    mut packed: Vec<FieldElement>,
    layout: Vec<FieldElement>,
) -> Result<Vec<FieldElement>, ComponentError<P::Error>> {
    packed.reverse();
    let mut unpacked = vec![];

    let mut unpacking: U256 = packed.pop().ok_or(ComponentError::UnpackingEntity)?.as_ref().into();
    let mut offset = 0;

    // Iterate over the layout.
    for size in layout {
        let size: u8 = size.try_into().map_err(|_| ComponentError::ConvertingFelt)?;
        let size: usize = size.into();
        let remaining_bits = 251 - offset;

        // If there are less remaining bits than the size, move to the next felt for unpacking.
        if remaining_bits < size {
            unpacking = packed.pop().ok_or(ComponentError::UnpackingEntity)?.as_ref().into();
            offset = 0;
        }

        // Calculate the result and push it to the unpacked values.
        let mask = U256::from(((1 << size) - 1) as u8);
        let result = mask & (unpacking >> offset);
        let result_fe = FieldElement::from_hex_be(&result.to_string())
            .map_err(|_| ComponentError::ConvertingFelt)?;
        unpacked.push(result_fe);

        // Update unpacking to be the shifted value after extracting the result.
        offset += size;
    }

    Ok(unpacked)
}
