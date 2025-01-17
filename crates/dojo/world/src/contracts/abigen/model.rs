// ****
// Auto-generated by cainome do not edit.
// ****

#![allow(clippy::all)]
#![allow(warnings)]

#[derive(Debug)]
pub struct ModelContract<A: starknet::accounts::ConnectedAccount + Sync> {
    pub address: starknet::core::types::Felt,
    pub account: A,
    pub block_id: starknet::core::types::BlockId,
}
impl<A: starknet::accounts::ConnectedAccount + Sync> ModelContract<A> {
    pub fn new(address: starknet::core::types::Felt, account: A) -> Self {
        Self {
            address,
            account,
            block_id: starknet::core::types::BlockId::Tag(starknet::core::types::BlockTag::Pending),
        }
    }
    pub fn set_contract_address(&mut self, address: starknet::core::types::Felt) {
        self.address = address;
    }
    pub fn provider(&self) -> &A::Provider {
        self.account.provider()
    }
    pub fn set_block(&mut self, block_id: starknet::core::types::BlockId) {
        self.block_id = block_id;
    }
    pub fn with_block(self, block_id: starknet::core::types::BlockId) -> Self {
        Self { block_id, ..self }
    }
}
#[derive(Debug)]
pub struct ModelContractReader<P: starknet::providers::Provider + Sync> {
    pub address: starknet::core::types::Felt,
    pub provider: P,
    pub block_id: starknet::core::types::BlockId,
}
impl<P: starknet::providers::Provider + Sync> ModelContractReader<P> {
    pub fn new(address: starknet::core::types::Felt, provider: P) -> Self {
        Self {
            address,
            provider,
            block_id: starknet::core::types::BlockId::Tag(starknet::core::types::BlockTag::Pending),
        }
    }
    pub fn set_contract_address(&mut self, address: starknet::core::types::Felt) {
        self.address = address;
    }
    pub fn provider(&self) -> &P {
        &self.provider
    }
    pub fn set_block(&mut self, block_id: starknet::core::types::BlockId) {
        self.block_id = block_id;
    }
    pub fn with_block(self, block_id: starknet::core::types::BlockId) -> Self {
        Self { block_id, ..self }
    }
}
#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct Enum {
    pub name: starknet::core::types::Felt,
    pub attrs: Vec<starknet::core::types::Felt>,
    pub children: Vec<(starknet::core::types::Felt, Ty)>,
}
impl cainome::cairo_serde::CairoSerde for Enum {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        let mut __size = 0;
        __size += starknet::core::types::Felt::cairo_serialized_size(&__rust.name);
        __size += Vec::<starknet::core::types::Felt>::cairo_serialized_size(&__rust.attrs);
        __size += Vec::<(starknet::core::types::Felt, Ty)>::cairo_serialized_size(&__rust.children);
        __size
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet::core::types::Felt> {
        let mut __out: Vec<starknet::core::types::Felt> = vec![];
        __out.extend(starknet::core::types::Felt::cairo_serialize(&__rust.name));
        __out.extend(Vec::<starknet::core::types::Felt>::cairo_serialize(&__rust.attrs));
        __out.extend(Vec::<(starknet::core::types::Felt, Ty)>::cairo_serialize(&__rust.children));
        __out
    }
    fn cairo_deserialize(
        __felts: &[starknet::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let mut __offset = __offset;
        let name = starknet::core::types::Felt::cairo_deserialize(__felts, __offset)?;
        __offset += starknet::core::types::Felt::cairo_serialized_size(&name);
        let attrs = Vec::<starknet::core::types::Felt>::cairo_deserialize(__felts, __offset)?;
        __offset += Vec::<starknet::core::types::Felt>::cairo_serialized_size(&attrs);
        let children =
            Vec::<(starknet::core::types::Felt, Ty)>::cairo_deserialize(__felts, __offset)?;
        __offset += Vec::<(starknet::core::types::Felt, Ty)>::cairo_serialized_size(&children);
        Ok(Enum { name, attrs, children })
    }
}
#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct FieldLayout {
    pub selector: starknet::core::types::Felt,
    pub layout: Layout,
}
impl cainome::cairo_serde::CairoSerde for FieldLayout {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        let mut __size = 0;
        __size += starknet::core::types::Felt::cairo_serialized_size(&__rust.selector);
        __size += Layout::cairo_serialized_size(&__rust.layout);
        __size
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet::core::types::Felt> {
        let mut __out: Vec<starknet::core::types::Felt> = vec![];
        __out.extend(starknet::core::types::Felt::cairo_serialize(&__rust.selector));
        __out.extend(Layout::cairo_serialize(&__rust.layout));
        __out
    }
    fn cairo_deserialize(
        __felts: &[starknet::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let mut __offset = __offset;
        let selector = starknet::core::types::Felt::cairo_deserialize(__felts, __offset)?;
        __offset += starknet::core::types::Felt::cairo_serialized_size(&selector);
        let layout = Layout::cairo_deserialize(__felts, __offset)?;
        __offset += Layout::cairo_serialized_size(&layout);
        Ok(FieldLayout { selector, layout })
    }
}
#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct Member {
    pub name: starknet::core::types::Felt,
    pub attrs: Vec<starknet::core::types::Felt>,
    pub ty: Ty,
}
impl cainome::cairo_serde::CairoSerde for Member {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        let mut __size = 0;
        __size += starknet::core::types::Felt::cairo_serialized_size(&__rust.name);
        __size += Vec::<starknet::core::types::Felt>::cairo_serialized_size(&__rust.attrs);
        __size += Ty::cairo_serialized_size(&__rust.ty);
        __size
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet::core::types::Felt> {
        let mut __out: Vec<starknet::core::types::Felt> = vec![];
        __out.extend(starknet::core::types::Felt::cairo_serialize(&__rust.name));
        __out.extend(Vec::<starknet::core::types::Felt>::cairo_serialize(&__rust.attrs));
        __out.extend(Ty::cairo_serialize(&__rust.ty));
        __out
    }
    fn cairo_deserialize(
        __felts: &[starknet::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let mut __offset = __offset;
        let name = starknet::core::types::Felt::cairo_deserialize(__felts, __offset)?;
        __offset += starknet::core::types::Felt::cairo_serialized_size(&name);
        let attrs = Vec::<starknet::core::types::Felt>::cairo_deserialize(__felts, __offset)?;
        __offset += Vec::<starknet::core::types::Felt>::cairo_serialized_size(&attrs);
        let ty = Ty::cairo_deserialize(__felts, __offset)?;
        __offset += Ty::cairo_serialized_size(&ty);
        Ok(Member { name, attrs, ty })
    }
}
#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct ModelDef {
    pub name: cainome::cairo_serde::ByteArray,
    pub layout: Layout,
    pub schema: Struct,
    pub packed_size: Option<u32>,
    pub unpacked_size: Option<u32>,
}
impl cainome::cairo_serde::CairoSerde for ModelDef {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        let mut __size = 0;
        __size += cainome::cairo_serde::ByteArray::cairo_serialized_size(&__rust.name);
        __size += Layout::cairo_serialized_size(&__rust.layout);
        __size += Struct::cairo_serialized_size(&__rust.schema);
        __size += Option::<u32>::cairo_serialized_size(&__rust.packed_size);
        __size += Option::<u32>::cairo_serialized_size(&__rust.unpacked_size);
        __size
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet::core::types::Felt> {
        let mut __out: Vec<starknet::core::types::Felt> = vec![];
        __out.extend(cainome::cairo_serde::ByteArray::cairo_serialize(&__rust.name));
        __out.extend(Layout::cairo_serialize(&__rust.layout));
        __out.extend(Struct::cairo_serialize(&__rust.schema));
        __out.extend(Option::<u32>::cairo_serialize(&__rust.packed_size));
        __out.extend(Option::<u32>::cairo_serialize(&__rust.unpacked_size));
        __out
    }
    fn cairo_deserialize(
        __felts: &[starknet::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let mut __offset = __offset;
        let name = cainome::cairo_serde::ByteArray::cairo_deserialize(__felts, __offset)?;
        __offset += cainome::cairo_serde::ByteArray::cairo_serialized_size(&name);
        let layout = Layout::cairo_deserialize(__felts, __offset)?;
        __offset += Layout::cairo_serialized_size(&layout);
        let schema = Struct::cairo_deserialize(__felts, __offset)?;
        __offset += Struct::cairo_serialized_size(&schema);
        let packed_size = Option::<u32>::cairo_deserialize(__felts, __offset)?;
        __offset += Option::<u32>::cairo_serialized_size(&packed_size);
        let unpacked_size = Option::<u32>::cairo_deserialize(__felts, __offset)?;
        __offset += Option::<u32>::cairo_serialized_size(&unpacked_size);
        Ok(ModelDef { name, layout, schema, packed_size, unpacked_size })
    }
}
#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct ResourceMetadata {
    pub resource_id: starknet::core::types::Felt,
    pub metadata_uri: cainome::cairo_serde::ByteArray,
    pub metadata_hash: starknet::core::types::Felt,
}
impl cainome::cairo_serde::CairoSerde for ResourceMetadata {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        let mut __size = 0;
        __size += starknet::core::types::Felt::cairo_serialized_size(&__rust.resource_id);
        __size += cainome::cairo_serde::ByteArray::cairo_serialized_size(&__rust.metadata_uri);
        __size += starknet::core::types::Felt::cairo_serialized_size(&__rust.metadata_hash);
        __size
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet::core::types::Felt> {
        let mut __out: Vec<starknet::core::types::Felt> = vec![];
        __out.extend(starknet::core::types::Felt::cairo_serialize(&__rust.resource_id));
        __out.extend(cainome::cairo_serde::ByteArray::cairo_serialize(&__rust.metadata_uri));
        __out.extend(starknet::core::types::Felt::cairo_serialize(&__rust.metadata_hash));
        __out
    }
    fn cairo_deserialize(
        __felts: &[starknet::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let mut __offset = __offset;
        let resource_id = starknet::core::types::Felt::cairo_deserialize(__felts, __offset)?;
        __offset += starknet::core::types::Felt::cairo_serialized_size(&resource_id);
        let metadata_uri = cainome::cairo_serde::ByteArray::cairo_deserialize(__felts, __offset)?;
        __offset += cainome::cairo_serde::ByteArray::cairo_serialized_size(&metadata_uri);
        let metadata_hash = starknet::core::types::Felt::cairo_deserialize(__felts, __offset)?;
        __offset += starknet::core::types::Felt::cairo_serialized_size(&metadata_hash);
        Ok(ResourceMetadata { resource_id, metadata_uri, metadata_hash })
    }
}
#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct ResourceMetadataValue {
    pub metadata_uri: cainome::cairo_serde::ByteArray,
    pub metadata_hash: starknet::core::types::Felt,
}
impl cainome::cairo_serde::CairoSerde for ResourceMetadataValue {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        let mut __size = 0;
        __size += cainome::cairo_serde::ByteArray::cairo_serialized_size(&__rust.metadata_uri);
        __size += starknet::core::types::Felt::cairo_serialized_size(&__rust.metadata_hash);
        __size
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet::core::types::Felt> {
        let mut __out: Vec<starknet::core::types::Felt> = vec![];
        __out.extend(cainome::cairo_serde::ByteArray::cairo_serialize(&__rust.metadata_uri));
        __out.extend(starknet::core::types::Felt::cairo_serialize(&__rust.metadata_hash));
        __out
    }
    fn cairo_deserialize(
        __felts: &[starknet::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let mut __offset = __offset;
        let metadata_uri = cainome::cairo_serde::ByteArray::cairo_deserialize(__felts, __offset)?;
        __offset += cainome::cairo_serde::ByteArray::cairo_serialized_size(&metadata_uri);
        let metadata_hash = starknet::core::types::Felt::cairo_deserialize(__felts, __offset)?;
        __offset += starknet::core::types::Felt::cairo_serialized_size(&metadata_hash);
        Ok(ResourceMetadataValue { metadata_uri, metadata_hash })
    }
}
#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct Struct {
    pub name: starknet::core::types::Felt,
    pub attrs: Vec<starknet::core::types::Felt>,
    pub children: Vec<Member>,
}
impl cainome::cairo_serde::CairoSerde for Struct {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        let mut __size = 0;
        __size += starknet::core::types::Felt::cairo_serialized_size(&__rust.name);
        __size += Vec::<starknet::core::types::Felt>::cairo_serialized_size(&__rust.attrs);
        __size += Vec::<Member>::cairo_serialized_size(&__rust.children);
        __size
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet::core::types::Felt> {
        let mut __out: Vec<starknet::core::types::Felt> = vec![];
        __out.extend(starknet::core::types::Felt::cairo_serialize(&__rust.name));
        __out.extend(Vec::<starknet::core::types::Felt>::cairo_serialize(&__rust.attrs));
        __out.extend(Vec::<Member>::cairo_serialize(&__rust.children));
        __out
    }
    fn cairo_deserialize(
        __felts: &[starknet::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let mut __offset = __offset;
        let name = starknet::core::types::Felt::cairo_deserialize(__felts, __offset)?;
        __offset += starknet::core::types::Felt::cairo_serialized_size(&name);
        let attrs = Vec::<starknet::core::types::Felt>::cairo_deserialize(__felts, __offset)?;
        __offset += Vec::<starknet::core::types::Felt>::cairo_serialized_size(&attrs);
        let children = Vec::<Member>::cairo_deserialize(__felts, __offset)?;
        __offset += Vec::<Member>::cairo_serialized_size(&children);
        Ok(Struct { name, attrs, children })
    }
}
#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub enum Event {}
impl cainome::cairo_serde::CairoSerde for Event {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = std::option::Option::None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        match __rust {
            _ => 0,
        }
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet::core::types::Felt> {
        match __rust {
            _ => vec![],
        }
    }
    fn cairo_deserialize(
        __felts: &[starknet::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let __f = __felts[__offset];
        let __index = u128::from_be_bytes(__f.to_bytes_be()[16..].try_into().unwrap());
        match __index as usize {
            _ => {
                return Err(cainome::cairo_serde::Error::Deserialize(format!(
                    "Index not handle for enum {}",
                    "Event"
                )));
            }
        }
    }
}
impl TryFrom<&starknet::core::types::EmittedEvent> for Event {
    type Error = String;
    fn try_from(event: &starknet::core::types::EmittedEvent) -> Result<Self, Self::Error> {
        use cainome::cairo_serde::CairoSerde;
        if event.keys.is_empty() {
            return Err("Event has no key".to_string());
        }
        Err(format!("Could not match any event from keys {:?}", event.keys))
    }
}
impl TryFrom<&starknet::core::types::Event> for Event {
    type Error = String;
    fn try_from(event: &starknet::core::types::Event) -> Result<Self, Self::Error> {
        use cainome::cairo_serde::CairoSerde;
        if event.keys.is_empty() {
            return Err("Event has no key".to_string());
        }
        Err(format!("Could not match any event from keys {:?}", event.keys))
    }
}
#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub enum Layout {
    Fixed(Vec<u8>),
    Struct(Vec<FieldLayout>),
    Tuple(Vec<Layout>),
    Array(Vec<Layout>),
    ByteArray,
    Enum(Vec<FieldLayout>),
}
impl cainome::cairo_serde::CairoSerde for Layout {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = std::option::Option::None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        match __rust {
            Layout::Fixed(val) => Vec::<u8>::cairo_serialized_size(val) + 1,
            Layout::Struct(val) => Vec::<FieldLayout>::cairo_serialized_size(val) + 1,
            Layout::Tuple(val) => Vec::<Layout>::cairo_serialized_size(val) + 1,
            Layout::Array(val) => Vec::<Layout>::cairo_serialized_size(val) + 1,
            Layout::ByteArray => 1,
            Layout::Enum(val) => Vec::<FieldLayout>::cairo_serialized_size(val) + 1,
            _ => 0,
        }
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet::core::types::Felt> {
        match __rust {
            Layout::Fixed(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&0usize));
                temp.extend(Vec::<u8>::cairo_serialize(val));
                temp
            }
            Layout::Struct(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&1usize));
                temp.extend(Vec::<FieldLayout>::cairo_serialize(val));
                temp
            }
            Layout::Tuple(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&2usize));
                temp.extend(Vec::<Layout>::cairo_serialize(val));
                temp
            }
            Layout::Array(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&3usize));
                temp.extend(Vec::<Layout>::cairo_serialize(val));
                temp
            }
            Layout::ByteArray => usize::cairo_serialize(&4usize),
            Layout::Enum(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&5usize));
                temp.extend(Vec::<FieldLayout>::cairo_serialize(val));
                temp
            }
            _ => vec![],
        }
    }
    fn cairo_deserialize(
        __felts: &[starknet::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let __f = __felts[__offset];
        let __index = u128::from_be_bytes(__f.to_bytes_be()[16..].try_into().unwrap());
        match __index as usize {
            0usize => Ok(Layout::Fixed(Vec::<u8>::cairo_deserialize(__felts, __offset + 1)?)),
            1usize => {
                Ok(Layout::Struct(Vec::<FieldLayout>::cairo_deserialize(__felts, __offset + 1)?))
            }
            2usize => Ok(Layout::Tuple(Vec::<Layout>::cairo_deserialize(__felts, __offset + 1)?)),
            3usize => Ok(Layout::Array(Vec::<Layout>::cairo_deserialize(__felts, __offset + 1)?)),
            4usize => Ok(Layout::ByteArray),
            5usize => {
                Ok(Layout::Enum(Vec::<FieldLayout>::cairo_deserialize(__felts, __offset + 1)?))
            }
            _ => {
                return Err(cainome::cairo_serde::Error::Deserialize(format!(
                    "Index not handle for enum {}",
                    "Layout"
                )));
            }
        }
    }
}
#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub enum Ty {
    Primitive(starknet::core::types::Felt),
    Struct(Struct),
    Enum(Enum),
    Tuple(Vec<Ty>),
    Array(Vec<Ty>),
    ByteArray,
}
impl cainome::cairo_serde::CairoSerde for Ty {
    type RustType = Self;
    const SERIALIZED_SIZE: std::option::Option<usize> = std::option::Option::None;
    #[inline]
    fn cairo_serialized_size(__rust: &Self::RustType) -> usize {
        match __rust {
            Ty::Primitive(val) => starknet::core::types::Felt::cairo_serialized_size(val) + 1,
            Ty::Struct(val) => Struct::cairo_serialized_size(val) + 1,
            Ty::Enum(val) => Enum::cairo_serialized_size(val) + 1,
            Ty::Tuple(val) => Vec::<Ty>::cairo_serialized_size(val) + 1,
            Ty::Array(val) => Vec::<Ty>::cairo_serialized_size(val) + 1,
            Ty::ByteArray => 1,
            _ => 0,
        }
    }
    fn cairo_serialize(__rust: &Self::RustType) -> Vec<starknet::core::types::Felt> {
        match __rust {
            Ty::Primitive(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&0usize));
                temp.extend(starknet::core::types::Felt::cairo_serialize(val));
                temp
            }
            Ty::Struct(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&1usize));
                temp.extend(Struct::cairo_serialize(val));
                temp
            }
            Ty::Enum(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&2usize));
                temp.extend(Enum::cairo_serialize(val));
                temp
            }
            Ty::Tuple(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&3usize));
                temp.extend(Vec::<Ty>::cairo_serialize(val));
                temp
            }
            Ty::Array(val) => {
                let mut temp = vec![];
                temp.extend(usize::cairo_serialize(&4usize));
                temp.extend(Vec::<Ty>::cairo_serialize(val));
                temp
            }
            Ty::ByteArray => usize::cairo_serialize(&5usize),
            _ => vec![],
        }
    }
    fn cairo_deserialize(
        __felts: &[starknet::core::types::Felt],
        __offset: usize,
    ) -> cainome::cairo_serde::Result<Self::RustType> {
        let __f = __felts[__offset];
        let __index = u128::from_be_bytes(__f.to_bytes_be()[16..].try_into().unwrap());
        match __index as usize {
            0usize => Ok(Ty::Primitive(starknet::core::types::Felt::cairo_deserialize(
                __felts,
                __offset + 1,
            )?)),
            1usize => Ok(Ty::Struct(Struct::cairo_deserialize(__felts, __offset + 1)?)),
            2usize => Ok(Ty::Enum(Enum::cairo_deserialize(__felts, __offset + 1)?)),
            3usize => Ok(Ty::Tuple(Vec::<Ty>::cairo_deserialize(__felts, __offset + 1)?)),
            4usize => Ok(Ty::Array(Vec::<Ty>::cairo_deserialize(__felts, __offset + 1)?)),
            5usize => Ok(Ty::ByteArray),
            _ => {
                return Err(cainome::cairo_serde::Error::Deserialize(format!(
                    "Index not handle for enum {}",
                    "Ty"
                )));
            }
        }
    }
}
impl<A: starknet::accounts::ConnectedAccount + Sync> ModelContract<A> {
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn definition(&self) -> cainome::cairo_serde::call::FCall<A::Provider, ModelDef> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("definition"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn dojo_name(
        &self,
    ) -> cainome::cairo_serde::call::FCall<A::Provider, cainome::cairo_serde::ByteArray> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("dojo_name"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn ensure_abi(
        &self,
        model: &ResourceMetadata,
    ) -> cainome::cairo_serde::call::FCall<A::Provider, ()> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        __calldata.extend(ResourceMetadata::cairo_serialize(model));
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("ensure_abi"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn ensure_unique(&self) -> cainome::cairo_serde::call::FCall<A::Provider, ()> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("ensure_unique"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn ensure_values(
        &self,
        value: &ResourceMetadataValue,
    ) -> cainome::cairo_serde::call::FCall<A::Provider, ()> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        __calldata.extend(ResourceMetadataValue::cairo_serialize(value));
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("ensure_values"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn layout(&self) -> cainome::cairo_serde::call::FCall<A::Provider, Layout> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("layout"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn packed_size(&self) -> cainome::cairo_serde::call::FCall<A::Provider, Option<u32>> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("packed_size"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn schema(&self) -> cainome::cairo_serde::call::FCall<A::Provider, Struct> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("schema"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn unpacked_size(&self) -> cainome::cairo_serde::call::FCall<A::Provider, Option<u32>> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("unpacked_size"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
}
impl<P: starknet::providers::Provider + Sync> ModelContractReader<P> {
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn definition(&self) -> cainome::cairo_serde::call::FCall<P, ModelDef> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("definition"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn dojo_name(
        &self,
    ) -> cainome::cairo_serde::call::FCall<P, cainome::cairo_serde::ByteArray> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("dojo_name"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn ensure_abi(&self, model: &ResourceMetadata) -> cainome::cairo_serde::call::FCall<P, ()> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        __calldata.extend(ResourceMetadata::cairo_serialize(model));
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("ensure_abi"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn ensure_unique(&self) -> cainome::cairo_serde::call::FCall<P, ()> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("ensure_unique"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn ensure_values(
        &self,
        value: &ResourceMetadataValue,
    ) -> cainome::cairo_serde::call::FCall<P, ()> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        __calldata.extend(ResourceMetadataValue::cairo_serialize(value));
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("ensure_values"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn layout(&self) -> cainome::cairo_serde::call::FCall<P, Layout> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("layout"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn packed_size(&self) -> cainome::cairo_serde::call::FCall<P, Option<u32>> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("packed_size"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn schema(&self) -> cainome::cairo_serde::call::FCall<P, Struct> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("schema"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::too_many_arguments)]
    pub fn unpacked_size(&self) -> cainome::cairo_serde::call::FCall<P, Option<u32>> {
        use cainome::cairo_serde::CairoSerde;
        let mut __calldata = vec![];
        let __call = starknet::core::types::FunctionCall {
            contract_address: self.address,
            entry_point_selector: starknet::macros::selector!("unpacked_size"),
            calldata: __calldata,
        };
        cainome::cairo_serde::call::FCall::new(__call, self.provider())
    }
}
