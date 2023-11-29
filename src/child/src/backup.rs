use crate::store::DATA;
use candid::{candid_method, Encode};
use ic_canister_backup::{logic::BACKUP, models::Chunk};
use ic_cdk::caller;
use ic_cdk_macros::{query, update};

#[update(guard = "is_owner")]
#[candid_method(update)]
fn backup_data() -> String {
    DATA.with(|data| {
        let serialized = Encode!(&*data).expect("Failed to encode data");
        BACKUP.with(|b| b.borrow_mut().backup_data(serialized))
    })
}

#[query(guard = "is_owner")]
#[candid_method(query)]
fn download_chunk(n: u64) -> Chunk {
    BACKUP.with(|b| b.borrow().download_chunk(n))
}

#[query(guard = "is_owner")]
#[candid_method(query)]
fn total_chunks() -> u64 {
    BACKUP.with(|b| b.borrow().total_chunks() as u64)
}

pub fn is_owner() -> Result<(), String> {
    const OWNERS: [&str; 1] = ["swcc7-vdu3r-tym5o-cfsiw-kpo3l-5qlgi-mq7al-xbn6l-bdspe-xjwau-wae"];

    match OWNERS.iter().any(|p| p == &caller().to_string()) {
        true => Ok(()),
        false => Err("Unauthorized".to_string()),
    }
}
