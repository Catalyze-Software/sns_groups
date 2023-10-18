use std::{collections::HashMap, iter::FromIterator, vec};

use candid::Principal;
use ic_cdk::api::{call, time};
use ic_scalable_canister::store::Data;
use ic_scalable_misc::{
    enums::{
        api_error_type::{ApiError, ApiErrorType},
        filter_type::FilterType,
        privacy_type::{GatedType, NeuronGatedRules, Privacy, TokenGated},
        sort_type::SortDirection,
    },
    helpers::{
        error_helper::api_error,
        paging_helper::get_paged_data,
        role_helper::{default_roles, get_member_roles, get_read_only_permissions, has_permission},
        serialize_helper::serialize,
        token_canister_helper::{
            dip20_balance_of, dip721_balance_of, ext_balance_of, legacy_dip721_balance_of,
        },
    },
    models::{
        group_role::GroupRole,
        identifier_model::Identifier,
        neuron_models::{DissolveState, ListNeurons, ListNeuronsResponse},
        paged_response_models::PagedResponse,
        permissions_models::{Permission, PermissionActionType, PermissionType, PostPermission},
    },
};

use shared::group_model::{Group, GroupFilter, GroupResponse, GroupSort, PostGroup, UpdateGroup};
use std::cell::RefCell;

use crate::{validation::validate_post_group, IDENTIFIER_KIND};

use super::validation::validate_update_group;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    {DefaultMemoryImpl, StableBTreeMap, StableCell},
};

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // NEW STABLE
    pub static STABLE_DATA: RefCell<StableCell<Data, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            Data::default(),
        ).expect("failed")
    );

    pub static ENTRIES: RefCell<StableBTreeMap<String, Group, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
        )
    );

    pub static DATA: RefCell<ic_scalable_misc::models::original_data::Data<Group>> = RefCell::new(ic_scalable_misc::models::original_data::Data::default());
}

pub struct Store;

impl Store {
    // Method to add a group to the data store
    pub async fn add_group(
        caller: Principal,
        post_group: PostGroup,
        member_canister: Principal,
        account_identifier: Option<String>,
    ) -> Result<GroupResponse, ApiError> {
        let temp_group = post_group.clone();
        // Map "post_group" to "group" struct
        let new_group = Group {
            name: temp_group.name,
            description: temp_group.description,
            website: temp_group.website,
            location: temp_group.location,
            privacy: temp_group.privacy,
            owner: caller,
            created_by: caller,
            matrix_space_id: temp_group.matrix_space_id,
            image: temp_group.image,
            banner_image: temp_group.banner_image,
            tags: temp_group.tags,
            // The member count is a hashmap with the canister id as key and the count as value
            member_count: HashMap::from_iter(vec![(member_canister, 1)].into_iter()),
            roles: vec![],
            is_deleted: false,
            updated_on: time(),
            created_on: time(),
            wallets: HashMap::new(),
        };

        let add_entry_result = match Self::validate_group_privacy(
            caller,
            account_identifier,
            post_group.privacy.clone(),
        )
        .await
        {
            Err(err) => Err(err),
            Ok(_) => {
                STABLE_DATA.with(|data| match validate_post_group(post_group) {
                    // Return an error if the group data is invalid
                    Err(err) => Err(err),

                    // Add the group to the data store and pass in the "kind" as a third parameter to generate a identifier
                    Ok(_) => ENTRIES.with(|entries| {
                        match Data::add_entry(
                            data,
                            entries,
                            new_group.clone(),
                            Some(IDENTIFIER_KIND.to_string()),
                        ) {
                            Err(err) => Err(err),
                            Ok(result) => Ok(result),
                        }
                    }),
                })
            }
        };

        // Check if the group was added to the data store successfully
        let _data = STABLE_DATA.with(|v| v.borrow().get().clone());
        match add_entry_result {
            // The group was not added to the data store because the canister is at capacity
            Err(err) => match err {
                ApiError::CanisterAtCapacity(message) => {
                    // Spawn a sibling canister and pass the group data to it
                    match Data::spawn_sibling(&_data, new_group.clone()).await {
                        Ok(_) => Err(ApiError::CanisterAtCapacity(message)),
                        Err(err) => Err(err),
                    }
                }
                _ => Err(err),
            },
            Ok((_identifier, _group_data)) => {
                // Add the owner of the group to the member canister as an entry
                match Self::add_owner(&caller, &_identifier, &member_canister).await {
                    Err(err) => {
                        // If the owner was not added to the member canister successfully, remove the group from the data store
                        ENTRIES.with(|data| Data::remove_entry(data, &_identifier));
                        Err(err)
                    }
                    Ok((_identifier, _group_data)) => {
                        // If successfull return the group data
                        Ok(Self::map_group_to_group_response(
                            _identifier.to_string(),
                            _group_data,
                        ))
                    }
                }
            }
        }
    }

    // Method to update a group in the data store
    pub fn update_group(
        caller: Principal,
        group_identifier: Principal,
        update_group: UpdateGroup,
    ) -> Result<GroupResponse, ApiError> {
        let inputs = Some(vec![
            format!("caller - {:?}", &caller),
            format!("id - {:?}", &group_identifier),
            format!("update_group - {:?}", &update_group),
        ]);

        // Validate the "update_group" data
        STABLE_DATA.with(|data| match validate_update_group(update_group.clone()) {
            Err(err) => Err(err),
            Ok(_) => {
                // Check if the group exists in the data store
                match ENTRIES.with(|entries| Data::get_entry(data, entries, group_identifier)) {
                    // Return an error if the group does not exist
                    Err(err) => Err(err),
                    Ok((_identifier, mut _group_data)) => {
                        // If the group is deleted return an error
                        if _group_data.is_deleted {
                            return Err(api_error(
                                ApiErrorType::BadRequest,
                                "DELETED_GROUP",
                                "You cant update a deleted group",
                                Data::get_name(data.borrow().get()).as_str(),
                                "update_group",
                                inputs,
                            ));
                        }
                        // Update group fields
                        _group_data.name = update_group.name;
                        _group_data.description = update_group.description;
                        _group_data.website = update_group.website;
                        _group_data.location = update_group.location;
                        _group_data.privacy = update_group.privacy;
                        _group_data.image = update_group.image;
                        _group_data.banner_image = update_group.banner_image;
                        _group_data.tags = update_group.tags;
                        _group_data.updated_on = time();

                        let update_group_result = ENTRIES.with(|entries| {
                            Data::update_entry(data, entries, group_identifier, _group_data)
                        });
                        match update_group_result {
                            Err(err) => Err(err),
                            Ok((_identifier, _group_data)) => {
                                Ok(Self::map_group_to_group_response(
                                    _identifier.to_string(),
                                    _group_data,
                                ))
                            }
                        }
                    }
                }
            }
        })
    }

    // Method to delete a group from the data store
    pub fn delete_group(caller: Principal, identifier: Principal) -> Result<Group, ApiError> {
        let inputs = Some(vec![
            format!("caller - {:?}", &caller),
            format!("id - {:?}", &identifier),
        ]);
        STABLE_DATA.with(|data| {
            // Check if the group exists in the data store
            match ENTRIES.with(|entries| Data::get_entry(data, entries, identifier)) {
                Err(err) => Err(err),
                Ok((_identifier, mut _group_data)) => {
                    // Check of the group owner is also the caller
                    if _group_data.owner != caller {
                        return Err(api_error(
                            ApiErrorType::Unauthorized,
                            "CANT_DELETE_GROUP",
                            "Only the owner can delete the group",
                            Data::get_name(data.borrow().get()).as_str(),
                            "delete_group",
                            inputs,
                        ));
                    }

                    _group_data.is_deleted = true;
                    _group_data.updated_on = time();

                    let update_group_result = ENTRIES.with(|entries| {
                        Data::update_entry(data, entries, _identifier, _group_data.clone())
                    });

                    match update_group_result {
                        Err(err) => Err(err),
                        Ok(_) => Ok(_group_data),
                    }
                }
            }
        })
    }

    // Method to get a group with an identifier from the data store
    pub fn get_group(identifier: Principal) -> Result<GroupResponse, ApiError> {
        STABLE_DATA.with(|data| {
            match ENTRIES.with(|entries| Data::get_entry(data, entries, identifier)) {
                Err(err) => Err(err),
                Ok((_identifier, _group_data)) => Ok(Self::map_group_to_group_response(
                    _identifier.to_string(),
                    _group_data,
                )),
            }
        })
    }

    pub fn add_wallet(
        caller: Principal,
        group_identifier: Principal,
        wallet_canister: Principal,
        description: String,
    ) -> Result<(), ApiError> {
        let inputs = Some(vec![
            format!("caller - {:?}", &caller),
            format!("group_identifier - {:?}", &group_identifier),
            format!("wallet_canister - {:?}", &wallet_canister),
            format!("description - {:?}", &description),
        ]);
        STABLE_DATA.with(|data| {
            match ENTRIES.with(|entries| Data::get_entry(data, entries, group_identifier)) {
                Err(err) => Err(err),
                Ok((_identifier, mut _group_data)) => {
                    if _group_data.owner != caller {
                        return Err(api_error(
                            ApiErrorType::Unauthorized,
                            "CANT_ADD_WALLET",
                            "Only the owner can add a wallet",
                            Data::get_name(data.borrow().get()).as_str(),
                            "add_wallet",
                            inputs,
                        ));
                    }

                    _group_data.wallets.insert(wallet_canister, description);
                    _group_data.updated_on = time();

                    let update_group_result = ENTRIES.with(|entries| {
                        Data::update_entry(data, entries, _identifier, _group_data.clone())
                    });

                    match update_group_result {
                        Err(err) => Err(err),
                        Ok(_) => Ok(()),
                    }
                }
            }
        })
    }

    pub fn remove_wallet(
        caller: Principal,
        group_identifier: Principal,
        wallet_canister: Principal,
    ) -> Result<(), ApiError> {
        let inputs = Some(vec![
            format!("caller - {:?}", &caller),
            format!("group_identifier - {:?}", &group_identifier),
            format!("wallet_canister - {:?}", &wallet_canister),
        ]);
        STABLE_DATA.with(|data| {
            // Check if the group exists in the data store
            match ENTRIES.with(|entries| Data::get_entry(data, entries, group_identifier)) {
                Err(err) => Err(err),
                Ok((_identifier, mut _group_data)) => {
                    if _group_data.owner != caller {
                        return Err(api_error(
                            ApiErrorType::Unauthorized,
                            "CANT_DELETE_WALLET",
                            "Only the owner can delete a wallet",
                            Data::get_name(data.borrow().get()).as_str(),
                            "remove_wallet",
                            inputs,
                        ));
                    }

                    _group_data.wallets.remove(&wallet_canister);
                    _group_data.updated_on = time();

                    let update_group_result = ENTRIES.with(|entries| {
                        Data::update_entry(data, entries, _identifier, _group_data.clone())
                    });

                    match update_group_result {
                        Err(err) => Err(err),
                        Ok(_) => Ok(()),
                    }
                }
            }
        })
    }

    // Method to get a group with an identifier from the data store
    pub fn get_group_owner_and_privacy(
        identifier: Principal,
    ) -> Result<(Principal, Privacy), ApiError> {
        STABLE_DATA.with(|data| {
            match ENTRIES.with(|entries| Data::get_entry(data, entries, identifier)) {
                Err(err) => Err(err),
                Ok((_, _group_data)) => Ok((_group_data.owner, _group_data.privacy)),
            }
        })
    }

    // This method is used to get groups filtered and sorted with pagination
    pub fn get_groups(
        limit: usize,
        page: usize,
        filters: Vec<GroupFilter>,
        filter_type: FilterType,
        sort: GroupSort,
        include_invite_only: bool,
    ) -> PagedResponse<GroupResponse> {
        let groups = ENTRIES.with(|data| Data::get_entries(data));
        // Get groups for filtering and sorting
        let mapped_groups: Vec<GroupResponse> = groups
            .iter()
            // Filter out deleted groups
            .filter(|(_identifier, _group_data)| !_group_data.is_deleted)
            .filter(|(_identifier, _group_data)| {
                if include_invite_only {
                    true
                } else {
                    _group_data.privacy != Privacy::InviteOnly
                }
            })
            // Map groups to group response
            .map(|(_identifier, _group_data)| {
                Self::map_group_to_group_response(_identifier.clone(), _group_data.clone())
            })
            .collect();

        // Filter groups
        let filtered_groups = Self::get_filtered_groups(mapped_groups, filters, filter_type);
        // Order groups
        let ordered_groups = Self::get_ordered_groups(filtered_groups, sort);

        // Paginate groups and return
        get_paged_data(ordered_groups, limit, page)
    }

    // Used for composite_query calls from the parent canister
    //
    // Method to get filtered groups serialized and chunked
    pub fn get_chunked_data(
        filters: Vec<GroupFilter>,
        filter_type: FilterType,
        chunk: usize,
        max_bytes_per_chunk: usize,
    ) -> (Vec<u8>, (usize, usize)) {
        let groups = ENTRIES.with(|entries| Data::get_entries(entries));
        // Get groups for filtering
        let mapped_groups: Vec<GroupResponse> = groups
            .iter()
            // Filter out deleted groups
            .filter(|(_identifier, _group_data)| !_group_data.is_deleted)
            // Map groups to group response
            .map(|(_identifier, _group_data)| {
                Self::map_group_to_group_response(_identifier.clone(), _group_data.clone())
            })
            .collect();

        let filtered_groups = Self::get_filtered_groups(mapped_groups, filters, filter_type);
        if let Ok(bytes) = serialize(&filtered_groups) {
            // Check if the bytes of the serialized groups are greater than the max bytes per chunk specified as an argument
            if bytes.len() >= max_bytes_per_chunk {
                // Get the start and end index of the bytes to be returned
                let start = chunk * max_bytes_per_chunk;
                let end = (chunk + 1) * (max_bytes_per_chunk);

                // Get the bytes to be returned, if the end index is greater than the length of the bytes, return the remaining bytes
                let response = if end >= bytes.len() {
                    bytes[start..].to_vec()
                } else {
                    bytes[start..end].to_vec()
                };

                // Determine the max number of chunks that can be returned, a float is used because the number of chunks can be a decimal in this step
                let mut max_chunks: f64 = 0.00;
                if max_bytes_per_chunk < bytes.len() {
                    max_chunks = (bytes.len() / max_bytes_per_chunk) as f64;
                }

                // return the response and start and end chunk index, the end chunk index is calculated by rounding up the max chunks
                return (response, (chunk, max_chunks.ceil() as usize));
            }

            // if the bytes of the serialized groups are less than the max bytes per chunk specified as an argument, return the bytes and start and end chunk index as 0
            return (bytes, (0, 0));
        } else {
            // if the groups cant be serialized return an empty vec and start and end chunk index as 0
            return (vec![], (0, 0));
        }
    }

    // Method to get multiple groups with an identifier from the data store
    pub fn get_groups_by_id(group_ids: Vec<Principal>) -> Vec<GroupResponse> {
        STABLE_DATA.with(|data| {
            let mut groups: Vec<GroupResponse> = vec![];

            // Loop over the group ids and get the group data
            group_ids.into_iter().for_each(|_identifier| {
                let existing = ENTRIES.with(|entries| Data::get_entry(data, entries, _identifier));

                // If the group data exists, map it to a group response and push it to the groups vec
                match existing {
                    Err(_) => {}
                    Ok((_identifier, _group_data)) => {
                        if !_group_data.is_deleted {
                            groups.push(Self::map_group_to_group_response(
                                _identifier.to_string(),
                                _group_data,
                            ))
                        }
                    }
                };
            });
            groups
        })
    }

    // Method to add a custom role to the group
    pub fn add_role(
        caller: Principal,
        group_identifier: Principal,
        role_name: String,
        color: String,
        index: u64,
    ) -> Result<GroupRole, ApiError> {
        let inputs = Some(vec![
            format!("caller - {:?}", &caller.to_string()),
            format!("group_id - {:?}", &group_identifier),
            format!("role_name - {:?}", &role_name),
        ]);

        // get the group data
        STABLE_DATA.with(|data| {
            match ENTRIES.with(|entries| Data::get_entry(data, entries, group_identifier)) {
                Err(err) => Err(err),
                Ok((_identifier, mut _group_data)) => {
                    // check if the caller is the owner of the group
                    if _group_data.owner != caller {
                        return Err(api_error(
                            ApiErrorType::Unauthorized,
                            "UNAUTHORIZED",
                            "Only owner of a group can add roles",
                            Data::get_name(data.borrow().get()).as_str(),
                            "add_role",
                            inputs,
                        ));
                    }

                    let mut roles = _group_data.roles;
                    let included_role = roles.iter().any(|r| r.name == role_name);

                    // check if the role name already exists in the custom or default roles
                    if included_role || default_roles().iter().any(|r| r.name == role_name) {
                        return Err(api_error(
                            ApiErrorType::BadRequest,
                            "EXISTING_ROLE",
                            "This role is already registered",
                            Data::get_name(data.borrow().get()).as_str(),
                            "add_role",
                            inputs,
                        ));
                    };

                    let new_role = GroupRole {
                        name: role_name,
                        protected: false,
                        // set default permissions to read-only
                        permissions: get_read_only_permissions(),
                        color,
                        // optional sorting index
                        index: Some(index),
                    };

                    roles.push(new_role.clone());

                    _group_data.roles = roles;
                    _group_data.updated_on = time();

                    let update_group_result = ENTRIES.with(|entries| {
                        Data::update_entry(data, entries, group_identifier, _group_data.clone())
                    });

                    match update_group_result {
                        Err(err) => Err(err),
                        Ok(_) => Ok(new_role),
                    }
                }
            }
        })
    }

    // Method to do an inter-canister call to the member canister to add an owner as a member
    async fn add_owner(
        owner_principal: &Principal,
        group_identifier: &Principal,
        member_canister: &Principal,
    ) -> Result<(Principal, Group), ApiError> {
        let add_owner_response: Result<(Result<Principal, ApiError>,), _> = call::call(
            member_canister.clone(),
            "add_owner",
            (owner_principal, group_identifier.clone()),
        )
        .await;

        STABLE_DATA.with(|data| match add_owner_response {
            Err(err) => Err(api_error(
                ApiErrorType::BadRequest,
                "OWNER_NOT_ADDED",
                err.1.as_str(),
                Data::get_name(data.borrow().get()).as_str(),
                "add_owner",
                None,
            )),
            Ok((_add_owner_response,)) => match _add_owner_response {
                Err(err) => Err(err),
                Ok(_owner_identifier) => {
                    let group = ENTRIES
                        .with(|entries| Data::get_entry(data, entries, group_identifier.clone()));
                    match group {
                        Err(err) => Err(err),
                        Ok((_identifier, mut _group_data)) => {
                            _group_data.owner = owner_principal.clone();
                            _group_data.updated_on = time();
                            ENTRIES.with(|entries| {
                                Data::update_entry(data, entries, _identifier, _group_data)
                            })
                        }
                    }
                }
            },
        })
    }

    // Method to get a list of all group roles
    pub fn get_group_roles(group_identifier: Principal) -> Vec<GroupRole> {
        let group = STABLE_DATA
            .with(|data| ENTRIES.with(|entries| Data::get_entry(data, entries, group_identifier)));
        if let Ok((_, mut _group)) = group {
            _group.roles.append(&mut default_roles());
            return _group.roles;
        }
        return vec![];
    }

    // TODO: inter-canister call to remove role from members
    // Method to remove custom role from group
    pub fn remove_role(
        caller: Principal,
        group_identifier: Principal,
        role_name: String,
    ) -> Result<bool, ApiError> {
        let inputs = Some(vec![
            format!("caller - {:?}", &caller.to_string()),
            format!("group_id - {:?}", &group_identifier),
            format!("role_name` - {:?}", &role_name),
        ]);

        STABLE_DATA.with(|data| {
            match ENTRIES.with(|entries| Data::get_entry(data, entries, group_identifier)) {
                Err(err) => Err(err),
                Ok((_identifier, mut _group_data)) => {
                    // check if the caller is the owner of the group
                    if _group_data.owner != caller {
                        return Err(api_error(
                            ApiErrorType::Unauthorized,
                            "UNAUTHORIZED",
                            "Only owner of a group can add roles",
                            Data::get_name(data.borrow().get()).as_str(),
                            "add_role",
                            inputs,
                        ));
                    }
                    // check if the role exists
                    let existing_role = _group_data.roles.iter().find(|r| r.name == role_name);

                    match existing_role {
                        None => Err(api_error(
                            ApiErrorType::NotFound,
                            "ROLE_NOT_FOUND",
                            "The role cant be found for this group",
                            Data::get_name(data.borrow().get()).as_str(),
                            "remove_role",
                            inputs,
                        )),
                        Some(_role) => {
                            // if the role is protected (default roles) then return an error
                            if _role.protected {
                                return Err(api_error(
                                    ApiErrorType::BadRequest,
                                    "PROTECTED_ROLE",
                                    "This role is protected from deletion",
                                    Data::get_name(data.borrow().get()).as_str(),
                                    "remove_role",
                                    inputs,
                                ));
                            };

                            // remove the role to update from the existing roles
                            let updated_roles: Vec<GroupRole> = _group_data
                                .roles
                                .iter()
                                .filter(|r| r.name != role_name)
                                .cloned()
                                .collect();

                            _group_data.roles = updated_roles;
                            _group_data.updated_on = time();

                            let update_group_result = ENTRIES.with(|entries| {
                                Data::update_entry(data, entries, _identifier, _group_data)
                            });

                            match update_group_result {
                                Err(err) => Err(err),
                                Ok(_) => Ok(true),
                            }
                        }
                    }
                }
            }
        })
    }

    // TODO: inter-canister call to remove role from members
    // Method to update role permissions for custom roles
    pub fn update_role_permissions(
        caller: Principal,
        group_identifier: Principal,
        role_name: String,
        post_permissions: Vec<PostPermission>,
    ) -> Result<bool, ApiError> {
        let inputs = Some(vec![
            format!("caller - {:?}", &caller.to_string()),
            format!("group_id - {:?}", &role_name),
            format!("role_name - {:?}", &role_name),
            format!("permissions - {:?}", &post_permissions),
        ]);

        STABLE_DATA.with(|data| {
            match ENTRIES.with(|entries| Data::get_entry(data, entries, group_identifier)) {
                Err(err) => Err(err),
                Ok((_identifier, mut _group_data)) => {
                    // check if the caller is the owner of the group
                    if _group_data.owner != caller {
                        return Err(api_error(
                            ApiErrorType::Unauthorized,
                            "UNAUTHORIZED",
                            "Only owner of a group can add roles",
                            Data::get_name(data.borrow().get()).as_str(),
                            "add_role",
                            inputs,
                        ));
                    }

                    // check if the role exists
                    let existing_role = _group_data.roles.iter().find(|r| r.name == role_name);
                    match existing_role {
                        None => Err(api_error(
                            ApiErrorType::NotFound,
                            "ROLE_NOT_FOUND",
                            "The role cant be found for this group",
                            Data::get_name(data.borrow().get()).as_str(),
                            "remove_role",
                            inputs,
                        )),
                        Some(_role) => {
                            let mut permissions: Vec<Permission> = vec![];
                            let required_permissions = get_read_only_permissions();

                            // iterate over the permissions passed as an argument
                            post_permissions.iter().for_each(|p| {
                                let required_permission =
                                    required_permissions.iter().find(|r| r.name == p.name);

                                // check if the permission is a required permission
                                match required_permission {
                                    None => permissions.push(Permission {
                                        name: p.name.clone(),
                                        protected: false,
                                        actions: p.actions.clone(),
                                    }),
                                    Some(_permission) => {
                                        // if the permission is protected set the actions as default
                                        if _permission.protected {
                                            permissions.push(Permission {
                                                name: _permission.name.clone(),
                                                protected: _permission.protected,
                                                actions: _permission.actions.clone(),
                                            })
                                        } else {
                                            permissions.push(Permission {
                                                name: _permission.name.clone(),
                                                protected: _permission.protected,
                                                actions: p.actions.clone(),
                                            })
                                        }
                                    }
                                }
                            });

                            let updated_role = GroupRole {
                                name: _role.name.clone(),
                                protected: _role.protected,
                                permissions,
                                // optional color for frontend consumption
                                color: _role.color.clone(),
                                index: _role.index,
                            };

                            let mut roles: Vec<GroupRole> = _group_data
                                .roles
                                .iter()
                                .filter(|r| &r.name != &_role.name)
                                .cloned()
                                .collect();

                            roles.push(updated_role);

                            _group_data.roles = roles;
                            _group_data.updated_on = time();

                            let update_group_result = ENTRIES.with(|entries| {
                                Data::update_entry(data, entries, _identifier, _group_data)
                            });

                            match update_group_result {
                                Err(err) => Err(err),
                                Ok(_) => Ok(true),
                            }
                        }
                    }
                }
            }
        })
    }

    async fn validate_group_privacy(
        caller: Principal,
        account_identifier: Option<String>,
        privacy: Privacy,
    ) -> Result<(), ApiError> {
        match privacy {
            Privacy::Public => Ok(()),
            Privacy::Private => Ok(()),
            Privacy::InviteOnly => Ok(()),
            Privacy::Gated(gated_type) => {
                let mut is_valid = false;
                use GatedType::*;
                match gated_type {
                    Neuron(neuron_canisters) => {
                        for neuron_canister in neuron_canisters {
                            is_valid = Self::validate_neuron_gated(
                                caller,
                                neuron_canister.governance_canister,
                                neuron_canister.rules,
                            )
                            .await;
                            if is_valid {
                                break;
                            }
                        }
                        if is_valid {
                            Ok(())
                            // If the caller does not own the neuron, throw an error
                        } else {
                            return Err(api_error(
                                ApiErrorType::Unauthorized,
                                "NOT_OWNING_NEURON",
                                "You are not owning this neuron required to join this group",
                                STABLE_DATA
                                    .with(|data| Data::get_name(data.borrow().get()))
                                    .as_str(),
                                "validate_group_privacy",
                                None,
                            ));
                        }
                    }
                    Token(nft_canisters) => {
                        // Loop over the canisters and check if the caller owns a specific NFT (inter-canister call)
                        for nft_canister in nft_canisters {
                            is_valid = Self::validate_nft_gated(
                                caller,
                                account_identifier.clone(),
                                nft_canister,
                            )
                            .await;
                            if is_valid {
                                break;
                            }
                        }
                        if is_valid {
                            Ok(())
                            // If the caller does not own the NFT, throw an error
                        } else {
                            return Err(api_error(
                                ApiErrorType::Unauthorized,
                                "NOT_OWNING_NFT",
                                "You are not owning NFT / token required to join this group",
                                STABLE_DATA
                                    .with(|data| Data::get_name(data.borrow().get()))
                                    .as_str(),
                                "add_invite_or_join_group_to_member",
                                None,
                            ));
                        }
                    }
                }
            }
        }
    }

    // Method to check if the caller owns a specific NFT
    pub async fn validate_nft_gated(
        principal: Principal,
        account_identifier: Option<String>,
        nft_canister: TokenGated,
    ) -> bool {
        // Check if the canister is a EXT, DIP20 or DIP721 canister
        match nft_canister.standard.as_str() {
            // If the canister is a EXT canister, check if the caller owns the NFT
            // This call uses the account_identifier
            "EXT" => match account_identifier {
                Some(_account_identifier) => {
                    let response =
                        ext_balance_of(nft_canister.principal, _account_identifier).await;
                    response as u64 >= nft_canister.amount
                }
                None => false,
            },
            // If the canister is a DIP20 canister, check if the caller owns the NFT
            "DIP20" => {
                let response = dip20_balance_of(nft_canister.principal, principal).await;
                response as u64 >= nft_canister.amount
            }
            // If the canister is a DIP721 canister, check if the caller owns the NFT
            "DIP721" => {
                let response = dip721_balance_of(nft_canister.principal, principal).await;
                response as u64 >= nft_canister.amount
            }
            // If the canister is a LEGACY DIP721 canister, check if the caller owns the NFT
            "DIP721_LEGACY" => {
                let response = legacy_dip721_balance_of(nft_canister.principal, principal).await;
                response as u64 >= nft_canister.amount
            }
            _ => false,
        }
    }

    // Method to check if the caller owns a specific neuron and it applies to the set rules
    pub async fn validate_neuron_gated(
        principal: Principal,
        governance_canister: Principal,
        rules: Vec<NeuronGatedRules>,
    ) -> bool {
        let list_neuron_arg = ListNeurons {
            of_principal: Some(principal),
            limit: 100,
            start_page_at: None,
        };

        let call: Result<(ListNeuronsResponse,), _> =
            call::call(governance_canister, "list_neurons", (list_neuron_arg,)).await;

        match call {
            Ok((neurons,)) => {
                let mut is_valid: HashMap<Vec<u8>, bool> = HashMap::new();
                // iterate over the neurons and check if the neuron applies to all the set rules
                for neuron in neurons.neurons {
                    let neuron_id = neuron.id.unwrap().id;
                    is_valid.insert(neuron_id.clone(), true);
                    for rule in rules.clone() {
                        match rule {
                            NeuronGatedRules::IsDisolving(_) => {
                                match &neuron.dissolve_state {
                                    Some(_state) => {
                                        use DissolveState::*;
                                        match _state {
                                            // neuron is not in a dissolving state
                                            DissolveDelaySeconds(_time) => {
                                                is_valid.insert(neuron_id, false);
                                                break;
                                            }
                                            // means that the neuron is in a dissolving state
                                            WhenDissolvedTimestampSeconds(_time) => {}
                                        }
                                    }
                                    None => {
                                        is_valid.insert(neuron_id, false);
                                        break;
                                    }
                                }
                            }
                            NeuronGatedRules::MinAge(_min_age_in_seconds) => {
                                if neuron.created_timestamp_seconds < _min_age_in_seconds {
                                    is_valid.insert(neuron_id, false);
                                    break;
                                }
                            }
                            NeuronGatedRules::MinStake(_min_stake) => {
                                let neuron_stake =
                                    neuron.cached_neuron_stake_e8s as f64 / 100_000_000.0;
                                let min_stake = _min_stake as f64 / 100_000_000.0;

                                if neuron_stake.ceil() < min_stake.ceil() {
                                    is_valid.insert(neuron_id, false);
                                    break;
                                }
                            }
                            NeuronGatedRules::MinDissolveDelay(_min_dissolve_delay_in_seconds) => {
                                match &neuron.dissolve_state {
                                    Some(_state) => {
                                        use DissolveState::*;
                                        match _state {
                                            // neuron is not in a dissolving state, time is locking period in seconds
                                            DissolveDelaySeconds(_dissolve_delay_in_seconds) => {
                                                if &_min_dissolve_delay_in_seconds
                                                    > _dissolve_delay_in_seconds
                                                {
                                                    is_valid.insert(neuron_id, false);
                                                    break;
                                                }
                                            }
                                            // if the neuron is dissolving, make invalid
                                            // means that the neuron is in a dissolving state, timestamp when neuron is done dissolving in seconds
                                            WhenDissolvedTimestampSeconds(_) => {
                                                is_valid.insert(neuron_id, false);
                                                break;
                                            }
                                        }
                                    }
                                    None => {
                                        is_valid.insert(neuron_id, false);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                return is_valid.iter().any(|v| v.1 == &true);
            }
            Err(_) => false,
        }
    }

    // need to call the member canister to transfer the ownership to a new member
    // pub fn transfer_ownership(
    //     caller: Principal,
    //     group_id: u64,
    //     principal: Principal,
    // ) -> Result<bool, ApiError> {
    //     let inputs = Some(vec![
    //         format!("caller - {:?}", &caller.to_string()),
    //         format!("group_id - {:?}", &group_id),
    //         format!("principal - {:?}", &principal.to_string()),
    //     ]);
    //     let _group = Data::get_entry::<Group>(group_id);
    //     match _group {
    //         Err(err) => Err(err),
    //         Ok(_group_data) => {
    //             if _group_data.data.owner.principal != caller {
    //                 return Err(api_error(
    //                     ApiErrorType::Unauthorized,
    //                     "CANT_TRANSFER_OWNERSHIP",
    //                     "Only the owner of the group can transfer ownership",
    //                     Data::get_name().as_str(),
    //                     "delete_group",
    //                     inputs,
    //                 ));
    //             }

    //             let updated_group = Group {
    //                 name: _group_data.data.name,
    //                 description: _group_data.data.description,
    //                 website: _group_data.data.website,
    //                 location: _group_data.data.location,
    //                 privacy: _group_data.data.privacy,
    //                 owner: principal,
    //                 matrix_space_id: _group_data.data.matrix_space_id,
    //                 image: _group_data.data.image,
    //                 banner_image: _group_data.data.banner_image,
    //                 tags: _group_data.data.tags,
    //                 roles: _group_data.data.roles,
    //                 is_deleted: _group_data.data.is_deleted,
    //                 updated_on: time(),
    //                 created_on: _group_data.data.created_on,
    //             };
    //             let update_group_result =
    //                 Data::update_entry::<Group>(_group_data.identifier.id, updated_group.clone());

    //             match update_group_result {
    //                 Err(err) => Err(err),
    //                 Ok(_) => Ok(true),
    //             }
    //         }
    //     }
    // }

    // Method to get filtered groups
    fn get_filtered_groups(
        mut groups: Vec<GroupResponse>,
        filters: Vec<GroupFilter>,
        filter_type: FilterType,
    ) -> Vec<GroupResponse> {
        if let FilterType::Or = filter_type {
            if filters.len() == 0 {
                return groups;
            }
        }

        match filter_type {
            // this filter type will return groups that match all the filters
            FilterType::And => {
                for filter in filters {
                    match filter {
                        GroupFilter::Name(value) => {
                            groups = groups
                                .into_iter()
                                .filter(|group| {
                                    group.name.to_lowercase().contains(&value.to_lowercase())
                                })
                                .collect();
                        }
                        GroupFilter::Tag(value) => {
                            groups = groups
                                .into_iter()
                                .filter(|group| group.tags.contains(&value))
                                .collect();
                        }
                        GroupFilter::UpdatedOn(value) => {
                            groups = groups
                                .into_iter()
                                .filter(|group| {
                                    if value.end_date > 0 {
                                        return group.updated_on >= value.start_date
                                            && group.updated_on <= value.end_date;
                                    } else {
                                        return group.updated_on >= value.start_date;
                                    }
                                })
                                .collect();
                        }
                        GroupFilter::CreatedOn(value) => {
                            groups = groups
                                .into_iter()
                                .filter(|group| {
                                    if value.end_date > 0 {
                                        return group.created_on >= value.start_date
                                            && group.created_on <= value.end_date;
                                    } else {
                                        return group.created_on >= value.start_date;
                                    }
                                })
                                .collect();
                        }
                        GroupFilter::Identifiers(value) => {
                            groups = groups
                                .into_iter()
                                .filter(|group| value.contains(&group.identifier))
                                .collect();
                        }
                        GroupFilter::Owner(value) => {
                            groups = groups
                                .into_iter()
                                .filter(|group| group.owner == value)
                                .collect();
                        }
                        GroupFilter::MemberCount(value) => {
                            groups = groups
                                .into_iter()
                                .filter(|group| {
                                    group.member_count >= value.0 && group.member_count <= value.1
                                })
                                .collect();
                        }
                    }
                }
                groups
            }
            // This filter type will return groups that match any of the filters
            FilterType::Or => {
                let mut hashmap_groups: HashMap<Principal, GroupResponse> = HashMap::new();
                for filter in filters {
                    match filter {
                        GroupFilter::Name(value) => {
                            groups
                                .iter()
                                .filter(|group| {
                                    group.name.to_lowercase().contains(&value.to_lowercase())
                                })
                                .for_each(|v| {
                                    hashmap_groups.insert(v.identifier.clone(), v.clone());
                                });
                        }
                        GroupFilter::Tag(value) => {
                            groups
                                .iter()
                                .filter(|group| group.tags.contains(&value))
                                .for_each(|v| {
                                    hashmap_groups.insert(v.identifier.clone(), v.clone());
                                });
                        }
                        GroupFilter::UpdatedOn(value) => {
                            groups
                                .iter()
                                .filter(|group| {
                                    if value.end_date > 0 {
                                        return group.updated_on >= value.start_date
                                            && group.updated_on <= value.end_date;
                                    } else {
                                        return group.updated_on >= value.start_date;
                                    }
                                })
                                .for_each(|v| {
                                    hashmap_groups.insert(v.identifier.clone(), v.clone());
                                });
                        }
                        GroupFilter::CreatedOn(value) => {
                            groups
                                .iter()
                                .filter(|group| {
                                    if value.end_date > 0 {
                                        return group.created_on >= value.start_date
                                            && group.created_on <= value.end_date;
                                    } else {
                                        return group.created_on >= value.start_date;
                                    }
                                })
                                .for_each(|v| {
                                    hashmap_groups.insert(v.identifier.clone(), v.clone());
                                });
                        }
                        GroupFilter::Identifiers(value) => {
                            groups
                                .iter()
                                .filter(|group| value.contains(&group.identifier))
                                .for_each(|v| {
                                    hashmap_groups.insert(v.identifier.clone(), v.clone());
                                });
                        }
                        GroupFilter::Owner(value) => {
                            groups
                                .iter()
                                .filter(|group| group.owner == value)
                                .for_each(|v| {
                                    hashmap_groups.insert(v.identifier.clone(), v.clone());
                                });
                        }
                        GroupFilter::MemberCount(value) => {
                            groups
                                .iter()
                                .filter(|group| {
                                    group.member_count >= value.0 && group.member_count <= value.1
                                })
                                .for_each(|v| {
                                    hashmap_groups.insert(v.identifier.clone(), v.clone());
                                });
                        }
                    }
                }

                hashmap_groups.into_iter().map(|v| v.1).collect()
            }
        }
    }

    // Method to get sorted groups
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

    // Method to map groups to a default response that can be used on the frontend
    pub fn map_group_to_group_response(identifier: String, group: Group) -> GroupResponse {
        let mut roles = group.roles;
        roles.append(&mut default_roles());
        GroupResponse {
            identifier: Principal::from_text(identifier).unwrap_or(Principal::anonymous()),
            name: group.name,
            description: group.description,
            website: group.website,
            location: group.location,
            privacy: group.privacy,
            owner: group.owner,
            created_by: group.created_by,
            matrix_space_id: group.matrix_space_id,
            image: group.image,
            banner_image: group.banner_image,
            tags: group.tags,
            wallets: group
                .wallets
                .into_iter()
                .map(|(key, value)| (key, value))
                .collect(),
            roles,
            member_count: group.member_count.into_iter().map(|(_, value)| value).sum(),
            is_deleted: group.is_deleted,
            updated_on: group.updated_on,
            created_on: group.created_on,
        }
    }

    // This method is used as an inter canister call to update the member count per canister
    // Member count is used for backend filtering
    // TODO: distinct member_canister and caller
    pub fn update_member_count(
        group_identifier: Principal,
        member_canister: Principal,
        member_count: usize,
    ) -> Result<(), bool> {
        let (_, _, _group_kind) = Identifier::decode(&group_identifier);

        if IDENTIFIER_KIND != _group_kind {
            return Err(false);
        };

        STABLE_DATA.with(|data| {
            let existing = ENTRIES.with(|entries| Data::get_entry(data, entries, group_identifier));
            match existing {
                Ok((_, mut _group)) => {
                    _group.member_count.insert(member_canister, member_count);
                    let _ = ENTRIES.with(|entries| {
                        Data::update_entry(data, entries, group_identifier, _group)
                    });
                    Ok(())
                }
                Err(_) => Err(false),
            }
        })
    }

    // This method is used for role / permission based access control
    pub async fn can_write(
        caller: Principal,
        group_identifier: Principal,
        member_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        Self::check_permission(
            caller,
            group_identifier,
            member_identifier,
            PermissionActionType::Write,
        )
        .await
    }

    // This method is used for role / permission based access control
    pub async fn can_read(
        caller: Principal,
        group_identifier: Principal,
        member_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        Self::check_permission(
            caller,
            group_identifier,
            member_identifier,
            PermissionActionType::Read,
        )
        .await
    }

    // This method is used for role / permission based access control
    pub async fn can_edit(
        caller: Principal,
        group_identifier: Principal,
        member_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        if let Ok((_, _group)) = STABLE_DATA
            .with(|data| ENTRIES.with(|entries| Data::get_entry(data, entries, group_identifier)))
        {
            if _group.owner == caller {
                return Ok(caller);
            }
        }

        Self::check_permission(
            caller,
            group_identifier,
            member_identifier,
            PermissionActionType::Edit,
        )
        .await
    }

    // This method is used for role / permission based access control
    pub async fn can_delete(
        caller: Principal,
        group_identifier: Principal,
        member_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        Self::check_permission(
            caller,
            group_identifier,
            member_identifier,
            PermissionActionType::Delete,
        )
        .await
    }

    // Global method to determine if a member has a specific permission
    async fn check_permission(
        caller: Principal,
        group_identifier: Principal,
        member_identifier: Principal,
        permission: PermissionActionType,
    ) -> Result<Principal, ApiError> {
        let mut group_roles = Store::get_group_roles(group_identifier);
        group_roles.append(&mut default_roles());
        let member_roles = get_member_roles(member_identifier, group_identifier).await;

        match member_roles {
            Ok((_principal, _roles)) => {
                if caller != _principal {
                    return Err(api_error(
                        ApiErrorType::Unauthorized,
                        "PRINCIPAL_MISMATCH",
                        "Principal mismatch",
                        STABLE_DATA
                            .with(|data| Data::get_name(data.borrow().get()))
                            .as_str(),
                        "check_permission",
                        None,
                    ));
                }

                let has_permission = has_permission(
                    &_roles,
                    &PermissionType::Group(None),
                    &group_roles,
                    &permission,
                );

                if !has_permission {
                    return Err(api_error(
                        ApiErrorType::Unauthorized,
                        "NO_PERMISSION",
                        "No permission",
                        STABLE_DATA
                            .with(|data| Data::get_name(data.borrow().get()))
                            .as_str(),
                        "check_permission",
                        Some(vec![
                            serde_json::to_string(&_roles).unwrap(),
                            serde_json::to_string(&group_roles).unwrap(),
                        ]),
                    ));
                }

                Ok(caller)
            }
            Err(err) => Err(api_error(
                ApiErrorType::Unauthorized,
                "NO_PERMISSION",
                err.as_str(),
                STABLE_DATA
                    .with(|data| Data::get_name(data.borrow().get()))
                    .as_str(),
                "check_permission",
                None,
            )),
        }
    }
}
