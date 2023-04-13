use std::time::Duration;

use candid::{candid_method, Principal};
use ic_cdk::{caller, storage, timer::set_timer};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query};
use ic_scalable_misc::{
    helpers::logger_helper::add_log,
    models::logger_models::{LogType, PostLog},
};

use super::store::{ScalableData, DATA};

// Stores the data in stable storage before upgrading the canister.
#[pre_upgrade]
pub fn pre_upgrade() {
    DATA.with(|data| storage::stable_save((&*data.borrow(),)))
        .expect("Something went wrong while upgrading");
}

// Restores the data from stable- to heap storage after upgrading the canister.
#[post_upgrade]
pub fn post_upgrade() {
    let (mut old_store,): (ScalableData,) = storage::stable_restore().unwrap();
    // Get the child wasm data from the old store
    use ic_scalable_misc::enums::wasm_version_type::WasmVersion::*;
    let version = match old_store.child_wasm_data.wasm_version {
        Version(_version) => _version + 1,
        _ => 0,
    };

    let child_wasm_data = ScalableData::get_child_wasm_data(&old_store, version);
    match child_wasm_data {
        // If the child wasm data is found, update the data in the new store
        Ok(_child_wasm_data) => {
            DATA.with(|d| {
                old_store.child_wasm_data = _child_wasm_data;
                *d.borrow_mut() = old_store;
            });

            add_log(PostLog {
                log_type: LogType::Info,
                description: "canister children upgrading".to_string(),
                source: "post_upgrade".to_string(),
                data: "".to_string(),
            });

            // Use a timer to trigger the upgrade_children method to upgrade the child WASMs
            set_timer(Duration::from_secs(0), || {
                ic_cdk::spawn(ScalableData::upgrade_children());
            });
        }
        // If the child wasm data is not found, continue restoring the old store
        Err(err) => {
            DATA.with(|d| {
                *d.borrow_mut() = old_store;
            });

            add_log(PostLog {
                log_type: LogType::Info,
                description: "No child upgrade needed".to_string(),
                source: "post_upgrade".to_string(),
                data: format!("Error: {}", err),
            });
        }
    }
}

// Init methods thats get triggered when the canister is installed
#[init]
#[candid_method(init)]
fn init() {
    DATA.with(|v| {
        let mut data = v.borrow_mut();
        data.name = "group_parent".to_string();
        data.parent = caller();
        // Set the child WASM data on first deploy from the file system
        data.child_wasm_data = ScalableData::get_child_wasm_data(&data, 0_0_1).unwrap();
    });

    // Spawn the first child canister
    set_timer(Duration::from_secs(0), || {
        ic_cdk::spawn(ScalableData::initialize_first_child_canister());
    });
}

// Hacky way to expose the candid interface to the outside world
#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
pub fn __export_did_tmp_() -> String {
    use candid::export_service;
    use ic_cdk::api::management_canister::http_request::HttpResponse;
    use ic_scalable_misc::enums::api_error_type::ApiError;
    use ic_scalable_misc::enums::filter_type::FilterType;
    use ic_scalable_misc::enums::wasm_version_type::WasmVersion;
    use ic_scalable_misc::models::canister_models::ScalableCanisterDetails;
    use ic_scalable_misc::models::http_models::HttpRequest;
    use ic_scalable_misc::models::paged_response_models::PagedResponse;
    use shared::group_model::*;
    export_service!();
    __export_service()
}

// Method used to save the candid interface to a file
#[test]
pub fn candid() {
    use ic_scalable_misc::helpers::candid_helper::save_candid;
    save_candid(__export_did_tmp_(), String::from("parent"));
}
