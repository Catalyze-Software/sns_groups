use std::{collections::HashMap, iter::FromIterator, vec};

use candid::Principal;
use ic_cdk::api::{call, time};
use ic_scalable_canister::store::Data;
use ic_scalable_misc::{
    enums::{
        api_error_type::{ApiError, ApiErrorType},
        filter_type::FilterType,
        privacy_type::Privacy,
        sort_type::SortDirection,
    },
    helpers::{
        error_helper::api_error,
        paging_helper::get_paged_data,
        role_helper::{default_roles, get_member_roles, get_read_only_permissions, has_permission},
        serialize_helper::serialize,
    },
    models::{
        group_role::GroupRole,
        identifier_model::Identifier,
        paged_response_models::PagedResponse,
        permissions_models::{Permission, PermissionActionType, PermissionType, PostPermission},
    },
};

use shared::group_model::{Group, GroupFilter, GroupResponse, GroupSort, PostGroup, UpdateGroup};
use std::cell::RefCell;

use crate::validation::validate_post_group;

use super::validation::validate_update_group;

thread_local! {
    pub static DATA: RefCell<Data<Group>> = RefCell::new(Data::default());
}

pub struct Store;

impl Store {
    pub async fn add_group(
        caller: Principal,
        post_group: PostGroup,
        member_canister: Principal,
    ) -> Result<GroupResponse, ApiError> {
        let temp_group = post_group.clone();
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
            member_count: HashMap::from_iter(vec![(member_canister, 1)].into_iter()),
            roles: vec![],
            is_deleted: false,
            updated_on: time(),
            created_on: time(),
        };

        let add_entry_result = DATA.with(|data| {
            let validate = validate_post_group(post_group);
            match validate {
                Err(err) => Err(err),
                Ok(_) => match Data::add_entry(data, new_group.clone(), Some("grp".to_string())) {
                    Err(err) => Err(err),
                    Ok(result) => Ok(result),
                },
            }
        });

        match add_entry_result {
            Err(err) => match err {
                ApiError::CanisterAtCapacity(message) => {
                    let _data = DATA.with(|v| v.borrow().clone());
                    match Data::spawn_sibling(_data, None, new_group.clone()).await {
                        Ok(_) => Err(ApiError::CanisterAtCapacity(message)),
                        Err(err) => Err(err),
                    }
                }
                _ => Err(err),
            },
            Ok((_identifier, _group_data)) => {
                match Self::add_owner(&caller, &_identifier, &member_canister).await {
                    Err(err) => {
                        DATA.with(|data| Data::remove_entry(data, &_identifier));
                        Err(err)
                    }
                    Ok((_identifier, _group_data)) => {
                        Ok(Self::map_group_to_group_response(_identifier, _group_data))
                    }
                }
            }
        }
    }

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
        let validate = validate_update_group(update_group.clone());
        DATA.with(|data| match validate {
            Err(err) => Err(err),
            Ok(_) => {
                let existing = Data::get_entry(data, group_identifier);

                match existing {
                    Err(err) => Err(err),
                    Ok((_identifier, mut _group_data)) => {
                        if _group_data.is_deleted {
                            return Err(api_error(
                                ApiErrorType::BadRequest,
                                "DELETED_GROUP",
                                "You cant update a deleted group",
                                Data::get_name(data).as_str(),
                                "update_group",
                                inputs,
                            ));
                        }

                        let updated_group = Group {
                            name: update_group.name,
                            description: update_group.description,
                            website: update_group.website,
                            location: update_group.location,
                            privacy: update_group.privacy,
                            image: update_group.image,
                            banner_image: update_group.banner_image,
                            tags: update_group.tags,
                            updated_on: time(),
                            .._group_data
                        };
                        let update_group_result =
                            Data::update_entry(data, group_identifier, updated_group.clone());
                        match update_group_result {
                            Err(err) => Err(err),
                            Ok((_identifier, _group_data)) => {
                                Ok(Self::map_group_to_group_response(_identifier, _group_data))
                            }
                        }
                    }
                }
            }
        })
    }

    pub fn delete_group(caller: Principal, identifier: Principal) -> Result<Group, ApiError> {
        let inputs = Some(vec![
            format!("caller - {:?}", &caller),
            format!("id - {:?}", &identifier),
        ]);
        DATA.with(|data| {
            let existing = Data::get_entry(data, identifier);
            match existing {
                Err(err) => Err(err),
                Ok((_identifier, mut _group_data)) => {
                    if _group_data.owner != caller {
                        return Err(api_error(
                            ApiErrorType::Unauthorized,
                            "CANT_DELETE_GROUP",
                            "Only the owner can delete the group",
                            Data::get_name(data).as_str(),
                            "delete_group",
                            inputs,
                        ));
                    }

                    _group_data.is_deleted = true;
                    _group_data.updated_on = time();

                    let update_group_result =
                        Data::update_entry(data, _identifier, _group_data.clone());

                    match update_group_result {
                        Err(err) => Err(err),
                        Ok(_) => Ok(_group_data),
                    }
                }
            }
        })
    }

    pub fn get_group(identifier: Principal) -> Result<GroupResponse, ApiError> {
        DATA.with(|data| {
            let group = Data::get_entry(data, identifier);
            match group {
                Err(err) => Err(err),
                Ok((_identifier, _group_data)) => {
                    Ok(Self::map_group_to_group_response(_identifier, _group_data))
                }
            }
        })
    }

    pub fn get_group_owner_and_privacy(
        identifier: Principal,
    ) -> Result<(Principal, Privacy), ApiError> {
        DATA.with(|data| {
            let group = Data::get_entry(data, identifier);
            match group {
                Err(err) => Err(err),
                Ok((_, _group_data)) => Ok((_group_data.owner, _group_data.privacy)),
            }
        })
    }

    pub fn get_groups(
        limit: usize,
        page: usize,
        filters: Vec<GroupFilter>,
        filter_type: FilterType,
        sort: GroupSort,
    ) -> PagedResponse<GroupResponse> {
        let groups = DATA.with(|data| Data::get_entries(data));
        let mapped_groups: Vec<GroupResponse> = groups
            .iter()
            .filter(|(_identifier, _group_data)| !_group_data.is_deleted)
            .map(|(_identifier, _group_data)| {
                Self::map_group_to_group_response(_identifier.clone(), _group_data.clone())
            })
            .collect();

        let filtered_groups = Self::get_filtered_groups(mapped_groups, filters, filter_type);
        let ordered_groups = Self::get_ordered_groups(filtered_groups, sort);

        get_paged_data(ordered_groups, limit, page)
    }

    pub fn get_chunked_data(
        filters: Vec<GroupFilter>,
        filter_type: FilterType,
        chunk: usize,
        max_bytes_per_chunk: usize,
    ) -> (Vec<u8>, (usize, usize)) {
        let groups = DATA.with(|data| Data::get_entries(data));
        let mapped_groups: Vec<GroupResponse> = groups
            .iter()
            .filter(|(_identifier, _group_data)| !_group_data.is_deleted)
            .map(|(_identifier, _group_data)| {
                Self::map_group_to_group_response(_identifier.clone(), _group_data.clone())
            })
            .collect();

        let filtered_groups = Self::get_filtered_groups(mapped_groups, filters, filter_type);
        if let Ok(bytes) = serialize(&filtered_groups) {
            if bytes.len() >= max_bytes_per_chunk {
                let start = chunk * max_bytes_per_chunk;
                let end = (chunk + 1) * (max_bytes_per_chunk);

                let response = if end >= bytes.len() {
                    bytes[start..].to_vec()
                } else {
                    bytes[start..end].to_vec()
                };

                let mut max_chunks: f64 = 0.00;
                if max_bytes_per_chunk < bytes.len() {
                    max_chunks = (bytes.len() / max_bytes_per_chunk) as f64;
                }
                return (response, (chunk, max_chunks.ceil() as usize));
            }
            return (bytes, (0, 0));
        } else {
            return (vec![], (0, 0));
        }
    }

    pub fn get_groups_by_id(group_ids: Vec<Principal>) -> Vec<GroupResponse> {
        DATA.with(|data| {
            let mut groups: Vec<GroupResponse> = vec![];

            group_ids.into_iter().for_each(|_identifier| {
                let existing = Data::get_entry(data, _identifier);

                match existing {
                    Err(_) => {}
                    Ok((_identifier, _group_data)) => {
                        if !_group_data.is_deleted {
                            groups.push(Self::map_group_to_group_response(_identifier, _group_data))
                        }
                    }
                };
            });
            groups
        })
    }

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
        DATA.with(|data| match Data::get_entry(data, group_identifier) {
            Err(err) => Err(err),
            Ok((_identifier, mut _group_data)) => {
                if _group_data.owner != caller {
                    return Err(api_error(
                        ApiErrorType::Unauthorized,
                        "UNAUTHORIZED",
                        "Only owner of a group can add roles",
                        Data::get_name(data).as_str(),
                        "add_role",
                        inputs,
                    ));
                }

                let mut roles = _group_data.roles;
                let included_role = roles.iter().any(|r| r.name == role_name);

                if included_role || default_roles().iter().any(|r| r.name == role_name) {
                    return Err(api_error(
                        ApiErrorType::BadRequest,
                        "EXISTING_ROLE",
                        "This role is already registered",
                        Data::get_name(data).as_str(),
                        "add_role",
                        inputs,
                    ));
                };

                let new_role = GroupRole {
                    name: role_name,
                    protected: false,
                    permissions: get_read_only_permissions(),
                    color,
                    index: Some(index),
                };

                roles.push(new_role.clone());

                _group_data.roles = roles;
                _group_data.updated_on = time();

                let update_group_result =
                    Data::update_entry(data, group_identifier, _group_data.clone());

                match update_group_result {
                    Err(err) => Err(err),
                    Ok(_) => Ok(new_role),
                }
            }
        })
    }

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

        DATA.with(|data| match add_owner_response {
            Err(err) => Err(api_error(
                ApiErrorType::BadRequest,
                "OWNER_NOT_ADDED",
                err.1.as_str(),
                Data::get_name(data).as_str(),
                "add_owner",
                None,
            )),
            Ok((_add_owner_response,)) => match _add_owner_response {
                Err(err) => Err(err),
                Ok(_owner_identifier) => {
                    let group = Data::get_entry(data, group_identifier.clone());
                    match group {
                        Err(err) => Err(err),
                        Ok((_identifier, mut _group_data)) => {
                            _group_data.owner = owner_principal.clone();
                            _group_data.updated_on = time();
                            Data::update_entry(data, _identifier, _group_data)
                        }
                    }
                }
            },
        })
    }

    pub fn get_group_roles(group_identifier: Principal) -> Vec<GroupRole> {
        let group = DATA.with(|data| Data::get_entry(data, group_identifier));
        if let Ok((_, mut _group)) = group {
            _group.roles.append(&mut default_roles());
            return _group.roles;
        }
        return vec![];
    }

    // TODO: inter-canister call to remove role from members
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

        DATA.with(|data| {
            let existing_group = Data::get_entry(data, group_identifier);

            match existing_group {
                Err(err) => Err(err),
                Ok((_identifier, mut _group_data)) => {
                    if _group_data.owner != caller {
                        return Err(api_error(
                            ApiErrorType::Unauthorized,
                            "UNAUTHORIZED",
                            "Only owner of a group can add roles",
                            Data::get_name(data).as_str(),
                            "add_role",
                            inputs,
                        ));
                    }
                    let existing_role = _group_data
                        .roles
                        .iter()
                        .find(|r| r.name == role_name && r.protected);

                    match existing_role {
                        None => Err(api_error(
                            ApiErrorType::NotFound,
                            "ROLE_NOT_FOUND",
                            "The role cant be found for this group",
                            Data::get_name(data).as_str(),
                            "remove_role",
                            inputs,
                        )),
                        Some(_role) => {
                            if _role.protected {
                                return Err(api_error(
                                    ApiErrorType::BadRequest,
                                    "PROTECTED_ROLE",
                                    "This role is protected from deletion",
                                    Data::get_name(data).as_str(),
                                    "remove_role",
                                    inputs,
                                ));
                            };

                            let updated_roles: Vec<GroupRole> = _group_data
                                .roles
                                .iter()
                                .filter(|r| r.name == role_name)
                                .cloned()
                                .collect();

                            _group_data.roles = updated_roles;
                            _group_data.updated_on = time();

                            let update_group_result =
                                Data::update_entry(data, _identifier, _group_data);

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

        DATA.with(|data| match Data::get_entry(data, group_identifier) {
            Err(err) => Err(err),
            Ok((_identifier, mut _group_data)) => {
                if _group_data.owner != caller {
                    return Err(api_error(
                        ApiErrorType::Unauthorized,
                        "UNAUTHORIZED",
                        "Only owner of a group can add roles",
                        Data::get_name(data).as_str(),
                        "add_role",
                        inputs,
                    ));
                }
                let existing_role = _group_data.roles.iter().find(|r| r.name == role_name);
                match existing_role {
                    None => Err(api_error(
                        ApiErrorType::NotFound,
                        "ROLE_NOT_FOUND",
                        "The role cant be found for this group",
                        Data::get_name(data).as_str(),
                        "remove_role",
                        inputs,
                    )),
                    Some(_role) => {
                        let mut permissions: Vec<Permission> = vec![];
                        let required_permissions = get_read_only_permissions();

                        post_permissions.iter().for_each(|p| {
                            let required_permission =
                                required_permissions.iter().find(|r| r.name == p.name);

                            match required_permission {
                                None => permissions.push(Permission {
                                    name: p.name.clone(),
                                    protected: false,
                                    actions: p.actions.clone(),
                                }),
                                Some(_permission) => {
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

                        let update_group_result =
                            Data::update_entry(data, _identifier, _group_data);

                        match update_group_result {
                            Err(err) => Err(err),
                            Ok(_) => Ok(true),
                        }
                    }
                }
            }
        })
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

    fn get_filtered_groups(
        mut groups: Vec<GroupResponse>,
        filters: Vec<GroupFilter>,
        filter_type: FilterType,
    ) -> Vec<GroupResponse> {
        match filter_type {
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

    pub fn map_group_to_group_response(identifier: Principal, group: Group) -> GroupResponse {
        let mut roles = group.roles;
        roles.append(&mut default_roles());
        GroupResponse {
            identifier,
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
            roles,
            member_count: group.member_count.into_iter().map(|(_, value)| value).sum(),
            is_deleted: group.is_deleted,
            updated_on: group.updated_on,
            created_on: group.created_on,
        }
    }

    pub fn update_member_count(
        group_identifier: Principal,
        member_canister: Principal,
        member_count: usize,
    ) -> Result<(), bool> {
        let (_, _, _group_kind) = Identifier::decode(&group_identifier);

        if "grp" != _group_kind {
            return Err(false);
        };

        DATA.with(|data| {
            let existing = Data::get_entry(data, group_identifier);
            match existing {
                Ok((_, mut _group)) => {
                    _group.member_count.insert(member_canister, member_count);
                    let _ = Data::update_entry(data, group_identifier, _group);
                    Ok(())
                }
                Err(_) => Err(false),
            }
        })
    }

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

    pub async fn can_edit(
        caller: Principal,
        group_identifier: Principal,
        member_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        Self::check_permission(
            caller,
            group_identifier,
            member_identifier,
            PermissionActionType::Edit,
        )
        .await
    }

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
                        DATA.with(|data| Data::get_name(data)).as_str(),
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
                        DATA.with(|data| Data::get_name(data)).as_str(),
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
                DATA.with(|data| Data::get_name(data)).as_str(),
                "check_permission",
                None,
            )),
        }
    }
}
