use std::{
    borrow::Borrow,
    cell::{Cell, RefCell},
    collections::HashMap,
};

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::{
    api::{call, time},
    id,
};

use ic_scalable_misc::{
    enums::{
        api_error_type::{ApiError, ApiErrorType},
        canister_type::CanisterType,
        filter_type::FilterType,
        wasm_version_type::WasmVersion,
        whitelist_rights_type::WhitelistRights,
    },
    helpers::{
        canister_helper::{Canister, CanisterID, CanisterSettings, InstallCodeMode},
        error_helper::api_error,
        ic_data_helper,
        paging_helper::get_paged_data,
        serialize_helper::deserialize,
    },
    models::{
        canister_models::ScalableCanisterDetails, error_message_models::ErrorMessage,
        paged_response_models::PagedResponse, wasm_models::WasmDetails,
        whitelist_models::WhitelistEntry,
    },
};

use crate::models::group_model::{GroupFilter, GroupResponse};

#[derive(CandidType, Clone, Deserialize)]
pub struct ScalableMetaData {
    pub name: String,
    pub canister_count: usize,
    pub has_child_wasm: bool,
    pub cycles: u64,
    pub used_data: u64,
    pub owner: Principal,
    pub parent: Principal,
    pub updated_at: u64,
    pub created_at: u64,
}

#[derive(CandidType, Clone, Deserialize)]
pub struct ScalableData {
    // The name of the scalable canister (ex; users)
    pub name: String,
    // The child canisters that are used for storing the scalable data
    pub canisters: HashMap<Principal, ScalableCanisterDetails>,
    // The wasm details that need to be installed on the child canisters
    pub owner: Principal,
    // The parent canister
    pub parent: Principal,
    // The wasm details that need to be installed on the child canisters
    pub child_wasm_data: WasmDetails,
    // whitelist for administrative access, the foundation an parent canister are added by default
    pub whitelist: Vec<WhitelistEntry>,
    // updated_at record
    pub updated_at: u64,
    // created_at record
    pub created_at: u64,
}

impl Default for ScalableData {
    fn default() -> Self {
        ScalableData {
            canisters: HashMap::new(),
            name: String::default(),
            child_wasm_data: WasmDetails::default(),
            whitelist: Vec::default(),
            owner: Principal::anonymous(),
            parent: Principal::anonymous(),
            updated_at: time(),
            created_at: time(),
        }
    }
}

pub static CHILD_WASM: &[u8; 1353304] =
    include_bytes!("../../../../wasm/child_group_canister.wasm");

thread_local! {
    pub static DATA: RefCell<ScalableData> = RefCell::new(ScalableData::default());
}
impl ScalableData {
    pub fn change_name(caller: Principal, name: String) -> bool {
        if !Self::has_whitelist_rights(caller, WhitelistRights::ReadWrite) {
            return false;
        };

        if DATA.with(|v| v.borrow().canisters.len() > 1) {
            return false;
        }

        DATA.with(|v| v.borrow_mut().name = name);
        return true;
    }

    pub fn get_metadata(caller: Principal) -> Result<ScalableMetaData, ApiError> {
        let inputs = Some(vec![format!("caller - {}", &caller.to_string())]);

        if !Self::has_whitelist_rights(caller, WhitelistRights::Read) {
            return Err(Self::whitelist_error("get_whitelist".to_string(), inputs));
        };

        let result = DATA.with(|v| ScalableMetaData {
            name: v.borrow().name.clone(),
            cycles: ic_data_helper::get_cycles(),
            used_data: ic_data_helper::get_stable_memory_size(),
            updated_at: v.borrow().updated_at,
            created_at: v.borrow().created_at,
            canister_count: v.borrow().canisters.len(),
            has_child_wasm: v.borrow().child_wasm_data.bytes.len() != 0,
            owner: v.borrow().owner,
            parent: v.borrow().parent,
        });

        Ok(result)
    }

    pub fn get_whitelist(
        caller: Principal,
        limit: usize,
        page: usize,
    ) -> Result<PagedResponse<WhitelistEntry>, ApiError> {
        let inputs = Some(vec![
            format!("caller - {}", &caller.to_string()),
            format!("limit - {}", &limit.to_string()),
            format!("page - {}", &page.to_string()),
        ]);

        if !Self::has_whitelist_rights(caller, WhitelistRights::Read) {
            return Err(Self::whitelist_error("get_whitelist".to_string(), inputs));
        };

        let result = DATA.with(|v| {
            v.borrow()
                .whitelist
                .iter()
                .map(|entry| entry)
                .cloned()
                .collect()
        });

        Ok(get_paged_data(result, limit, page))
    }

    pub fn add_to_whitelist(
        caller: Principal,
        label: String,
        principal: Principal,
        rights: WhitelistRights,
    ) -> Result<bool, ApiError> {
        let inputs = Some(vec![
            format!("caller - {}", &caller.to_string()),
            format!("label - {}", &label),
            format!("principal - {}", &principal.to_string()),
            format!("rights - {}", &rights.to_string()),
        ]);

        if rights == WhitelistRights::Owner {
            return Err(api_error(
                ApiErrorType::BadRequest,
                "CANT_SET_RIGHTS",
                "It's not possible to set the whitelist rights to owner",
                &Self::get_name(),
                "add_to_whitelist",
                inputs,
            ));
        }

        if !Self::has_whitelist_rights(caller, WhitelistRights::Owner) {
            return Err(Self::whitelist_error(
                "add_to_whitelist".to_string(),
                inputs,
            ));
        };

        if DATA.with(|v| {
            v.borrow()
                .whitelist
                .iter()
                .any(|w| w.principal == principal)
        }) {
            return Err(api_error(
                ApiErrorType::BadRequest,
                "ALREADY_WHITELISTED",
                "This principal is already whitelisted",
                &Self::get_name(),
                "add_to_whitelist",
                inputs,
            ));
        }

        let new_entry = WhitelistEntry {
            label,
            principal,
            rights,
            created_on: time(),
        };

        DATA.with(|v| v.borrow_mut().whitelist.push(new_entry));
        Ok(true)
    }

    pub fn remove_from_whitelist(
        caller: Principal,
        principal: Principal,
    ) -> Result<bool, ApiError> {
        let inputs = Some(vec![
            format!("caller - {}", &caller.to_string()),
            format!("principal - {}", &principal.to_string()),
        ]);

        let whitelist_entry = DATA.with(|v| {
            v.borrow()
                .whitelist
                .iter()
                .find(|w| w.principal == principal)
                .cloned()
        });

        match whitelist_entry {
            None => Err(api_error(
                ApiErrorType::NotFound,
                "WHITELIST_ENTRY_NOT_FOUND",
                "The whitelist entry is not found",
                &Self::get_name(),
                "remove_from_whitelist",
                inputs,
            )),
            Some(_entry) => {
                if _entry.rights == WhitelistRights::Owner {
                    return Err(api_error(
                        ApiErrorType::BadRequest,
                        "CANT_SET_RIGHTS",
                        "It's not possible to set the whitelist rights to owner",
                        &Self::get_name(),
                        "remove_from_whitelist",
                        inputs,
                    ));
                }

                if !Self::has_whitelist_rights(caller, WhitelistRights::Owner) {
                    return Err(Self::whitelist_error(
                        "remove_from_whitelist".to_string(),
                        inputs,
                    ));
                };

                let filtered_whitelist: Vec<WhitelistEntry> = DATA.with(|v| {
                    v.borrow()
                        .whitelist
                        .iter()
                        .filter(|w| w.principal != principal)
                        .cloned()
                        .collect()
                });

                DATA.with(|v| v.borrow_mut().whitelist = filtered_whitelist);
                Ok(true)
            }
        }
    }

    pub fn get_name() -> String {
        DATA.with(|v| v.borrow().name.clone())
    }

    pub fn _get_data() -> ScalableData {
        DATA.with(|v| v.borrow().clone())
    }

    pub fn get_available_canister(caller: Principal) -> Result<ScalableCanisterDetails, String> {
        let canister = DATA.with(|v| {
            v.borrow()
                .canisters
                .iter()
                .filter(|(_, c)| c.principal != caller)
                .find(|(_, c)| c.is_available)
                .map(|(_, details)| details.clone())
        });

        match canister {
            None => Err("No available canister found".to_string()),
            Some(c) => Ok(c),
        }
    }

    pub fn get_canisters() -> Vec<ScalableCanisterDetails> {
        let canisters: Vec<ScalableCanisterDetails> = DATA.with(|v| {
            v.borrow()
                .canisters
                .iter()
                .map(|(_, details)| details.clone())
                .collect()
        });
        return canisters;
    }

    // pub fn get_wasm(caller: Principal) -> Result<WasmDetails, ApiError> {
    //     let inputs = Some(vec![format!("caller - {}", &caller.to_string())]);

    //     if !Self::has_whitelist_rights(caller, WhitelistRights::Read) {
    //         return Err(Self::whitelist_error("get_whitelist".to_string(), inputs));
    //     };

    //     let result = DATA.with(|v| v.borrow().child_wasm_data.clone());

    //     Ok(result)
    // }

    // pub fn add_wasm(caller: Principal, label: String, bytes: Vec<u8>) -> Result<bool, ApiError> {
    //     let inputs = Some(vec![
    //         format!("caller - {}", &caller.to_string()),
    //         format!("bytes - {}", &bytes.len()),
    //     ]);

    //     if !Self::has_whitelist_rights(caller, WhitelistRights::ReadWrite) {
    //         return Err(Self::whitelist_error("get_whitelist".to_string(), inputs));
    //     };

    //     let previous_wasm = DATA.with(|v| v.borrow().child_wasm_data.clone());
    //     let new_version = match previous_wasm.wasm_version {
    //         WasmVersion::Version(number) => number + 1,
    //         _ => 1,
    //     };

    //     let new_wasm = WasmDetails {
    //         label,
    //         bytes,
    //         wasm_type: CanisterType::ScalableChild,
    //         wasm_version: WasmVersion::Version(new_version),
    //         updated_at: time(),
    //         created_at: previous_wasm.created_at,
    //     };

    //     DATA.with(|v| v.borrow_mut().child_wasm_data = new_wasm);
    //     Ok(true)
    // }

    pub async fn initialize_first_child_canister(caller: Principal) -> Result<Principal, ApiError> {
        let inputs = Some(vec![format!("caller - {}", &caller.to_string())]);

        if CHILD_WASM.len() == 0 {
            return Err(api_error(
                ApiErrorType::BadRequest,
                "NO_WASM_SPECIFIED",
                "There is no child WASM uploaded",
                &Self::get_name(),
                "initialize_first_child_canister",
                inputs,
            ));
        }

        if !DATA.with(|v| {
            v.borrow()
                .canisters
                .iter()
                .any(|(principal, _)| principal == &caller)
                || Self::has_whitelist_rights(caller, WhitelistRights::ReadWrite)
        }) {
            return Err(api_error(
                ApiErrorType::BadRequest,
                "UNKNOWN_CANISTER",
                "The caller principal isnt known to this canister",
                &Self::get_name(),
                "initialize_first_child_canister",
                inputs,
            ));
        }

        if DATA.with(|v| v.borrow().canisters.iter().len() != 0) {
            return Err(api_error(
                ApiErrorType::BadRequest,
                "ALREADY_INITIALIZED",
                "This scalable canister has already initialized a child canister",
                &Self::get_name(),
                "initialize_first_child_canister",
                inputs,
            ));
        }

        let new_canister = Self::spawn_empty_canister(caller).await;
        match new_canister {
            Err(err) => Err(err),
            Ok(new_canister_principal) => {
                let installed_canister = Self::_install_child_canister(
                    caller,
                    Self::get_name(),
                    caller,
                    new_canister_principal,
                    InstallCodeMode::Install,
                )
                .await;
                match installed_canister {
                    Err(err) => Err(err),
                    Ok(new_installed_canister_principal) => Ok(new_installed_canister_principal),
                }
            }
        }
    }

    pub async fn close_child_canister_and_spawn_sibling(
        caller: Principal,
        owner: Principal,
        last_entry_id: u64,
        entry: Vec<u8>,
        principal_entry_reference: Option<Principal>,
    ) -> Result<Principal, ApiError> {
        let inputs = Some(vec![
            format!("caller - {}", &caller.to_string()),
            format!("owner - {}", &owner.to_string()),
            format!("last_entry_id - {:?}", &last_entry_id),
            format!(
                "principal_entry_reference - {:?}",
                &principal_entry_reference
            ),
        ]);

        if DATA.with(|v| v.borrow().child_wasm_data.bytes.len() == 0) {
            return Err(api_error(
                ApiErrorType::BadRequest,
                "NO_WASM_SPECIFIED",
                "There is no foundation WASM uploaded",
                &Self::get_name(),
                "close_child_canister_and_spawn_sibling",
                inputs,
            ));
        }

        if !DATA.with(|v| {
            v.borrow()
                .canisters
                .iter()
                .any(|(principal, _)| principal == &caller)
        }) {
            return Err(api_error(
                ApiErrorType::BadRequest,
                "UNKNOWN_CANISTER",
                "The caller principal isnt known to this canister",
                &Self::get_name(),
                "close_child_canister_and_spawn_sibling",
                inputs,
            ));
        }

        let caller_canister = DATA.with(|v| v.borrow().canisters.get(&caller).cloned());
        match caller_canister {
            None => Err(api_error(
                ApiErrorType::BadRequest,
                "UNKNOWN_CANISTER",
                "The caller principal isnt known to this canister",
                &Self::get_name(),
                "close_child_canister_and_spawn_sibling",
                inputs,
            )),
            Some(_caller_canister) => {
                let new_canister = Self::spawn_empty_canister(caller).await;
                match new_canister {
                    Err(err) => Err(err),
                    Ok(new_canister_principal) => {
                        let installed_canister = Self::_install_child_canister(
                            caller,
                            Self::get_name(),
                            owner,
                            new_canister_principal,
                            InstallCodeMode::Install,
                        )
                        .await;
                        match installed_canister {
                            Err(err) => Err(err),
                            Ok(new_installed_canister_principal) => {
                                let updated_canister = ScalableCanisterDetails {
                                    principal: _caller_canister.principal.clone(),
                                    canister_type: _caller_canister.canister_type.clone(),
                                    wasm_version: _caller_canister.wasm_version.clone(),
                                    is_available: false,
                                    entry_range: (0, Some(last_entry_id)),
                                };

                                DATA.with(|v| {
                                    v.borrow_mut()
                                        .canisters
                                        .insert(_caller_canister.principal, updated_canister)
                                });

                                let call_result: Result<(Result<(), ApiError>,), _> = call::call(
                                    new_installed_canister_principal,
                                    "add_entry_by_parent",
                                    (principal_entry_reference, entry),
                                )
                                .await;

                                match call_result {
                                    Err(err) => Err(api_error(
                                        ApiErrorType::BadRequest,
                                        "FAILED_TO_STORE_DATA",
                                        err.1.as_str(),
                                        &Self::get_name(),
                                        "close_child_canister_and_spawn_sibling",
                                        inputs,
                                    )),
                                    Ok(_) => Ok(new_installed_canister_principal),
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub async fn upgrade_child_canister(
        caller: Principal,
        canister_principal: Principal,
    ) -> Result<ScalableCanisterDetails, ApiError> {
        let inputs = Some(vec![
            format!("caller - {}", &caller.to_string()),
            format!("canister_principal - {}", &canister_principal.to_string()),
        ]);

        if !Self::has_whitelist_rights(caller, WhitelistRights::ReadWrite) {
            return Err(Self::whitelist_error(
                "upgrade_scalable_canister".to_string(),
                inputs,
            ));
        };
        let data = DATA.with(|v| v.borrow().clone());
        let existing_child = data.canisters.get(&canister_principal);
        match existing_child {
            None => Err(api_error(
                ApiErrorType::NotFound,
                "NO_CHILDREN",
                "There are no child canisters found",
                &Self::get_name(),
                "upgrade_scalable_canister",
                inputs,
            )),
            Some(_child) => {
                if data.child_wasm_data.wasm_version == _child.wasm_version {
                    return Err(api_error(
                        ApiErrorType::BadRequest,
                        "CANISTER_UP_TO_DATE",
                        "The latest WASM version is already installed",
                        &Self::get_name(),
                        "upgrade_scalable_canister",
                        inputs,
                    ));
                }

                let canister = Canister::from(_child.principal);
                let upgrade_result = canister
                    .install_code(
                        InstallCodeMode::Upgrade,
                        data.child_wasm_data.bytes.clone(),
                        (),
                    )
                    .await;
                match upgrade_result {
                    Err(err) => Err(api_error(
                        ApiErrorType::BadRequest,
                        "UPGRADE_FAILED",
                        &err.1.as_str(),
                        &Self::get_name(),
                        "upgrade_scalable_canister",
                        inputs,
                    )),
                    Ok(_) => {
                        let updated_child = ScalableCanisterDetails {
                            principal: _child.principal,
                            canister_type: _child.canister_type.clone(),
                            wasm_version: data.child_wasm_data.wasm_version.clone(),
                            is_available: _child.is_available,
                            entry_range: _child.entry_range,
                        };

                        DATA.with(|v| {
                            v.borrow_mut()
                                .canisters
                                .insert(canister_principal, updated_child.clone())
                        });
                        Ok(updated_child)
                    }
                }
            }
        }
    }

    pub async fn reinstall_child_canister(
        caller: Principal,
        canister_principal: Principal,
    ) -> Result<Principal, ApiError> {
        if !Self::has_whitelist_rights(caller, WhitelistRights::ReadWrite) {
            return Err(Self::whitelist_error(
                "upgrade_scalable_canister".to_string(),
                None,
            ));
        };

        Self::_install_child_canister(
            caller,
            Self::get_name(),
            caller,
            canister_principal,
            InstallCodeMode::Reinstall,
        )
        .await
    }

    async fn spawn_empty_canister(caller: Principal) -> Result<Principal, ApiError> {
        let inputs = Some(vec![format!("caller - {}", &caller.to_string())]);

        let canister_settings = CanisterSettings {
            controllers: Some(vec![caller, id()]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        };

        let new_canister = Canister::create(Some(canister_settings), 2_000_000_000_000).await;
        match new_canister {
            Err(err) => Err(api_error(
                ApiErrorType::BadRequest,
                "CANISTER_NOT_CREATED",
                err.1.as_str(),
                &Self::get_name(),
                "_spawn_empty_canister",
                inputs,
            )),
            Ok(_canister) => {
                let new_canister_principal = CanisterID::from(_canister);
                let canister_data = ScalableCanisterDetails {
                    principal: new_canister_principal,
                    wasm_version: WasmVersion::None,
                    canister_type: CanisterType::Empty,
                    is_available: true,
                    entry_range: (0, None),
                };

                DATA.with(|v| {
                    v.borrow_mut()
                        .canisters
                        .insert(new_canister_principal, canister_data)
                });
                Ok(new_canister_principal)
            }
        }
    }

    async fn _install_child_canister(
        caller: Principal,
        name: String,
        owner: Principal,
        canister_principal: Principal,
        install_code_mode: InstallCodeMode,
    ) -> Result<Principal, ApiError> {
        let inputs = Some(vec![
            format!("caller - {}", &caller.to_string()),
            format!("name - {}", &name.to_string()),
        ]);

        let data = DATA.with(|v| v.borrow().clone());
        if CHILD_WASM.len() == 0 {
            return Err(api_error(
                ApiErrorType::BadRequest,
                "NO_WASM_SPECIFIED",
                "There is no foundation WASM uploaded",
                &Self::get_name(),
                "install_child_canister",
                inputs,
            ));
        }

        let install_canister = Canister::from(canister_principal)
            .install_code(
                install_code_mode,
                CHILD_WASM.to_vec(),
                (owner, id(), name, data.canisters.iter().len()),
            )
            .await;

        match install_canister {
            Err(err) => Err(api_error(
                ApiErrorType::NotFound,
                "CANISTER_INSTALL_FAILED",
                err.1.as_str(),
                &Self::get_name(),
                "_install_child_canister",
                inputs,
            )),
            Ok(_) => {
                let new_child_details = ScalableCanisterDetails {
                    principal: canister_principal,
                    wasm_version: data.child_wasm_data.wasm_version.clone(),
                    is_available: true,
                    canister_type: CanisterType::ScalableChild,
                    entry_range: (0, None),
                };

                DATA.with(|v| {
                    v.borrow_mut()
                        .canisters
                        .insert(canister_principal, new_child_details)
                });
                Ok(canister_principal)
            }
        }
    }

    fn has_whitelist_rights(principal: Principal, rights: WhitelistRights) -> bool {
        let entry = DATA.with(|v| {
            v.borrow()
                .whitelist
                .iter()
                .find(|v| v.principal == principal)
                .cloned()
        });

        match entry {
            None => false,
            Some(_entry) => {
                if _entry.rights <= rights {
                    true
                } else {
                    false
                }
            }
        }
    }

    fn whitelist_error(method_name: String, inputs: Option<Vec<String>>) -> ApiError {
        ApiError::Unauthorized(ErrorMessage {
            tag: "NOT_WHITELISTED".to_string(),
            message: "You are not allowed to perform this action".to_string(),
            location: format!("{}/{}/{}", id(), &Self::get_name(), method_name),
            inputs,
        })
    }

    pub async fn get_child_canister_data(
        // limit: usize,
        // page: usize,
        filters: Vec<GroupFilter>,
        filter_type: FilterType,
        // sort: GroupSort,
    ) -> Vec<GroupResponse> {
        let canisters: Vec<Principal> = DATA.with(|data| {
            data.borrow()
                .canisters
                .clone()
                .into_iter()
                .map(|c| c.1.principal.clone())
                .collect()
        });

        let mut groups: Vec<GroupResponse> = vec![];
        for canister in canisters {
            let mut canister_data =
                Self::get_filtered_child_data(canister, &filters, &filter_type).await;
            groups.append(&mut canister_data);
        }

        groups
    }

    async fn get_filtered_child_data(
        canister_principal: Principal,
        filters: &Vec<GroupFilter>,
        filter_type: &FilterType,
    ) -> Vec<GroupResponse> {
        let (mut bytes, (_, last)) =
            Self::get_child_data_call(canister_principal, filters, filter_type, 0, None).await;

        if last > 1 {
            for i in 1..last + 1 {
                let (mut _bytes, _) =
                    Self::get_child_data_call(canister_principal, filters, filter_type, i, None)
                        .await;
                bytes.append(&mut _bytes);
            }
        }

        match deserialize::<Vec<GroupResponse>>(bytes.clone()) {
            Ok(_res) => _res,
            Err(_err) => {
                ic_cdk::println!("Error: {}", _err);
                vec![]
            }
        }
    }

    pub async fn get_child_data_call(
        canister_principal: Principal,
        filters: &Vec<GroupFilter>,
        filter_type: &FilterType,
        chunk: usize,
        max_bytes_per_chunk: Option<usize>,
    ) -> (Vec<u8>, (usize, usize)) {
        let _max_bytes_per_chunk = max_bytes_per_chunk.unwrap_or(2_000_000);
        let result: Result<(Vec<u8>, (usize, usize)), _> = call::call(
            canister_principal,
            "get_groups_for_parent",
            (filters, filter_type, chunk, _max_bytes_per_chunk),
        )
        .await;

        match result {
            Ok(_res) => _res,
            _ => (vec![], (0, 0)),
        }
    }
}
