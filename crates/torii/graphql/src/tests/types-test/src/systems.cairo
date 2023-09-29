use dojo::world::{IWorldDispatcher, IWorldDispatcherTrait};
use starknet::{ContractAddress, ClassHash};

#[starknet::interface]
trait IRecords<TContractState> {
    fn create(self: @TContractState, world: IWorldDispatcher, num_records: u8);
}

#[system]
mod records {
    use types_test::models::{Record, Nested, MoreNested};
    use super::IRecords;

    #[external(v0)]
    impl RecordsImpl of IRecords<ContractState> {
        fn create(self: @ContractState, world: IWorldDispatcher, num_records: u8) {
            let mut record_idx = 0;
            loop {
                if record_idx == num_records {
                    break ();
                }

                let type_felt: felt252 = record_idx.into();

                set!(
                    world,
                    (Record {
                        record_id: world.uuid(),
                        type_u8: record_idx.into(),
                        type_u16: record_idx.into(),
                        type_u32: record_idx.into(),
                        type_u64: record_idx.into(),
                        type_u128: record_idx.into(),
                        //type_u256: curr_record.into(),
                        type_bool: if record_idx % 2 == 0 {
                            true
                        } else {
                            false
                        },
                        type_felt: record_idx.into(),
                        type_class_hash: type_felt.try_into().unwrap(),
                        type_contract_address: type_felt.try_into().unwrap(),
                        type_nested: Nested {
                            type_number: record_idx.into(),
                            type_string: type_felt,
                            type_more_nested: MoreNested {
                                type_number: record_idx.into(), type_string: type_felt,
                            }
                        }
                    })
                );

                record_idx += 1;
            };
            return ();
        }
    }
}
