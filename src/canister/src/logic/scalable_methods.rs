use candid::{candid_method, Principal};
use ic_cdk::caller;
use ic_cdk_macros::{query, update};
use ic_scalable_misc::{
    enums::{
        api_error_type::ApiError, wasm_version_type::WasmVersion,
        whitelist_rights_type::WhitelistRights,
    },
    helpers::{
        canister_helper::Canister,
        metrics_helper::{http_request as _http_request, metrics, PathEntry},
    },
    models::{
        canister_models::ScalableCanisterDetails,
        http_models::{HeaderField, HttpRequest, HttpResponse},
        paged_response_models::PagedResponse,
        wasm_models::WasmDetails,
        whitelist_models::WhitelistEntry,
    },
};

use super::store::{ScalableData, ScalableMetaData, DATA};

#[query]
#[candid_method(query)]
fn get_metadata() -> Result<ScalableMetaData, ApiError> {
    ScalableData::get_metadata(caller())
}

#[update]
#[candid_method(update)]
fn change_name(name: String) -> bool {
    ScalableData::change_name(caller(), name)
}

#[query]
#[candid_method(query)]
fn get_whitelist(limit: usize, page: usize) -> Result<PagedResponse<WhitelistEntry>, ApiError> {
    ScalableData::get_whitelist(caller(), limit, page)
}

#[update]
#[candid_method(update)]
async fn add_to_whitelist(
    label: String,
    principal: Principal,
    rights: WhitelistRights,
) -> Result<bool, ApiError> {
    ScalableData::add_to_whitelist(caller(), label, principal, rights)
}

#[update]
#[candid_method(update)]
fn remove_from_whitelist(principal: Principal) -> Result<bool, ApiError> {
    ScalableData::remove_from_whitelist(caller(), principal)
}

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

#[query]
#[candid_method(query)]
fn get_wasms() -> Result<WasmDetails, ApiError> {
    ScalableData::get_wasm(caller())
}

#[update]
#[candid_method(update)]
fn add_wasm(label: String, bytes: Vec<u8>) -> Result<bool, ApiError> {
    ScalableData::add_wasm(caller(), label, bytes)
}

#[update]
#[candid_method(update)]
async fn initialize_first_child_canister() -> Result<Principal, ApiError> {
    ScalableData::initialize_first_child_canister(caller()).await
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

#[update]
#[candid_method(update)]
async fn upgrade_child_canister(
    canister_principal: Principal,
) -> Result<ScalableCanisterDetails, ApiError> {
    ScalableData::upgrade_child_canister(caller(), canister_principal).await
}

#[update]
#[candid_method(update)]
async fn reinstall_child_canister(canister_principal: Principal) -> Result<Principal, ApiError> {
    ScalableData::reinstall_child_canister(caller(), canister_principal).await
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
