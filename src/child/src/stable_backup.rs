use crate::backup::is_owner;
use crate::store::{ENTRIES, STABLE_DATA};
use ic_canister_backup::{
    canister_backup::{ENTRIES_BACKUP, STABLE_DATA_BACKUP},
    models::Chunk,
};
use ic_cdk::{query, update};
use ic_scalable_canister::store::Data;
use shared::group_model::Group;

/*
* BACKUP LOGIC
*/
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
        let data: Vec<(String, Group)> = tree
            .borrow()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let serialized = serde_cbor::to_vec(&data).unwrap();

        // immediate deserialize check
        let _: Vec<(String, Group)> = serde_cbor::from_slice(&serialized).unwrap();

        ENTRIES_BACKUP.with(|b| b.borrow_mut().backup_data(serialized))
    });

    (stable_data_hash, entries_hash)
}

/*
* BACKUP METHODS
*/
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

/*
* RESTORE METHODS
*/
// #[update(guard = "is_owner")]
// fn canister_clear_backup() {
//     STABLE_DATA_BACKUP.with(|b| b.borrow_mut().clear_backup());
//     ENTRIES_BACKUP.with(|b| b.borrow_mut().clear_backup());
// }

// #[update(guard = "is_owner")]
// fn upload_stable_data_chunk(chunk: Chunk) {
//     STABLE_DATA_BACKUP.with(|b| b.borrow_mut().upload_chunk(chunk));
// }

// #[update(guard = "is_owner")]
// fn upload_entries_chunk(chunk: Chunk) {
//     ENTRIES_BACKUP.with(|b| b.borrow_mut().upload_chunk(chunk));
// }

// #[update(guard = "is_owner")]
// fn canister_finalize_upload(stable_data_hash: Vec<u8>, entries_hash: Vec<u8>) {
//     let computed_stable_data_hash = STABLE_DATA_BACKUP.with(|b| b.borrow_mut().finalize_upload());
//     let computed_entries_hash = ENTRIES_BACKUP.with(|b| b.borrow_mut().finalize_upload());

//     assert_eq!(stable_data_hash, computed_stable_data_hash);
//     assert_eq!(entries_hash, computed_entries_hash);
// }

/*
* RESTORE LOGIC
*/
// #[update(guard = "is_owner")]
// fn canister_restore_data() {
//     let stable_data = STABLE_DATA_BACKUP.with(|b| b.borrow_mut().get_serialized_restore_data());
//     let entries = ENTRIES_BACKUP.with(|b| b.borrow_mut().get_serialized_restore_data());

//     let stable_data: Data = serde_cbor::from_slice(&stable_data).expect("Failed to deserialize");
//     let entries: Vec<(String, Member)> =
//         serde_cbor::from_slice(&entries).expect("Failed to deserialize");

//     STABLE_DATA.with(|cell| {
//         let mut cell = cell.borrow_mut();
//         cell.set(stable_data).expect("Failed to set stable data");
//     });

//     ENTRIES.with(|tree| {
//         // Workaround since `.clear()` takes ownership
//         let _ = tree.replace(StableBTreeMap::new(
//             MEMORY_MANAGER.with(|m| m.borrow().get(ENTRIES_MEMORY_ID)),
//         ));

//         for (k, v) in entries {
//             tree.borrow_mut().insert(k, v);
//         }
//     });
// }
