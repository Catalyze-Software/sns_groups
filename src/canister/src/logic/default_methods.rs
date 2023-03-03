use candid::{candid_method, Principal};
use ic_cdk::{api::time, storage};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query};
use ic_scalable_misc::{
    enums::whitelist_rights_type::WhitelistRights, models::whitelist_models::WhitelistEntry,
};

use super::store::{ScalableData, DATA};

#[query]
#[candid_method(query)]
fn sanity_check() -> String {
    "Scalable sane".to_string()
}

#[pre_upgrade]
pub fn pre_upgrade() {
    DATA.with(|data| storage::stable_save((&*data.borrow(),)))
        .expect("Something went wrong while upgrading");
}

#[post_upgrade]
pub fn post_upgrade() {
    let (old_store,): (ScalableData,) = storage::stable_restore().unwrap();
    DATA.with(|d| *d.borrow_mut() = old_store);
}

#[init]
#[candid_method(init)]
fn init(name: String, owner: Principal, parent: Principal) {
    DATA.with(|v| {
        let mut data = v.borrow_mut();
        data.name = name;
        data.owner = owner;
        data.parent = parent;
        data.whitelist = vec![WhitelistEntry {
            label: "Owner".to_string(),
            principal: owner,
            rights: WhitelistRights::Owner,
            created_on: time(),
        }];
    });
}

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
pub fn __export_did_tmp_() -> String {
    use crate::logic::store::ScalableMetaData;
    use crate::models::group_model::*;
    use candid::export_service;
    use ic_cdk::api::management_canister::http_request::HttpResponse;
    use ic_scalable_misc::enums::api_error_type::ApiError;
    use ic_scalable_misc::enums::filter_type::FilterType;
    use ic_scalable_misc::enums::wasm_version_type::WasmVersion;
    use ic_scalable_misc::models::canister_models::ScalableCanisterDetails;
    use ic_scalable_misc::models::http_models::HttpRequest;
    use ic_scalable_misc::models::paged_response_models::PagedResponse;
    export_service!();
    __export_service()
}
