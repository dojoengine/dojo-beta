use crate::contract::Address;
use crate::Felt;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrderedL2ToL1Message {
    pub order: u64,
    pub from_address: Address,
    pub to_address: Felt,
    pub payload: Vec<Felt>,
}
