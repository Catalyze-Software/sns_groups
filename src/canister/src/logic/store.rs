use std::{cell::RefCell, collections::HashMap};

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
        sort_type::SortDirection,
        wasm_version_type::WasmVersion,
    },
    helpers::{
        canister_helper::{Canister, CanisterID, CanisterSettings, InstallCodeMode},
        error_helper::api_error,
        paging_helper::get_paged_data,
        serialize_helper::deserialize,
    },
    models::{
        canister_models::ScalableCanisterDetails, paged_response_models::PagedResponse,
        wasm_models::WasmDetails, whitelist_models::WhitelistEntry,
    },
};

use crate::models::group_model::{GroupFilter, GroupResponse, GroupSort};

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
            child_wasm_data: Default::default(),
            whitelist: Vec::default(),
            owner: Principal::anonymous(),
            parent: Principal::anonymous(),
            updated_at: time(),
            created_at: time(),
        }
    }
}

thread_local! {
    pub static DATA: RefCell<ScalableData> = RefCell::new(ScalableData::default());
}
impl ScalableData {
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

    pub async fn initialize_first_child_canister() -> () {
        if DATA.with(|data| data.borrow().child_wasm_data.bytes.len()) == 0 {
            return;
        }

        if DATA.with(|v| v.borrow().canisters.len() != 0) {
            return;
        }

        let new_canister = Self::spawn_empty_canister(id()).await;
        let _ = match new_canister {
            Err(err) => Err(err),
            Ok(new_canister_principal) => {
                let installed_canister = Self::_install_child_canister(
                    id(),
                    Self::get_name(),
                    id(),
                    new_canister_principal,
                    InstallCodeMode::Install,
                )
                .await;
                match installed_canister {
                    Err(err) => Err(err),
                    Ok(new_installed_canister_principal) => Ok(new_installed_canister_principal),
                }
            }
        };
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
        canister_principal: Principal,
    ) -> Result<ScalableCanisterDetails, ApiError> {
        let inputs = Some(vec![format!(
            "canister_principal - {}",
            &canister_principal.to_string()
        )]);

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
        if data.child_wasm_data.bytes.len() == 0 {
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
                data.child_wasm_data.bytes,
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

    pub async fn upgrade_children() {
        let data = DATA.with(|data| data.borrow().clone());
        for child in data.canisters {
            if child.1.wasm_version != data.child_wasm_data.wasm_version {
                match ScalableData::upgrade_child_canister(child.0.clone()).await {
                    Ok(_) => ic_cdk::println!("Installed"),
                    Err(err) => ic_cdk::println!("Install error: {:?}", err),
                };
            }
        }
    }

    pub fn get_child_wasm_data(
        old_store: &ScalableData,
        version: u64,
    ) -> Result<WasmDetails, String> {
        let bytes = include_bytes!("../../../../wasm/demo_child_canister.wasm").to_vec();

        if bytes.is_empty() {
            return Err("No WASM found, skipping child WASM update".to_string());
        }

        if old_store.child_wasm_data.bytes == bytes {
            return Err("WASM is the same, skipping child WASM update".to_string());
        }

        if old_store.child_wasm_data.bytes != bytes {
            match old_store.child_wasm_data.wasm_version {
                WasmVersion::None => {
                    // return Err("Wrong WASM version type, skipping child WASM update".to_string())
                }
                WasmVersion::Version(_version) => {
                    if version <= _version {
                        return Err(format!(
                            "Please provide a higher version as {_version}, skipping child WASM update"
                        )
                        .to_string());
                    }
                }
                WasmVersion::Custom => {
                    return Err("Wrong WASM version type, skipping child WASM update".to_string())
                }
            }
        }

        let details = WasmDetails {
            label: "child_group_canister".to_string(),
            bytes,
            wasm_type: CanisterType::ScalableChild,
            wasm_version: WasmVersion::Version(version),
            updated_at: time(),
            created_at: old_store.child_wasm_data.created_at,
        };

        Ok(details)
    }

    pub async fn get_child_canister_data(
        limit: usize,
        page: usize,
        filters: Vec<GroupFilter>,
        filter_type: FilterType,
        sort: GroupSort,
    ) -> PagedResponse<GroupResponse> {
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

        let ordered_groups = Self::get_ordered_groups(groups, sort);
        get_paged_data(ordered_groups, limit, page)
    }

    async fn get_filtered_child_data(
        canister_principal: Principal,
        filters: &Vec<GroupFilter>,
        filter_type: &FilterType,
    ) -> Vec<GroupResponse> {
        let (mut bytes, (_, last)) =
            Self::get_chunked_child_data(canister_principal, filters, filter_type, 0, None).await;

        if last > 1 {
            for i in 1..last + 1 {
                let (mut _bytes, _) =
                    Self::get_chunked_child_data(canister_principal, filters, filter_type, i, None)
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

    fn get_ordered_groups(mut groups: Vec<GroupResponse>, sort: GroupSort) -> Vec<GroupResponse> {
        match sort {
            GroupSort::CreatedOn(direction) => match direction {
                SortDirection::Asc => groups.sort_by(|a, b| a.created_on.cmp(&b.created_on)),
                SortDirection::Desc => groups.sort_by(|a, b| b.created_on.cmp(&a.created_on)),
            },
            GroupSort::UpdatedOn(direction) => match direction {
                SortDirection::Asc => groups.sort_by(|a, b| a.updated_on.cmp(&b.updated_on)),
                SortDirection::Desc => groups.sort_by(|a, b| b.updated_on.cmp(&a.updated_on)),
            },
            GroupSort::Name(direction) => match direction {
                SortDirection::Asc => groups.sort_by(|a, b| a.name.cmp(&b.name)),
                SortDirection::Desc => groups.sort_by(|a, b| b.name.cmp(&a.name)),
            },
            GroupSort::MemberCount(direction) => match direction {
                SortDirection::Asc => groups.sort_by(|a, b| a.member_count.cmp(&b.member_count)),
                SortDirection::Desc => groups.sort_by(|a, b| b.member_count.cmp(&a.member_count)),
            },
        };
        groups
    }

    async fn get_chunked_child_data(
        canister_principal: Principal,
        filters: &Vec<GroupFilter>,
        filter_type: &FilterType,
        chunk: usize,
        max_bytes_per_chunk: Option<usize>,
    ) -> (Vec<u8>, (usize, usize)) {
        let _max_bytes_per_chunk = max_bytes_per_chunk.unwrap_or(2_000_000);
        let result: Result<(Vec<u8>, (usize, usize)), _> = call::call(
            canister_principal,
            "get_chunked_data",
            (filters, filter_type, chunk, _max_bytes_per_chunk),
        )
        .await;

        match result {
            Ok(_res) => _res,
            _ => (vec![], (0, 0)),
        }
    }

    fn get_name() -> String {
        DATA.with(|v| v.borrow().name.clone())
    }
}
