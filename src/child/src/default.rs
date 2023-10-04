use candid::{export_service, Principal};
use ic_cdk::{caller, init, query, update};
use ic_scalable_canister::{ic_methods, store::Data};
use ic_scalable_misc::{
    enums::api_error_type::ApiError,
    models::http_models::{HttpRequest, HttpResponse},
};

use crate::{
    store::{DATA, ENTRIES},
    IDENTIFIER_KIND,
};

// #[update]
// pub fn migrate_to_stable(code: String) {
//     if code != "rem.codes" {
//         return;
//     }

//     let data: Data<Group> = DATA.with(|d| d.borrow().clone());
//     let _ = STABLE_DATA.with(|s| {
//         s.borrow_mut().set(StableData {
//             name: data.name.clone(),
//             identifier: data.identifier.clone(),
//             current_entry_id: data.current_entry_id.clone(),
//             parent: data.parent.clone(),
//             is_available: data.is_available.clone(),
//             updated_at: data.updated_at.clone(),
//             created_at: data.created_at.clone(),
//         })
//     });

//     let _ = ENTRIES.with(|e| {
//         data.entries.iter().for_each(|entry| {
//             e.borrow_mut().insert(entry.0.to_string(), entry.1.clone());
//         });
//     });
// }

#[query]
pub fn check_stable_data() -> Data {
    DATA.with(|s| s.borrow().get().clone())
}

#[query]
pub fn check_stable_entries() -> u64 {
    ENTRIES.with(|s| s.borrow().len() as u64)
}

// This call get triggered when a new canister is spun up
// the data is passed along to the new canister as a byte array
#[update]
async fn add_entry_by_parent(entry: Vec<u8>) -> Result<(), ApiError> {
    DATA.with(|data| {
        ENTRIES.with(|entries| {
            Data::add_entry_by_parent(
                data,
                entries,
                caller(),
                entry,
                Some(IDENTIFIER_KIND.to_string()),
            )
        })
    })
}

// Method to accept cycles when send to this canister
#[update]
fn accept_cycles() -> u64 {
    ic_methods::accept_cycles()
}

// HTTP request handler, canister metrics are added to the response by default
// can be extended by adding `Vec<PathEntry>` as a third parameter
#[query]
fn http_request(req: HttpRequest) -> HttpResponse {
    DATA.with(|data| {
        ENTRIES.with(|entries| Data::http_request_with_metrics(data, entries, req, vec![]))
    })
}

// Hacky way to expose the candid interface to the outside world
#[query(name = "__get_candid_interface_tmp_hack")]
pub fn __export_did_tmp_() -> String {
    use candid::export_service;
    use candid::Principal;
    use ic_cdk::api::management_canister::http_request::HttpResponse;
    use ic_scalable_misc::enums::api_error_type::ApiError;
    use ic_scalable_misc::enums::filter_type::FilterType;
    use ic_scalable_misc::enums::privacy_type::Privacy;
    use ic_scalable_misc::models::group_role::GroupRole;
    use ic_scalable_misc::models::http_models::HttpRequest;
    use ic_scalable_misc::models::paged_response_models::PagedResponse;
    use ic_scalable_misc::models::permissions_models::PostPermission;
    use shared::group_model::*;
    export_service!();
    __export_service()
}

// Init methods thats get triggered when the canister is installed
// The parent canister is the canister that spins up this canister
// the name is a simple identification of what the canister stores
// the identifier is a incremented number that is used to create a unique name for the canister combined with the name
#[init]
pub fn init(parent: Principal, name: String, identifier: usize) {
    DATA.with(|data| {
        ic_methods::init(data, parent, name, identifier);
    });
}

// Method used to save the candid interface to a file
#[test]
pub fn candid() {
    use ic_scalable_misc::helpers::candid_helper::save_candid;
    save_candid(__export_did_tmp_(), String::from("child"));
}
