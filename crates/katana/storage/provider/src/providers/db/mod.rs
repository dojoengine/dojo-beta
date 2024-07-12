pub mod state;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Range, RangeInclusive};

use katana_db::abstraction::{Database, DbCursor, DbCursorMut, DbDupSortCursor, DbTx, DbTxMut};
use katana_db::error::DatabaseError;
use katana_db::mdbx::DbEnv;
use katana_db::models::block::StoredBlockBodyIndices;
use katana_db::models::contract::{
    ContractClassChange, ContractInfoChangeList, ContractNonceChange,
};
use katana_db::models::list::BlockList;
use katana_db::models::storage::{ContractStorageEntry, ContractStorageKey, StorageEntry};
use katana_db::tables::{self, DupSort, Table};
use katana_db::utils::KeyValue;
use katana_primitives::block::{
    Block, BlockHash, BlockHashOrNumber, BlockNumber, BlockWithTxHashes, FinalityStatus, Header,
    SealedBlockWithStatus,
};
use katana_primitives::class::{ClassHash, CompiledClassHash};
use katana_primitives::contract::{
    ContractAddress, GenericContractInfo, Nonce, StorageKey, StorageValue,
};
use katana_primitives::env::BlockEnv;
use katana_primitives::receipt::Receipt;
use katana_primitives::state::{StateUpdates, StateUpdatesWithDeclaredClasses};
use katana_primitives::trace::TxExecInfo;
use katana_primitives::transaction::{TxHash, TxNumber, TxWithHash};
use katana_primitives::FieldElement;

use crate::error::ProviderError;
use crate::traits::block::{
    BlockHashProvider, BlockNumberProvider, BlockProvider, BlockStatusProvider, BlockWriter,
    HeaderProvider,
};
use crate::traits::env::BlockEnvProvider;
use crate::traits::state::{StateFactoryProvider, StateProvider, StateRootProvider};
use crate::traits::state_update::StateUpdateProvider;
use crate::traits::transaction::{
    ReceiptProvider, TransactionProvider, TransactionStatusProvider, TransactionTraceProvider,
    TransactionsProviderExt,
};
use crate::traits::{Provider, ProviderFactory, ProviderMut};
use crate::{BlockchainProvider, ProviderResult};

#[derive(Debug)]
pub struct DbProviderFactory {
    db: DbEnv,
}

impl DbProviderFactory {
    pub fn new(db: DbEnv) -> Self {
        Self { db }
    }
}

impl ProviderFactory for DbProviderFactory {
    fn provider(&self) -> ProviderResult<BlockchainProvider<Box<dyn Provider>>> {
        let provider = Box::new(DbProvider::new(self.db.tx()?));
        Ok(BlockchainProvider::new(provider as Box<dyn Provider>))
    }

    fn provider_mut(&self) -> ProviderResult<BlockchainProvider<Box<dyn ProviderMut>>> {
        let provider = Box::new(DbProvider::new(self.db.tx_mut()?));
        Ok(BlockchainProvider::new(provider as Box<dyn ProviderMut>))
    }
}

/// A provider implementation that uses a persistent database as the backend.
// TODO: remove the default generic type
pub struct DbProvider<Tx>(Tx);

impl<Tx: DbTx> DbProvider<Tx> {
    pub fn new(tx: Tx) -> Self {
        Self(tx)
    }
}

impl<Tx> StateFactoryProvider for DbProvider<Tx>
where
    Tx: DbTx + Clone + 'static,
{
    fn latest(&self) -> ProviderResult<Box<dyn StateProvider>> {
        Ok(Box::new(self::state::LatestStateProvider::new(self.0.clone())))
    }

    fn historical(
        &self,
        block_id: BlockHashOrNumber,
    ) -> ProviderResult<Option<Box<dyn StateProvider>>> {
        let block_number = match block_id {
            BlockHashOrNumber::Num(num) => {
                let latest_num = self.latest_number()?;

                match num.cmp(&latest_num) {
                    std::cmp::Ordering::Less => Some(num),
                    std::cmp::Ordering::Greater => return Ok(None),
                    std::cmp::Ordering::Equal => return self.latest().map(Some),
                }
            }

            BlockHashOrNumber::Hash(hash) => self.block_number_by_hash(hash)?,
        };

        let Some(num) = block_number else { return Ok(None) };

        Ok(Some(Box::new(self::state::HistoricalStateProvider::new(self.0.clone(), num))))
    }
}

impl<Tx: DbTx> BlockNumberProvider for DbProvider<Tx> {
    fn block_number_by_hash(&self, hash: BlockHash) -> ProviderResult<Option<BlockNumber>> {
        Ok(self.0.get::<tables::BlockNumbers>(hash)?)
    }

    fn latest_number(&self) -> ProviderResult<BlockNumber> {
        let res = self.0.cursor::<tables::BlockHashes>()?.last()?.map(|(num, _)| num);
        let total_blocks = res.ok_or(ProviderError::MissingLatestBlockNumber)?;
        Ok(total_blocks)
    }
}

impl<Tx: DbTx> BlockHashProvider for DbProvider<Tx> {
    fn latest_hash(&self) -> ProviderResult<BlockHash> {
        let latest_block = self.latest_number()?;
        let res = self.0.get::<tables::BlockHashes>(latest_block)?;
        res.ok_or(ProviderError::MissingLatestBlockHash)
    }

    fn block_hash_by_num(&self, num: BlockNumber) -> ProviderResult<Option<BlockHash>> {
        let block_hash = self.0.get::<tables::BlockHashes>(num)?;
        Ok(block_hash)
    }
}

impl<Tx: DbTx> HeaderProvider for DbProvider<Tx> {
    fn header(&self, id: BlockHashOrNumber) -> ProviderResult<Option<Header>> {
        let num = match id {
            BlockHashOrNumber::Num(num) => Some(num),
            BlockHashOrNumber::Hash(hash) => self.0.get::<tables::BlockNumbers>(hash)?,
        };

        if let Some(num) = num {
            let header = self
                .0
                .get::<tables::Headers>(num)?
                .ok_or(ProviderError::MissingBlockHeader(num))?;
            Ok(Some(header))
        } else {
            Ok(None)
        }
    }
}

impl<Tx: DbTx> BlockProvider for DbProvider<Tx> {
    fn block_body_indices(
        &self,
        id: BlockHashOrNumber,
    ) -> ProviderResult<Option<StoredBlockBodyIndices>> {
        let block_num = match id {
            BlockHashOrNumber::Num(num) => Some(num),
            BlockHashOrNumber::Hash(hash) => self.0.get::<tables::BlockNumbers>(hash)?,
        };

        if let Some(num) = block_num {
            let indices = self.0.get::<tables::BlockBodyIndices>(num)?;
            Ok(indices)
        } else {
            Ok(None)
        }
    }

    fn block(&self, id: BlockHashOrNumber) -> ProviderResult<Option<Block>> {
        if let Some(header) = self.header(id)? {
            let res = self.transactions_by_block(id)?;
            let body = res.ok_or(ProviderError::MissingBlockTxs(header.number))?;
            Ok(Some(Block { header, body }))
        } else {
            Ok(None)
        }
    }

    fn block_with_tx_hashes(
        &self,
        id: BlockHashOrNumber,
    ) -> ProviderResult<Option<BlockWithTxHashes>> {
        let block_num = match id {
            BlockHashOrNumber::Num(num) => Some(num),
            BlockHashOrNumber::Hash(hash) => self.0.get::<tables::BlockNumbers>(hash)?,
        };

        let Some(block_num) = block_num else { return Ok(None) };

        if let Some(header) = self.0.get::<tables::Headers>(block_num)? {
            let res = self.0.get::<tables::BlockBodyIndices>(block_num)?;
            let body_indices = res.ok_or(ProviderError::MissingBlockTxs(block_num))?;

            let body = self.transaction_hashes_in_range(Range::from(body_indices))?;
            let block = BlockWithTxHashes { header, body };

            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    fn blocks_in_range(&self, range: RangeInclusive<u64>) -> ProviderResult<Vec<Block>> {
        let total = range.end() - range.start() + 1;
        let mut blocks = Vec::with_capacity(total as usize);

        for num in range {
            if let Some(header) = self.0.get::<tables::Headers>(num)? {
                let res = self.0.get::<tables::BlockBodyIndices>(num)?;
                let body_indices = res.ok_or(ProviderError::MissingBlockBodyIndices(num))?;

                let body = self.transaction_in_range(Range::from(body_indices))?;
                blocks.push(Block { header, body })
            }
        }

        Ok(blocks)
    }
}

impl<Tx: DbTx> BlockStatusProvider for DbProvider<Tx> {
    fn block_status(&self, id: BlockHashOrNumber) -> ProviderResult<Option<FinalityStatus>> {
        let block_num = match id {
            BlockHashOrNumber::Num(num) => Some(num),
            BlockHashOrNumber::Hash(hash) => self.block_number_by_hash(hash)?,
        };

        if let Some(block_num) = block_num {
            let res = self.0.get::<tables::BlockStatusses>(block_num)?;
            let status = res.ok_or(ProviderError::MissingBlockStatus(block_num))?;
            Ok(Some(status))
        } else {
            Ok(None)
        }
    }
}

impl<Tx: DbTx> StateRootProvider for DbProvider<Tx> {
    fn state_root(&self, block_id: BlockHashOrNumber) -> ProviderResult<Option<FieldElement>> {
        let block_num = match block_id {
            BlockHashOrNumber::Num(num) => Some(num),
            BlockHashOrNumber::Hash(hash) => self.0.get::<tables::BlockNumbers>(hash)?,
        };

        if let Some(block_num) = block_num {
            let header = self.0.get::<tables::Headers>(block_num)?;
            Ok(header.map(|h| h.state_root))
        } else {
            Ok(None)
        }
    }
}

impl<Tx: DbTx> StateUpdateProvider for DbProvider<Tx> {
    fn state_update(&self, block_id: BlockHashOrNumber) -> ProviderResult<Option<StateUpdates>> {
        // A helper function that iterates over all entries in a dupsort table and collects the
        // results into `V`. If `key` is not found, `V::default()` is returned.
        fn dup_entries<Tb, V, T>(
            db_tx: &impl DbTx,
            key: <Tb as Table>::Key,
            f: impl FnMut(Result<KeyValue<Tb>, DatabaseError>) -> ProviderResult<T>,
        ) -> ProviderResult<V>
        where
            Tb: DupSort + Debug,
            V: FromIterator<T> + Default,
        {
            Ok(db_tx
                .cursor_dup::<Tb>()?
                .walk_dup(Some(key), None)?
                .map(|walker| walker.map(f).collect::<ProviderResult<V>>())
                .transpose()?
                .unwrap_or_default())
        }

        let block_num = self.block_number_by_id(block_id)?;

        if let Some(block_num) = block_num {
            let nonce_updates = dup_entries::<
                tables::NonceChangeHistory,
                HashMap<ContractAddress, Nonce>,
                _,
            >(&self.0, block_num, |entry| {
                let (_, ContractNonceChange { contract_address, nonce }) = entry?;
                Ok((contract_address, nonce))
            })?;

            let contract_updates = dup_entries::<
                tables::ClassChangeHistory,
                HashMap<ContractAddress, ClassHash>,
                _,
            >(&self.0, block_num, |entry| {
                let (_, ContractClassChange { contract_address, class_hash }) = entry?;
                Ok((contract_address, class_hash))
            })?;

            let declared_classes = dup_entries::<
                tables::ClassDeclarations,
                HashMap<ClassHash, CompiledClassHash>,
                _,
            >(&self.0, block_num, |entry| {
                let (_, class_hash) = entry?;

                let compiled_hash = self
                    .0
                    .get::<tables::CompiledClassHashes>(class_hash)?
                    .ok_or(ProviderError::MissingCompiledClassHash(class_hash))?;

                Ok((class_hash, compiled_hash))
            })?;

            let storage_updates = {
                let entries = dup_entries::<
                    tables::StorageChangeHistory,
                    Vec<(ContractAddress, (StorageKey, StorageValue))>,
                    _,
                >(&self.0, block_num, |entry| {
                    let (_, ContractStorageEntry { key, value }) = entry?;
                    Ok((key.contract_address, (key.key, value)))
                })?;

                let mut map: HashMap<_, HashMap<StorageKey, StorageValue>> = HashMap::new();

                entries.into_iter().for_each(|(addr, (key, value))| {
                    map.entry(addr).or_default().insert(key, value);
                });

                map
            };

            Ok(Some(StateUpdates {
                nonce_updates,
                storage_updates,
                contract_updates,
                declared_classes,
            }))
        } else {
            Ok(None)
        }
    }
}

impl<Tx: DbTx> TransactionProvider for DbProvider<Tx> {
    fn transaction_by_hash(&self, hash: TxHash) -> ProviderResult<Option<TxWithHash>> {
        if let Some(num) = self.0.get::<tables::TxNumbers>(hash)? {
            let res = self.0.get::<tables::Transactions>(num)?;
            let transaction = res.ok_or(ProviderError::MissingTx(num))?;
            Ok(Some(TxWithHash { hash, transaction }))
        } else {
            Ok(None)
        }
    }

    fn transactions_by_block(
        &self,
        block_id: BlockHashOrNumber,
    ) -> ProviderResult<Option<Vec<TxWithHash>>> {
        if let Some(indices) = self.block_body_indices(block_id)? {
            Ok(Some(self.transaction_in_range(Range::from(indices))?))
        } else {
            Ok(None)
        }
    }

    fn transaction_in_range(&self, range: Range<TxNumber>) -> ProviderResult<Vec<TxWithHash>> {
        let total = range.end - range.start;
        let mut transactions = Vec::with_capacity(total as usize);

        for i in range {
            if let Some(transaction) = self.0.get::<tables::Transactions>(i)? {
                let res = self.0.get::<tables::TxHashes>(i)?;
                let hash = res.ok_or(ProviderError::MissingTxHash(i))?;

                transactions.push(TxWithHash { hash, transaction });
            };
        }

        Ok(transactions)
    }

    fn transaction_block_num_and_hash(
        &self,
        hash: TxHash,
    ) -> ProviderResult<Option<(BlockNumber, BlockHash)>> {
        if let Some(num) = self.0.get::<tables::TxNumbers>(hash)? {
            let block_num =
                self.0.get::<tables::TxBlocks>(num)?.ok_or(ProviderError::MissingTxBlock(num))?;

            let res = self.0.get::<tables::BlockHashes>(block_num)?;
            let block_hash = res.ok_or(ProviderError::MissingBlockHash(num))?;

            Ok(Some((block_num, block_hash)))
        } else {
            Ok(None)
        }
    }

    fn transaction_by_block_and_idx(
        &self,
        block_id: BlockHashOrNumber,
        idx: u64,
    ) -> ProviderResult<Option<TxWithHash>> {
        match self.block_body_indices(block_id)? {
            // make sure the requested idx is within the range of the block tx count
            Some(indices) if idx < indices.tx_count => {
                let num = indices.tx_offset + idx;

                let res = self.0.get::<tables::TxHashes>(num)?;
                let hash = res.ok_or(ProviderError::MissingTxHash(num))?;

                let res = self.0.get::<tables::Transactions>(num)?;
                let transaction = res.ok_or(ProviderError::MissingTx(num))?;

                Ok(Some(TxWithHash { hash, transaction }))
            }

            _ => Ok(None),
        }
    }

    fn transaction_count_by_block(
        &self,
        block_id: BlockHashOrNumber,
    ) -> ProviderResult<Option<u64>> {
        if let Some(indices) = self.block_body_indices(block_id)? {
            Ok(Some(indices.tx_count))
        } else {
            Ok(None)
        }
    }
}

impl<Tx: DbTx> TransactionsProviderExt for DbProvider<Tx> {
    fn transaction_hashes_in_range(&self, range: Range<TxNumber>) -> ProviderResult<Vec<TxHash>> {
        let total = range.end - range.start;
        let mut hashes = Vec::with_capacity(total as usize);

        for i in range {
            if let Some(hash) = self.0.get::<tables::TxHashes>(i)? {
                hashes.push(hash);
            }
        }

        Ok(hashes)
    }
}

impl<Tx: DbTx> TransactionStatusProvider for DbProvider<Tx> {
    fn transaction_status(&self, hash: TxHash) -> ProviderResult<Option<FinalityStatus>> {
        if let Some(tx_num) = self.0.get::<tables::TxNumbers>(hash)? {
            let res = self.0.get::<tables::TxBlocks>(tx_num)?;
            let block_num = res.ok_or(ProviderError::MissingTxBlock(tx_num))?;

            let res = self.0.get::<tables::BlockStatusses>(block_num)?;
            let status = res.ok_or(ProviderError::MissingBlockStatus(block_num))?;

            Ok(Some(status))
        } else {
            Ok(None)
        }
    }
}

impl<Tx: DbTx> TransactionTraceProvider for DbProvider<Tx> {
    fn transaction_execution(&self, hash: TxHash) -> ProviderResult<Option<TxExecInfo>> {
        if let Some(num) = self.0.get::<tables::TxNumbers>(hash)? {
            let execution = self
                .0
                .get::<tables::TxTraces>(num)?
                .ok_or(ProviderError::MissingTxExecution(num))?;

            Ok(Some(execution))
        } else {
            Ok(None)
        }
    }

    fn transaction_executions_by_block(
        &self,
        block_id: BlockHashOrNumber,
    ) -> ProviderResult<Option<Vec<TxExecInfo>>> {
        if let Some(index) = self.block_body_indices(block_id)? {
            let traces = self.transaction_executions_in_range(index.into())?;
            Ok(Some(traces))
        } else {
            Ok(None)
        }
    }

    fn transaction_executions_in_range(
        &self,
        range: Range<TxNumber>,
    ) -> ProviderResult<Vec<TxExecInfo>> {
        let total = range.end - range.start;
        let mut traces = Vec::with_capacity(total as usize);

        for i in range {
            if let Some(trace) = self.0.get::<tables::TxTraces>(i)? {
                traces.push(trace);
            }
        }

        Ok(traces)
    }
}

impl<Tx: DbTx> ReceiptProvider for DbProvider<Tx> {
    fn receipt_by_hash(&self, hash: TxHash) -> ProviderResult<Option<Receipt>> {
        if let Some(num) = self.0.get::<tables::TxNumbers>(hash)? {
            let receipt =
                self.0.get::<tables::Receipts>(num)?.ok_or(ProviderError::MissingTxReceipt(num))?;

            Ok(Some(receipt))
        } else {
            Ok(None)
        }
    }

    fn receipts_by_block(
        &self,
        block_id: BlockHashOrNumber,
    ) -> ProviderResult<Option<Vec<Receipt>>> {
        if let Some(indices) = self.block_body_indices(block_id)? {
            let mut receipts = Vec::with_capacity(indices.tx_count as usize);

            let range = indices.tx_offset..indices.tx_offset + indices.tx_count;
            for i in range {
                if let Some(receipt) = self.0.get::<tables::Receipts>(i)? {
                    receipts.push(receipt);
                }
            }

            Ok(Some(receipts))
        } else {
            Ok(None)
        }
    }
}

impl<Tx: DbTx> BlockEnvProvider for DbProvider<Tx> {
    fn block_env_at(&self, block_id: BlockHashOrNumber) -> ProviderResult<Option<BlockEnv>> {
        let Some(header) = self.header(block_id)? else { return Ok(None) };

        Ok(Some(BlockEnv {
            number: header.number,
            timestamp: header.timestamp,
            l1_gas_prices: header.gas_prices,
            sequencer_address: header.sequencer_address,
        }))
    }
}

impl<Tx: DbTxMut> BlockWriter for DbProvider<Tx> {
    fn insert_block_with_states_and_receipts(
        &self,
        block: SealedBlockWithStatus,
        states: StateUpdatesWithDeclaredClasses,
        receipts: Vec<Receipt>,
        executions: Vec<TxExecInfo>,
    ) -> ProviderResult<()> {
        let block_hash = block.block.header.hash;
        let block_number = block.block.header.header.number;

        let block_header = block.block.header.header;
        let transactions = block.block.body;

        let tx_count = transactions.len() as u64;
        let tx_offset = self.0.entries::<tables::Transactions>()? as u64;
        let block_body_indices = StoredBlockBodyIndices { tx_offset, tx_count };

        self.0.put::<tables::BlockHashes>(block_number, block_hash)?;
        self.0.put::<tables::BlockNumbers>(block_hash, block_number)?;
        self.0.put::<tables::BlockStatusses>(block_number, block.status)?;

        self.0.put::<tables::Headers>(block_number, block_header)?;
        self.0.put::<tables::BlockBodyIndices>(block_number, block_body_indices)?;

        for (i, (transaction, receipt, execution)) in transactions
            .into_iter()
            .zip(receipts.into_iter())
            .zip(executions.into_iter())
            .map(|((transaction, receipt), execution)| (transaction, receipt, execution))
            .enumerate()
        {
            let tx_number = tx_offset + i as u64;
            let tx_hash = transaction.hash;

            self.0.put::<tables::TxHashes>(tx_number, tx_hash)?;
            self.0.put::<tables::TxNumbers>(tx_hash, tx_number)?;
            self.0.put::<tables::TxBlocks>(tx_number, block_number)?;
            self.0.put::<tables::Transactions>(tx_number, transaction.transaction)?;
            self.0.put::<tables::Receipts>(tx_number, receipt)?;
            self.0.put::<tables::TxTraces>(tx_number, execution)?;
        }

        // insert classes

        for (class_hash, compiled_hash) in states.state_updates.declared_classes {
            self.0.put::<tables::CompiledClassHashes>(class_hash, compiled_hash)?;

            self.0.put::<tables::ClassDeclarationBlock>(class_hash, block_number)?;
            self.0.put::<tables::ClassDeclarations>(block_number, class_hash)?
        }

        for (hash, compiled_class) in states.declared_compiled_classes {
            self.0.put::<tables::CompiledClasses>(hash, compiled_class)?;
        }

        for (class_hash, sierra_class) in states.declared_sierra_classes {
            self.0.put::<tables::SierraClasses>(class_hash, sierra_class)?;
        }

        // insert storage changes
        {
            let mut storage_cursor = self.0.cursor_dup_mut::<tables::ContractStorage>()?;
            for (addr, entries) in states.state_updates.storage_updates {
                let entries = entries.into_iter().map(|(key, value)| StorageEntry { key, value });

                for entry in entries {
                    match storage_cursor.seek_by_key_subkey(addr, entry.key)? {
                        Some(current) if current.key == entry.key => {
                            storage_cursor.delete_current()?;
                        }

                        _ => {}
                    }

                    // update block list in the change set
                    let changeset_key =
                        ContractStorageKey { contract_address: addr, key: entry.key };
                    let list = self.0.get::<tables::StorageChangeSet>(changeset_key.clone())?;

                    let updated_list = match list {
                        Some(mut list) => {
                            list.insert(block_number);
                            list
                        }
                        // create a new block list if it doesn't yet exist, and insert the block
                        // number
                        None => BlockList::from([block_number]),
                    };

                    self.0.put::<tables::StorageChangeSet>(changeset_key, updated_list)?;
                    storage_cursor.upsert(addr, entry)?;

                    let storage_change_sharded_key =
                        ContractStorageKey { contract_address: addr, key: entry.key };

                    self.0.put::<tables::StorageChangeHistory>(
                        block_number,
                        ContractStorageEntry {
                            key: storage_change_sharded_key,
                            value: entry.value,
                        },
                    )?;
                }
            }
        }

        // update contract info

        for (addr, class_hash) in states.state_updates.contract_updates {
            let value = if let Some(info) = self.0.get::<tables::ContractInfo>(addr)? {
                GenericContractInfo { class_hash, ..info }
            } else {
                GenericContractInfo { class_hash, ..Default::default() }
            };

            let new_change_set =
                if let Some(mut change_set) = self.0.get::<tables::ContractInfoChangeSet>(addr)? {
                    change_set.class_change_list.insert(block_number);
                    change_set
                } else {
                    ContractInfoChangeList {
                        class_change_list: BlockList::from([block_number]),
                        ..Default::default()
                    }
                };

            self.0.put::<tables::ContractInfo>(addr, value)?;

            let class_change_key = ContractClassChange { contract_address: addr, class_hash };
            self.0.put::<tables::ClassChangeHistory>(block_number, class_change_key)?;
            self.0.put::<tables::ContractInfoChangeSet>(addr, new_change_set)?;
        }

        for (addr, nonce) in states.state_updates.nonce_updates {
            let value = if let Some(info) = self.0.get::<tables::ContractInfo>(addr)? {
                GenericContractInfo { nonce, ..info }
            } else {
                GenericContractInfo { nonce, ..Default::default() }
            };

            let new_change_set =
                if let Some(mut change_set) = self.0.get::<tables::ContractInfoChangeSet>(addr)? {
                    change_set.nonce_change_list.insert(block_number);
                    change_set
                } else {
                    ContractInfoChangeList {
                        nonce_change_list: BlockList::from([block_number]),
                        ..Default::default()
                    }
                };

            self.0.put::<tables::ContractInfo>(addr, value)?;

            let nonce_change_key = ContractNonceChange { contract_address: addr, nonce };
            self.0.put::<tables::NonceChangeHistory>(block_number, nonce_change_key)?;
            self.0.put::<tables::ContractInfoChangeSet>(addr, new_change_set)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use katana_db::mdbx::DbEnvKind;
    use katana_primitives::block::{
        Block, BlockHashOrNumber, FinalityStatus, Header, SealedBlockWithStatus,
    };
    use katana_primitives::contract::ContractAddress;
    use katana_primitives::fee::TxFeeInfo;
    use katana_primitives::receipt::{InvokeTxReceipt, Receipt};
    use katana_primitives::state::{StateUpdates, StateUpdatesWithDeclaredClasses};
    use katana_primitives::trace::TxExecInfo;
    use katana_primitives::transaction::{InvokeTx, Tx, TxHash, TxWithHash};
    use starknet::core::types::PriceUnit;
    use starknet::macros::felt;

    use super::DbProvider;
    use crate::traits::block::{
        BlockHashProvider, BlockNumberProvider, BlockProvider, BlockStatusProvider, BlockWriter,
    };
    use crate::traits::state::StateFactoryProvider;
    use crate::traits::transaction::TransactionProvider;

    fn create_dummy_block() -> SealedBlockWithStatus {
        let header = Header { parent_hash: 199u8.into(), number: 0, ..Default::default() };
        let block = Block {
            header,
            body: vec![TxWithHash {
                hash: 24u8.into(),
                transaction: Tx::Invoke(InvokeTx::V1(Default::default())),
            }],
        }
        .seal();
        SealedBlockWithStatus { block, status: FinalityStatus::AcceptedOnL2 }
    }

    fn create_dummy_state_updates() -> StateUpdatesWithDeclaredClasses {
        StateUpdatesWithDeclaredClasses {
            state_updates: StateUpdates {
                nonce_updates: HashMap::from([
                    (ContractAddress::from(felt!("1")), felt!("1")),
                    (ContractAddress::from(felt!("2")), felt!("2")),
                ]),
                contract_updates: HashMap::from([
                    (ContractAddress::from(felt!("1")), felt!("3")),
                    (ContractAddress::from(felt!("2")), felt!("4")),
                ]),
                declared_classes: HashMap::from([
                    (felt!("3"), felt!("89")),
                    (felt!("4"), felt!("90")),
                ]),
                storage_updates: HashMap::from([(
                    ContractAddress::from(felt!("1")),
                    HashMap::from([(felt!("1"), felt!("1")), (felt!("2"), felt!("2"))]),
                )]),
            },
            ..Default::default()
        }
    }

    fn create_dummy_state_updates_2() -> StateUpdatesWithDeclaredClasses {
        StateUpdatesWithDeclaredClasses {
            state_updates: StateUpdates {
                nonce_updates: HashMap::from([
                    (ContractAddress::from(felt!("1")), felt!("5")),
                    (ContractAddress::from(felt!("2")), felt!("6")),
                ]),
                contract_updates: HashMap::from([
                    (ContractAddress::from(felt!("1")), felt!("77")),
                    (ContractAddress::from(felt!("2")), felt!("66")),
                ]),
                storage_updates: HashMap::from([(
                    ContractAddress::from(felt!("1")),
                    HashMap::from([(felt!("1"), felt!("100")), (felt!("2"), felt!("200"))]),
                )]),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn create_db_provider() -> DbProvider {
        DbProvider(katana_db::mdbx::test_utils::create_test_db(DbEnvKind::RW))
    }

    #[test]
    fn insert_block() {
        let provider = create_db_provider();

        let block = create_dummy_block();
        let state_updates = create_dummy_state_updates();

        // insert block
        BlockWriter::insert_block_with_states_and_receipts(
            &provider,
            block.clone(),
            state_updates,
            vec![Receipt::Invoke(InvokeTxReceipt {
                revert_error: None,
                events: Vec::new(),
                messages_sent: Vec::new(),
                execution_resources: Default::default(),
                fee: TxFeeInfo {
                    gas_consumed: 0,
                    gas_price: 0,
                    overall_fee: 0,
                    unit: PriceUnit::Wei,
                },
            })],
            vec![TxExecInfo::default()],
        )
        .expect("failed to insert block");

        // get values

        let block_id: BlockHashOrNumber = block.block.header.hash.into();

        let latest_number = provider.latest_number().unwrap();
        let latest_hash = provider.latest_hash().unwrap();

        let actual_block = provider.block(block_id).unwrap().unwrap();
        let tx_count = provider.transaction_count_by_block(block_id).unwrap().unwrap();
        let block_status = provider.block_status(block_id).unwrap().unwrap();
        let body_indices = provider.block_body_indices(block_id).unwrap().unwrap();

        let tx_hash: TxHash = 24u8.into();
        let tx = provider.transaction_by_hash(tx_hash).unwrap().unwrap();

        let state_prov = StateFactoryProvider::latest(&provider).unwrap();

        let nonce1 = state_prov.nonce(ContractAddress::from(felt!("1"))).unwrap().unwrap();
        let nonce2 = state_prov.nonce(ContractAddress::from(felt!("2"))).unwrap().unwrap();

        let class_hash1 = state_prov.class_hash_of_contract(felt!("1").into()).unwrap().unwrap();
        let class_hash2 = state_prov.class_hash_of_contract(felt!("2").into()).unwrap().unwrap();

        let compiled_hash1 =
            state_prov.compiled_class_hash_of_class_hash(class_hash1).unwrap().unwrap();
        let compiled_hash2 =
            state_prov.compiled_class_hash_of_class_hash(class_hash2).unwrap().unwrap();

        let storage1 =
            state_prov.storage(ContractAddress::from(felt!("1")), felt!("1")).unwrap().unwrap();
        let storage2 =
            state_prov.storage(ContractAddress::from(felt!("1")), felt!("2")).unwrap().unwrap();

        // assert values are populated correctly

        assert_eq!(tx_hash, tx.hash);
        assert_eq!(tx.transaction, Tx::Invoke(InvokeTx::V1(Default::default())));

        assert_eq!(tx_count, 1);
        assert_eq!(body_indices.tx_offset, 0);
        assert_eq!(body_indices.tx_count, tx_count);

        assert_eq!(block_status, FinalityStatus::AcceptedOnL2);
        assert_eq!(block.block.header.hash, latest_hash);
        assert_eq!(block.block.body.len() as u64, tx_count);
        assert_eq!(block.block.header.header.number, latest_number);
        assert_eq!(block.block.unseal(), actual_block);

        assert_eq!(nonce1, felt!("1"));
        assert_eq!(nonce2, felt!("2"));
        assert_eq!(class_hash1, felt!("3"));
        assert_eq!(class_hash2, felt!("4"));

        assert_eq!(compiled_hash1, felt!("89"));
        assert_eq!(compiled_hash2, felt!("90"));

        assert_eq!(storage1, felt!("1"));
        assert_eq!(storage2, felt!("2"));
    }

    #[test]
    fn storage_updated_correctly() {
        let provider = create_db_provider();

        let block = create_dummy_block();
        let state_updates1 = create_dummy_state_updates();
        let state_updates2 = create_dummy_state_updates_2();

        // insert block
        BlockWriter::insert_block_with_states_and_receipts(
            &provider,
            block.clone(),
            state_updates1,
            vec![Receipt::Invoke(InvokeTxReceipt {
                revert_error: None,
                events: Vec::new(),
                messages_sent: Vec::new(),
                execution_resources: Default::default(),
                fee: TxFeeInfo {
                    gas_consumed: 0,
                    gas_price: 0,
                    overall_fee: 0,
                    unit: PriceUnit::Wei,
                },
            })],
            vec![TxExecInfo::default()],
        )
        .expect("failed to insert block");

        // insert another block
        BlockWriter::insert_block_with_states_and_receipts(
            &provider,
            block,
            state_updates2,
            vec![Receipt::Invoke(InvokeTxReceipt {
                revert_error: None,
                events: Vec::new(),
                messages_sent: Vec::new(),
                execution_resources: Default::default(),
                fee: TxFeeInfo {
                    gas_consumed: 0,
                    gas_price: 0,
                    overall_fee: 0,
                    unit: PriceUnit::Wei,
                },
            })],
            vec![TxExecInfo::default()],
        )
        .expect("failed to insert block");

        // assert storage is updated correctly

        let state_prov = StateFactoryProvider::latest(&provider).unwrap();

        let nonce1 = state_prov.nonce(ContractAddress::from(felt!("1"))).unwrap().unwrap();
        let nonce2 = state_prov.nonce(ContractAddress::from(felt!("2"))).unwrap().unwrap();

        let class_hash1 = state_prov.class_hash_of_contract(felt!("1").into()).unwrap().unwrap();
        let class_hash2 = state_prov.class_hash_of_contract(felt!("2").into()).unwrap().unwrap();

        let storage1 =
            state_prov.storage(ContractAddress::from(felt!("1")), felt!("1")).unwrap().unwrap();
        let storage2 =
            state_prov.storage(ContractAddress::from(felt!("1")), felt!("2")).unwrap().unwrap();

        assert_eq!(nonce1, felt!("5"));
        assert_eq!(nonce2, felt!("6"));

        assert_eq!(class_hash1, felt!("77"));
        assert_eq!(class_hash2, felt!("66"));

        assert_eq!(storage1, felt!("100"));
        assert_eq!(storage2, felt!("200"));
    }
}
