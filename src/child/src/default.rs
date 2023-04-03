use candid::{candid_method, export_service, Principal};
use ic_cdk::caller;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use ic_scalable_canister::{ic_methods, store::Data};
use ic_scalable_misc::{
    enums::api_error_type::ApiError,
    models::http_models::{HttpRequest, HttpResponse},
};

use crate::store::DATA;

// Stores the data in stable storage before upgrading the canister.
#[pre_upgrade]
pub fn pre_upgrade() {
    DATA.with(|data| ic_methods::pre_upgrade(data))
}

// Restores the data from stable- to heap storage after upgrading the canister.
#[post_upgrade]
pub fn post_upgrade() {
    DATA.with(|data| ic_methods::post_upgrade(data))
}

// This call get triggered when a new canister is spun up
// the data is passed along to the new canister as a byte array
#[update]
#[candid_method(update)]
async fn add_entry_by_parent(entry: Vec<u8>) -> Result<(), ApiError> {
    DATA.with(|v| Data::add_entry_by_parent(v, caller(), entry, Some("grp".to_string())))
}

// Method to accept cycles when send to this canister
#[update]
#[candid_method(update)]
fn accept_cycles() -> u64 {
    ic_methods::accept_cycles()
}

// HTTP request handler, canister metrics are added to the response by default
// can be extended by adding `Vec<PathEntry>` as a third parameter
#[query]
#[candid_method(query)]
fn http_request(req: HttpRequest) -> HttpResponse {
    DATA.with(|data| Data::http_request_with_metrics(data, req, vec![]))
}

// Hacky way to expose the candid interface to the outside world
#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
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
#[candid_method(init)]
pub fn init(parent: Principal, name: String, identifier: usize) {
    DATA.with(|data| {
        ic_methods::init(data, parent, name, identifier);
    })
}

// Method used to save the candid interface to a file
#[test]
pub fn candid() {
    use ic_scalable_misc::helpers::candid_helper::save_candid;
    save_candid(__export_did_tmp_(), String::from("child"));
}
