use candid::{candid_method, Principal};
use ic_cdk::caller;
use ic_cdk_macros::{query, update};
use ic_scalable_misc::{
    enums::{api_error_type::ApiError, wasm_version_type::WasmVersion},
    helpers::{
        canister_helper::Canister,
        metrics_helper::{http_request as _http_request, metrics, PathEntry},
    },
    models::{
        canister_models::ScalableCanisterDetails,
        http_models::{HeaderField, HttpRequest, HttpResponse},
    },
};

use super::store::{ScalableData, DATA};

// Method to retrieve an available canister to write updated to
#[query]
#[candid_method(query)]
fn get_available_canister() -> Result<ScalableCanisterDetails, String> {
    ScalableData::get_available_canister(caller())
}

// Methods to retrieve all the canisters
#[query]
#[candid_method(query)]
fn get_canisters() -> Vec<ScalableCanisterDetails> {
    ScalableData::get_canisters()
}

// Method called by child canister once full (inter-canister call)
// can only be called by a child canister
#[update]
#[candid_method(update)]
async fn close_child_canister_and_spawn_sibling(
    last_entry_id: u64,
    entry: Vec<u8>,
) -> Result<Principal, ApiError> {
    ScalableData::close_child_canister_and_spawn_sibling(caller(), last_entry_id, entry).await
}

// Method to retrieve the latest wasm version of the child canister that is currently stored
#[query]
#[candid_method(query)]
fn get_latest_wasm_version() -> WasmVersion {
    DATA.with(|v| v.borrow().child_wasm_data.wasm_version.clone())
}

// HTTP request handler
// canister metrics are added to the response
#[query]
#[candid_method(query)]
fn http_request(req: HttpRequest) -> HttpResponse {
    let path_entries = vec![PathEntry {
        match_path: vec!["metrics".to_string()],
        response: HttpResponse {
            status_code: 200,
            headers: vec![HeaderField(
                "content-type".to_string(),
                "text/plain".to_string(),
            )],
            body: metrics(vec![]).as_bytes().to_vec(),
        },
    }];

    _http_request(req, path_entries)
}

// Method to accept cycles when send to this canister
#[update]
#[candid_method(update)]
fn accept_cycles() -> u64 {
    Canister::accept_cycles()
}
