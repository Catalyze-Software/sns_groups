use crate::store::DATA;
use candid::{candid_method, Decode, Encode};
use ic_canister_backup::{logic::BACKUP, models::Chunk};
use ic_cdk::caller;
use ic_cdk_macros::{query, update};
use ic_scalable_canister::store::Data;
use shared::group_model::Group;

#[update(guard = "is_owner")]
#[candid_method(update)]
fn backup_data() -> String {
    DATA.with(|data| {
        let serialized = Encode!(&*data).expect("Failed to encode data");
        BACKUP.with(|b| b.borrow_mut().backup_data(serialized))
    })
}

#[update(guard = "is_owner")]
#[candid_method(update)]
fn restore_data() -> () {
    let serialized = BACKUP.with(|b| b.borrow().get_serialized_restore_data());
    let decoded = Decode!(&serialized, Data<Group>).expect("Failed to decode data");
    DATA.with(|data| *data.borrow_mut() = decoded);
}

#[query(guard = "is_owner")]
#[candid_method(query)]
fn download_chunk(n: u64) -> Chunk {
    BACKUP.with(|b| b.borrow().download_chunk(n))
}

#[update(guard = "is_owner")]
#[candid_method(update)]
fn upload_chunk(chunk: Chunk) {
    BACKUP.with(|b| b.borrow_mut().upload_chunk(chunk))
}

#[update(guard = "is_owner")]
#[candid_method(update)]
fn finalize_upload() -> String {
    BACKUP.with(|b| b.borrow_mut().finalize_upload())
}

#[query(guard = "is_owner")]
#[candid_method(query)]
fn total_chunks() -> u64 {
    BACKUP.with(|b| b.borrow().total_chunks() as u64)
}

#[update(guard = "is_owner")]
#[candid_method(update)]
fn clear_backup() {
    BACKUP.with(|b| b.borrow_mut().clear_backup());
}

pub fn is_owner() -> Result<(), String> {
    const OWNERS: [&str; 1] = ["swcc7-vdu3r-tym5o-cfsiw-kpo3l-5qlgi-mq7al-xbn6l-bdspe-xjwau-wae"];

    match OWNERS.iter().any(|p| p == &caller().to_string()) {
        true => Ok(()),
        false => Err("Unauthorized".to_string()),
    }
}
