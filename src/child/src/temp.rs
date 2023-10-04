use std::borrow::Cow;

use candid::{CandidType, Decode, Encode, Principal};
use ic_cdk::api::time;
use ic_stable_structures::{storable::Bound, Storable};
use serde::Deserialize;

#[derive(CandidType, Clone, Deserialize)]
pub struct StableData {
    // The name of the canister
    pub name: String,
    // identifier of the specific canister
    pub identifier: usize,
    // The current entry id
    pub current_entry_id: u64,
    // Entry id range in canister
    pub parent: Principal,
    // The data entries
    pub is_available: bool,
    // updated_at record
    pub updated_at: u64,
    // created_at record
    pub created_at: u64,
}

impl Storable for StableData {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Default for StableData {
    fn default() -> Self {
        StableData {
            name: String::default(),
            identifier: 0,
            current_entry_id: 0,
            parent: Principal::anonymous(),
            is_available: bool::default(),
            updated_at: time(),
            created_at: time(),
        }
    }
}
