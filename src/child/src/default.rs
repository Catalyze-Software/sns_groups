use candid::{candid_method, export_service, Principal};
use ic_cdk::caller;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use ic_scalable_canister::{ic_methods, store::Data};
use ic_scalable_misc::{
    enums::api_error_type::ApiError,
    models::http_models::{HttpRequest, HttpResponse},
};

use crate::store::DATA;

#[pre_upgrade]
pub fn pre_upgrade() {
    DATA.with(|data| ic_methods::pre_upgrade(data))
}

#[post_upgrade]
pub fn post_upgrade() {
    DATA.with(|data| ic_methods::post_upgrade(data))
}

#[update]
#[candid_method(update)]
async fn add_entry_by_parent(principal: Option<Principal>, entry: Vec<u8>) -> Result<(), ApiError> {
    DATA.with(|v| Data::add_entry_by_parent(v, caller(), principal, entry, Some("grp".to_string())))
}

#[update]
#[candid_method(update)]
fn accept_cycles() -> u64 {
    ic_methods::accept_cycles()
}

#[query]
#[candid_method(query)]
fn http_request(req: HttpRequest) -> HttpResponse {
    DATA.with(|data| Data::http_request_with_metrics(data, req, vec![]))
}

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

#[init]
#[candid_method(init)]
pub fn init(parent: Principal, name: String, identifier: usize) {
    DATA.with(|data| {
        ic_methods::init(data, parent, name, identifier);
    })
}

#[test]
pub fn candid() {
    use ic_scalable_misc::helpers::candid_helper::save_candid;
    save_candid(__export_did_tmp_(), String::from("child"));
}
