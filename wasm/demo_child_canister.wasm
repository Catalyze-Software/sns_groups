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

#[query]
#[candid_method(query)]
fn get_available_canister() -> Result<ScalableCanisterDetails, String> {
    ScalableData::get_available_canister(caller())
}

#[query]
#[candid_method(query)]
fn get_canisters() -> Vec<ScalableCanisterDetails> {
    ScalableData::get_canisters()
}

#[update]
#[candid_method(update)]
async fn close_child_canister_and_spawn_sibling(
    owner: Principal,
    last_entry_id: u64,
    entry: Vec<u8>,
    principal_entry_reference: Option<Principal>,
) -> Result<Principal, ApiError> {
    ScalableData::close_child_canister_and_spawn_sibling(
        caller(),
        owner,
        last_entry_id,
        entry,
        principal_entry_reference,
    )
    .await
}

#[query]
#[candid_method(query)]
fn get_latest_wasm_version() -> WasmVersion {
    DATA.with(|v| v.borrow().child_wasm_data.wasm_version.clone())
}

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

#[update]
#[candid_method(update)]
fn accept_cycles() -> u64 {
    Canister::accept_cycles()
}
