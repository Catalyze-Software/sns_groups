use candid::Decode;
use ic_canister_backup::{logic::BACKUP, models::Chunk};
use ic_cdk::{caller, query, update};
use ic_scalable_canister::{ic_scalable_misc, store::Data};
use ic_stable_structures::StableBTreeMap;
use shared::group_model::Group;

use crate::store::{ENTRIES, ENTRIES_MEMORY_ID, MEMORY_MANAGER, STABLE_DATA};

//
#[update(guard = "is_owner")]
pub fn restore_data() {
    ENTRIES.with(|n| {
        n.replace(StableBTreeMap::new(
            MEMORY_MANAGER.with(|m| m.borrow().get(ENTRIES_MEMORY_ID)),
        ))
    });

    let serialized = BACKUP.with(|b| b.borrow().get_serialized_restore_data());
    let data = Decode!(
        &serialized,
        ic_scalable_misc::models::original_data::Data<Group>
    )
    .expect("Failed to decode data");

    let _ = STABLE_DATA.with(|s| {
        s.borrow_mut().set(Data {
            name: data.name.clone(),
            identifier: data.identifier.clone(),
            current_entry_id: data.current_entry_id.clone(),
            parent: data.parent.clone(),
            is_available: data.is_available.clone(),
            updated_at: data.updated_at.clone(),
            created_at: data.created_at.clone(),
        })
    });

    let _ = ENTRIES.with(|e| {
        data.entries.iter().for_each(|entry| {
            e.borrow_mut().insert(entry.0.to_string(), entry.1.clone());
        });
    });
}

// #[update(guard = "is_owner")]
// fn backup_data() -> String {
//     ENTRIES.with(|data| {
//         let serialized = Encode!(&*data).expect("Failed to encode data");
//         BACKUP.with(|b| b.borrow_mut().backup_data(serialized))
//     })
// }

// #[update(guard = "is_owner")]
// fn restore_data() -> () {
//     let serialized = BACKUP.with(|b| b.borrow().get_serialized_restore_data());
//     let decoded = Decode!(
//         &serialized,
//         ic_scalable_misc::models::original_data::Data<Group>
//     )
//     .expect("Failed to decode data");
//     ENTRIES.with(|data| *data.borrow_mut() = decoded);
// }

#[query(guard = "is_owner")]
fn download_chunk(n: u64) -> Chunk {
    BACKUP.with(|b| b.borrow().download_chunk(n))
}

#[update(guard = "is_owner")]
fn upload_chunk(chunk: Chunk) {
    BACKUP.with(|b| b.borrow_mut().upload_chunk(chunk))
}

#[update(guard = "is_owner")]
fn finalize_upload() -> String {
    BACKUP.with(|b| b.borrow_mut().finalize_upload())
}

#[query(guard = "is_owner")]
fn total_chunks() -> u64 {
    BACKUP.with(|b| b.borrow().total_chunks() as u64)
}

#[update(guard = "is_owner")]
fn clear_backup() {
    BACKUP.with(|b| b.borrow_mut().clear_backup());
}

pub fn is_owner() -> Result<(), String> {
    const OWNERS: [&str; 2] = [
        // catalyze backup dfx principal
        "swcc7-vdu3r-tym5o-cfsiw-kpo3l-5qlgi-mq7al-xbn6l-bdspe-xjwau-wae",
        // backup_canister
        "srb2o-nqaaa-aaaap-abt3q-cai",
    ];

    match OWNERS.iter().any(|p| p == &caller().to_string()) {
        true => Ok(()),
        false => Err("Unauthorized".to_string()),
    }
}
