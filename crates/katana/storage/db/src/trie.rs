use core::fmt;
use std::fmt::Debug;
use std::marker::PhantomData;

use anyhow::Result;
use katana_primitives::block::BlockNumber;
use katana_trie::bonsai::{BonsaiDatabase, BonsaiPersistentDatabase, ByteVec, DatabaseKey};
use katana_trie::CommitId;
use smallvec::ToSmallVec;

use crate::abstraction::{DbCursor, DbTxMutRef, DbTxRef};
use crate::models::trie::{TrieDatabaseKey, TrieDatabaseKeyType};
use crate::models::{self};
use crate::tables::{self, Trie};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] crate::error::DatabaseError);

impl katana_trie::bonsai::DBError for Error {}

#[derive(Debug)]
pub struct TrieDbFactory<'a, Tx: DbTxRef<'a>> {
    tx: Tx,
    _phantom: &'a PhantomData<()>,
}

impl<'a, Tx: DbTxRef<'a>> TrieDbFactory<'a, Tx> {
    pub fn new(tx: Tx) -> Self {
        Self { tx, _phantom: &PhantomData }
    }

    pub fn latest(&self) -> GlobalTrie<'a, Tx> {
        GlobalTrie { tx: self.tx.clone(), _phantom: &PhantomData }
    }

    // TODO: check that the snapshot for the block number is available
    pub fn historical(&self, block: BlockNumber) -> Option<HistoricalGlobalTrie<'a, Tx>> {
        Some(HistoricalGlobalTrie { tx: self.tx.clone(), block, _phantom: &PhantomData })
    }
}

/// Provides access to the latest tries.
#[derive(Debug)]
pub struct GlobalTrie<'a, Tx: DbTxRef<'a>> {
    tx: Tx,
    _phantom: &'a PhantomData<()>,
}

impl<'a, Tx> GlobalTrie<'a, Tx>
where
    Tx: DbTxRef<'a> + Debug,
{
    /// Returns the contracts trie.
    pub fn contracts_trie(
        &self,
    ) -> katana_trie::ContractsTrie<TrieDb<'a, tables::ContractsTrie, Tx>> {
        katana_trie::ContractsTrie::new(TrieDb::new(self.tx.clone()))
    }

    /// Returns the classes trie.
    pub fn classes_trie(&self) -> katana_trie::ClassesTrie<TrieDb<'a, tables::ClassesTrie, Tx>> {
        katana_trie::ClassesTrie::new(TrieDb::new(self.tx.clone()))
    }

    /// Returns the storages trie.
    pub fn storages_trie(&self) -> katana_trie::StoragesTrie<TrieDb<'a, tables::StoragesTrie, Tx>> {
        katana_trie::StoragesTrie::new(TrieDb::new(self.tx.clone()))
    }
}

/// Historical tries, allowing access to the state tries at each block.
#[derive(Debug)]
pub struct HistoricalGlobalTrie<'a, Tx: DbTxRef<'a>> {
    /// The database transaction.
    tx: Tx,
    /// The block number at which the trie was constructed.
    block: BlockNumber,
    _phantom: &'a PhantomData<()>,
}

impl<'a, Tx> HistoricalGlobalTrie<'a, Tx>
where
    Tx: DbTxRef<'a> + Debug,
{
    /// Returns the historical contracts trie.
    pub fn contracts_trie(
        &self,
    ) -> katana_trie::ContractsTrie<SnapshotTrieDb<'a, tables::ContractsTrie, Tx>> {
        let commit = CommitId::new(self.block);
        katana_trie::ContractsTrie::new(SnapshotTrieDb::new(self.tx.clone(), commit))
    }

    /// Returns the historical classes trie.
    pub fn classes_trie(
        &self,
    ) -> katana_trie::ClassesTrie<SnapshotTrieDb<'a, tables::ClassesTrie, Tx>> {
        let commit = CommitId::new(self.block);
        katana_trie::ClassesTrie::new(SnapshotTrieDb::new(self.tx.clone(), commit))
    }

    /// Returns the historical storages trie.
    pub fn storages_trie(
        &self,
    ) -> katana_trie::StoragesTrie<SnapshotTrieDb<'a, tables::StoragesTrie, Tx>> {
        let commit = CommitId::new(self.block);
        katana_trie::StoragesTrie::new(SnapshotTrieDb::new(self.tx.clone(), commit))
    }
}

// --- Trie's database implementations. These are implemented based on the Bonsai Trie
// functionalities and abstractions.

pub struct TrieDb<'a, Tb, Tx>
where
    Tb: Trie,
    Tx: DbTxRef<'a>,
{
    tx: Tx,
    _phantom: &'a PhantomData<Tb>,
}

impl<'a, Tb, Tx> fmt::Debug for TrieDb<'a, Tb, Tx>
where
    Tb: Trie,
    Tx: DbTxRef<'a> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrieDbMut").field("tx", &self.tx).finish()
    }
}

impl<'a, Tb, Tx> TrieDb<'a, Tb, Tx>
where
    Tb: Trie,
    Tx: DbTxRef<'a>,
{
    pub(crate) fn new(tx: Tx) -> Self {
        Self { tx, _phantom: &PhantomData }
    }
}

impl<'a, Tb, Tx> BonsaiDatabase for TrieDb<'a, Tb, Tx>
where
    Tb: Trie,
    Tx: DbTxRef<'a> + fmt::Debug,
{
    type Batch = ();
    type DatabaseError = Error;

    fn create_batch(&self) -> Self::Batch {}

    fn remove_by_prefix(&mut self, prefix: &DatabaseKey<'_>) -> Result<(), Self::DatabaseError> {
        let _ = prefix;
        Ok(())
    }

    fn get(&self, key: &DatabaseKey<'_>) -> Result<Option<ByteVec>, Self::DatabaseError> {
        let value = self.tx.get::<Tb>(to_db_key(key))?;
        Ok(value)
    }

    fn get_by_prefix(
        &self,
        prefix: &DatabaseKey<'_>,
    ) -> Result<Vec<(ByteVec, ByteVec)>, Self::DatabaseError> {
        let _ = prefix;
        todo!()
    }

    fn insert(
        &mut self,
        key: &DatabaseKey<'_>,
        value: &[u8],
        batch: Option<&mut Self::Batch>,
    ) -> Result<Option<ByteVec>, Self::DatabaseError> {
        let _ = key;
        let _ = value;
        let _ = batch;
        unimplemented!("not supported in read-only transaction")
    }

    fn remove(
        &mut self,
        key: &DatabaseKey<'_>,
        batch: Option<&mut Self::Batch>,
    ) -> Result<Option<ByteVec>, Self::DatabaseError> {
        let _ = key;
        let _ = batch;
        unimplemented!("not supported in read-only transaction")
    }

    fn contains(&self, key: &DatabaseKey<'_>) -> Result<bool, Self::DatabaseError> {
        let key = to_db_key(key);
        let value = self.tx.get::<Tb>(key)?;
        Ok(value.is_some())
    }

    fn write_batch(&mut self, batch: Self::Batch) -> Result<(), Self::DatabaseError> {
        let _ = batch;
        unimplemented!("not supported in read-only transaction")
    }
}

impl<'tx, Tb, Tx> BonsaiPersistentDatabase<CommitId> for TrieDbMut<'tx, Tb, Tx>
where
    Tb: Trie,
    Tx: DbTxMutRef<'tx> + fmt::Debug + 'tx,
{
    type DatabaseError = Error;
    type Transaction<'a> = SnapshotTrieDb<'tx, Tb, Tx>  where Self: 'a;

    fn snapshot(&mut self, id: CommitId) {
        let _ = id;
        todo!()
    }

    // merging should recompute the trie again
    fn merge<'a>(&mut self, transaction: Self::Transaction<'a>) -> Result<(), Self::DatabaseError>
    where
        Self: 'a,
    {
        let _ = transaction;
        unimplemented!();
    }

    // TODO: check if the snapshot exist
    fn transaction(&self, id: CommitId) -> Option<(CommitId, Self::Transaction<'_>)> {
        Some((id, SnapshotTrieDb::new(self.tx.clone(), id)))
    }
}

pub struct TrieDbMut<'tx, Tb, Tx>
where
    Tb: Trie,
    Tx: DbTxMutRef<'tx>,
{
    tx: Tx,
    _phantom: &'tx PhantomData<Tb>,
}

impl<'tx, Tb, Tx> fmt::Debug for TrieDbMut<'tx, Tb, Tx>
where
    Tb: Trie,
    Tx: DbTxMutRef<'tx> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrieDbMut").field("tx", &self.tx).finish()
    }
}

impl<'tx, Tb, Tx> TrieDbMut<'tx, Tb, Tx>
where
    Tb: Trie,
    Tx: DbTxMutRef<'tx>,
{
    pub fn new(tx: Tx) -> Self {
        Self { tx, _phantom: &PhantomData }
    }
}

impl<'tx, Tb, Tx> BonsaiDatabase for TrieDbMut<'tx, Tb, Tx>
where
    Tb: Trie,
    Tx: DbTxMutRef<'tx> + fmt::Debug,
{
    type Batch = ();
    type DatabaseError = Error;

    fn create_batch(&self) -> Self::Batch {}

    fn remove_by_prefix(&mut self, prefix: &DatabaseKey<'_>) -> Result<(), Self::DatabaseError> {
        let mut cursor = self.tx.cursor_mut::<Tb>()?;
        let walker = cursor.walk(None)?;

        let mut keys_to_remove = Vec::new();
        // iterate over all entries in the table
        for entry in walker {
            let (key, _) = entry?;
            if key.key.starts_with(prefix.as_slice()) {
                keys_to_remove.push(key);
            }
        }

        for key in keys_to_remove {
            let _ = self.tx.delete::<Tb>(key, None)?;
        }

        Ok(())
    }

    fn get(&self, key: &DatabaseKey<'_>) -> Result<Option<ByteVec>, Self::DatabaseError> {
        let value = self.tx.get::<Tb>(to_db_key(key))?;
        Ok(value)
    }

    fn get_by_prefix(
        &self,
        prefix: &DatabaseKey<'_>,
    ) -> Result<Vec<(ByteVec, ByteVec)>, Self::DatabaseError> {
        let _ = prefix;
        todo!()
    }

    fn insert(
        &mut self,
        key: &DatabaseKey<'_>,
        value: &[u8],
        _batch: Option<&mut Self::Batch>,
    ) -> Result<Option<ByteVec>, Self::DatabaseError> {
        let key = to_db_key(key);
        let value: ByteVec = value.to_smallvec();
        let old_value = self.tx.get::<Tb>(key.clone())?;
        self.tx.put::<Tb>(key, value)?;
        Ok(old_value)
    }

    fn remove(
        &mut self,
        key: &DatabaseKey<'_>,
        _batch: Option<&mut Self::Batch>,
    ) -> Result<Option<ByteVec>, Self::DatabaseError> {
        let key = to_db_key(key);
        let old_value = self.tx.get::<Tb>(key.clone())?;
        self.tx.delete::<Tb>(key, None)?;
        Ok(old_value)
    }

    fn contains(&self, key: &DatabaseKey<'_>) -> Result<bool, Self::DatabaseError> {
        let key = to_db_key(key);
        let value = self.tx.get::<Tb>(key)?;
        Ok(value.is_some())
    }

    fn write_batch(&mut self, batch: Self::Batch) -> Result<(), Self::DatabaseError> {
        let _ = batch;
        Ok(())
    }
}

pub struct SnapshotTrieDb<'tx, Tb, Tx>
where
    Tb: Trie,
    Tx: DbTxRef<'tx>,
{
    tx: Tx,
    snapshot_id: CommitId,
    _table: &'tx PhantomData<Tb>,
}

impl<'a, Tb, Tx> fmt::Debug for SnapshotTrieDb<'a, Tb, Tx>
where
    Tb: Trie,
    Tx: DbTxRef<'a> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotTrieDb").field("tx", &self.tx).finish()
    }
}

impl<'tx, Tb, Tx> SnapshotTrieDb<'tx, Tb, Tx>
where
    Tb: Trie,
    Tx: DbTxRef<'tx>,
{
    pub(crate) fn new(tx: Tx, id: CommitId) -> Self {
        Self { tx, snapshot_id: id, _table: &PhantomData }
    }
}

impl<'tx, Tb, Tx> BonsaiDatabase for SnapshotTrieDb<'tx, Tb, Tx>
where
    Tb: Trie,
    Tx: DbTxRef<'tx> + fmt::Debug,
{
    type Batch = ();
    type DatabaseError = Error;

    fn create_batch(&self) -> Self::Batch {}

    fn remove_by_prefix(&mut self, prefix: &DatabaseKey<'_>) -> Result<(), Self::DatabaseError> {
        let _ = prefix;
        unimplemented!("modifying trie snapshot is not supported")
    }

    fn get(&self, key: &DatabaseKey<'_>) -> Result<Option<ByteVec>, Self::DatabaseError> {
        todo!()
    }

    fn get_by_prefix(
        &self,
        prefix: &DatabaseKey,
    ) -> Result<Vec<(ByteVec, ByteVec)>, Self::DatabaseError> {
        todo!()
    }

    fn insert(
        &mut self,
        key: &DatabaseKey<'_>,
        value: &[u8],
        batch: Option<&mut Self::Batch>,
    ) -> Result<Option<ByteVec>, Self::DatabaseError> {
        let _ = key;
        let _ = value;
        let _ = batch;
        unimplemented!("modifying trie snapshot is not supported")
    }

    fn remove(
        &mut self,
        key: &DatabaseKey<'_>,
        batch: Option<&mut Self::Batch>,
    ) -> Result<Option<ByteVec>, Self::DatabaseError> {
        let _ = key;
        let _ = batch;
        unimplemented!("modifying trie snapshot is not supported")
    }

    fn contains(&self, key: &DatabaseKey<'_>) -> Result<bool, Self::DatabaseError> {
        todo!()
    }

    fn write_batch(&mut self, batch: Self::Batch) -> Result<(), Self::DatabaseError> {
        let _ = batch;
        unimplemented!("modifying trie snapshot is not supported")
    }
}

fn to_db_key(key: &DatabaseKey<'_>) -> models::trie::TrieDatabaseKey {
    match key {
        DatabaseKey::Flat(bytes) => {
            TrieDatabaseKey { key: bytes.to_vec(), r#type: TrieDatabaseKeyType::Flat }
        }
        DatabaseKey::Trie(bytes) => {
            TrieDatabaseKey { key: bytes.to_vec(), r#type: TrieDatabaseKeyType::Trie }
        }
        DatabaseKey::TrieLog(bytes) => {
            TrieDatabaseKey { key: bytes.to_vec(), r#type: TrieDatabaseKeyType::TrieLog }
        }
    }
}
