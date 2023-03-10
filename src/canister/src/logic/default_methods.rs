use std::time::Duration;

use candid::{candid_method, Principal};
use ic_cdk::{storage, timer::set_timer};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query};

use super::store::{ScalableData, DATA};

#[pre_upgrade]
pub fn pre_upgrade() {
    DATA.with(|data| storage::stable_save((&*data.borrow(),)))
        .expect("Something went wrong while upgrading");
}

#[post_upgrade]
pub fn post_upgrade() {
    let (mut old_store,): (ScalableData,) = storage::stable_restore().unwrap();
    let child_wasm_data = ScalableData::get_child_wasm_data(&old_store, 0_0_2);
    match child_wasm_data {
        Ok(_child_wasm_data) => {
            DATA.with(|d| {
                old_store.child_wasm_data = _child_wasm_data;
                *d.borrow_mut() = old_store;
            });

            set_timer(Duration::from_secs(0), || {
                ic_cdk::spawn(ScalableData::upgrade_children());
            });
        }
        Err(error) => {
            ic_cdk::println!("Error: {:?}", error);
            DATA.with(|d| {
                *d.borrow_mut() = old_store;
            });
        }
    }
}

#[init]
#[candid_method(init)]
fn init(name: String, owner: Principal, parent: Principal) {
    DATA.with(|v| {
        let mut data = v.borrow_mut();
        data.name = name;
        data.owner = owner;
        data.parent = parent;
        data.child_wasm_data = ScalableData::get_child_wasm_data(&data, 0_0_1).unwrap();
        data.whitelist = vec![];
    });
    set_timer(Duration::from_secs(0), || {
        ic_cdk::spawn(ScalableData::initialize_first_child_canister());
    });
}

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
pub fn __export_did_tmp_() -> String {
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
