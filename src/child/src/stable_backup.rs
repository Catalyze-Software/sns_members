use crate::backup::is_owner;
use crate::store::{ENTRIES, STABLE_DATA};
use ic_canister_backup::{
    canister_backup::{ENTRIES_BACKUP, STABLE_DATA_BACKUP},
    models::Chunk,
};
use ic_cdk::{query, update};
use ic_scalable_canister::store::Data;
use shared::member_model::Member;

#[update(guard = "is_owner")]
fn canister_backup_data() -> (String, String) {
    let stable_data_hash = STABLE_DATA.with(|cell| {
        let cell = cell.borrow();
        let data = cell.get();
        let serialized = serde_cbor::to_vec(&data).unwrap();

        // immediate deserialize check
        let _: Data = serde_cbor::from_slice(&serialized).unwrap();

        STABLE_DATA_BACKUP.with(|b| b.borrow_mut().backup_data(serialized))
    });

    let entries_hash = ENTRIES.with(|tree| {
        let data: Vec<(String, Member)> = tree
            .borrow()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let serialized = serde_cbor::to_vec(&data).unwrap();

        // immediate deserialize check
        let _: Vec<(String, Member)> = serde_cbor::from_slice(&serialized).unwrap();

        ENTRIES_BACKUP.with(|b| b.borrow_mut().backup_data(serialized))
    });

    (stable_data_hash, entries_hash)
}

#[query(guard = "is_owner")]
fn total_stable_data_chunks() -> u64 {
    STABLE_DATA_BACKUP.with(|b| b.borrow().total_chunks() as u64)
}

#[query(guard = "is_owner")]
fn download_stable_data_chunk(n: u64) -> Chunk {
    STABLE_DATA_BACKUP.with(|b| b.borrow().download_chunk(n))
}

#[query(guard = "is_owner")]
fn total_entries_chunks() -> u64 {
    ENTRIES_BACKUP.with(|b| b.borrow().total_chunks() as u64)
}

#[query(guard = "is_owner")]
fn download_entries_chunk(n: u64) -> Chunk {
    ENTRIES_BACKUP.with(|b| b.borrow().download_chunk(n))
}
