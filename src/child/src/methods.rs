use std::collections::HashMap;

use candid::Principal;
use ic_cdk::{
    api::call::{self, RejectionCode},
    caller, query, update,
};

use ic_scalable_misc::{
    enums::{
        api_error_type::ApiError, filter_type::FilterType, privacy_type::Privacy,
        sort_type::SortDirection,
    },
    models::{
        group_role::GroupRole, paged_response_models::PagedResponse,
        permissions_models::PostPermission,
    },
};
use shared::group_model::{Group, GroupFilter, GroupResponse, GroupSort, PostGroup, UpdateGroup};

use crate::store::ENTRIES;

use super::store::{Store, DATA};

#[update]
pub async fn set_entry_count() -> Result<(), String> {
    if caller()
        == Principal::from_text("ledm3-52ncq-rffuv-6ed44-hg5uo-iicyu-pwkzj-syfva-heo4k-p7itq-aqe")
            .unwrap()
    {
        DATA.with(|d| {
            let mut old_data = d.borrow().get().clone();
            old_data.current_entry_id = 1000;
            let _ = d.borrow_mut().set(old_data);
        })
    }

    return Ok(());
}

#[update]
pub async fn migration_add_groups() -> Result<(), String> {
    if caller()
        != Principal::from_text("ledm3-52ncq-rffuv-6ed44-hg5uo-iicyu-pwkzj-syfva-heo4k-p7itq-aqe")
            .unwrap()
    {
        return Err("Unauthorized".to_string());
    }
    let result: Result<(Result<PagedResponse<GroupResponse>, ApiError>,), (RejectionCode, String)> =
        call::call(
            Principal::from_text("5rvte-7aaaa-aaaap-aa4ja-cai").unwrap(),
            "get_groups",
            (
                1000 as u64,
                1 as u64,
                Vec::<GroupFilter>::new(),
                FilterType::And,
                GroupSort::Name(SortDirection::Asc),
                false,
            ),
        )
        .await;

    match result {
        Err((_, err)) => {
            return Err(err);
        }
        Ok((Err(err),)) => {
            return Err(err.to_string());
        }
        Ok((Ok(groups),)) => {
            ENTRIES.with(|data| {
                // data.borrow_mut().entries = HashMap::from_iter(groups);
                groups.data.into_iter().for_each(|g| {
                    data.borrow_mut().insert(
                        g.identifier.to_string(),
                        Group {
                            name: g.name,
                            description: g.description,
                            website: g.website,
                            location: g.location,
                            privacy: g.privacy,
                            owner: g.owner,
                            created_by: g.created_by,
                            matrix_space_id: g.matrix_space_id,
                            image: g.image,
                            banner_image: g.banner_image,
                            tags: g.tags,
                            roles: g.roles,
                            is_deleted: g.is_deleted,
                            member_count: HashMap::new(),
                            wallets: HashMap::new(),
                            updated_on: g.updated_on,
                            created_on: g.created_on,
                        },
                    );
                });
            });
            Ok(())
        }
    }
}

// #[update]
// pub fn migration_add_groups(groups: Vec<(Principal, Group)>) -> () {
//     if caller()
//         == Principal::from_text("ledm3-52ncq-rffuv-6ed44-hg5uo-iicyu-pwkzj-syfva-heo4k-p7itq-aqe")
//             .unwrap()
//     {
//         DATA.with(|data| {
//             let _ = data.borrow_mut().set(Data {
//                 current_entry_id: groups.clone().len() as u64,
//                 ..data.borrow().get().clone()
//             });
//         });

//         ENTRIES.with(|data| {
//             // data.borrow_mut().entries = HashMap::from_iter(groups);
//             groups.into_iter().for_each(|(k, v)| {
//                 data.borrow_mut().insert(k.to_string(), v);
//             });
//         });
//     }
// }

// This method is used to add a group to the canister,
// The method is async because it optionally creates a new canister is created
#[update]
async fn add_group(
    post_group: PostGroup,
    member_canister: Principal,
    account_identifier: Option<String>,
) -> Result<GroupResponse, ApiError> {
    Store::add_group(caller(), post_group, member_canister, account_identifier).await
}

// This method is used to get a group from the canister
#[query]
fn get_group(identifier: Principal) -> Result<GroupResponse, ApiError> {
    Store::get_group(identifier)
}

// This method is used to get groups filtered and sorted with pagination
#[query]
fn get_groups(
    limit: usize,
    page: usize,
    filters: Vec<GroupFilter>,
    filter_type: FilterType,
    sort: GroupSort,
    include_invite_only: bool,
) -> Result<PagedResponse<GroupResponse>, ApiError> {
    Ok(Store::get_groups(
        limit,
        page,
        filters,
        filter_type,
        sort,
        include_invite_only,
    ))
}

// This method is used to edit a group
#[update]
async fn edit_group(
    group_identifier: Principal,
    update_group: UpdateGroup,
    member_identifier: Principal,
) -> Result<GroupResponse, ApiError> {
    match Store::can_edit(caller(), group_identifier, member_identifier).await {
        Ok(_caller) => Store::update_group(_caller, group_identifier, update_group),
        Err(err) => Err(err),
    }
}

// COMPOSITE_QUERY PREPARATION
// This methods is used by the parent canister to get filtered groups the (this) child canister
// Data serialized and send as byte array chunks ` (bytes, (start_chunk, end_chunk)) `
// The parent canister can then deserialize the data and pass it to the frontend
#[query]
fn get_chunked_data(
    filters: Vec<GroupFilter>,
    filter_type: FilterType,
    chunk: usize,
    max_bytes_per_chunk: usize,
) -> (Vec<u8>, (usize, usize)) {
    if DATA.with(|data| data.borrow().get().parent != caller()) {
        return (vec![], (0, 0));
    }

    Store::get_chunked_data(filters, filter_type, chunk, max_bytes_per_chunk)
}

// This method is used to get the owner and privacy of a group
// This is used for inter-canister calls to determine is a user can do a group specific action
#[query]
fn get_group_owner_and_privacy(
    group_identifier: Principal,
) -> Result<(Principal, Privacy), ApiError> {
    Store::get_group_owner_and_privacy(group_identifier)
}

// Get multiple groups by their identifiers
#[query]
fn get_groups_by_id(group_identifiers: Vec<Principal>) -> Result<Vec<GroupResponse>, ApiError> {
    Ok(Store::get_groups_by_id(group_identifiers))
}

// This method is used to (soft) delete a group
#[update]
async fn delete_group(
    group_identifier: Principal,
    member_identifier: Principal,
) -> Result<Group, ApiError> {
    match Store::can_delete(caller(), group_identifier, member_identifier).await {
        Ok(_caller) => Store::delete_group(_caller, group_identifier),
        Err(err) => Err(err),
    }
}

#[update]
pub fn add_wallet(
    group_identifier: Principal,
    wallet_canister: Principal,
    description: String,
) -> Result<(), ApiError> {
    Store::add_wallet(caller(), group_identifier, wallet_canister, description)
}

#[update]
pub fn remove_wallet(
    group_identifier: Principal,
    wallet_canister: Principal,
) -> Result<(), ApiError> {
    Store::remove_wallet(caller(), group_identifier, wallet_canister)
}

// This method is used to add a custom role to a group
#[update]
async fn add_role(
    group_identifier: Principal,
    role_name: String,
    color: String,
    index: u64,
    member_identifier: Principal,
) -> Result<GroupRole, ApiError> {
    match Store::can_edit(caller(), group_identifier, member_identifier).await {
        Ok(_caller) => Store::add_role(_caller, group_identifier, role_name, color, index),
        Err(err) => Err(err),
    }
}

// This method is used to remove a custom role from a group
#[update]
async fn remove_role(
    group_identifier: Principal,
    role_name: String,
    member_identifier: Principal,
) -> Result<bool, ApiError> {
    match Store::can_edit(caller(), group_identifier, member_identifier).await {
        Ok(_caller) => Store::remove_role(_caller, group_identifier, role_name),
        Err(err) => Err(err),
    }
}

// This method is used to get all the roles of a group
#[query]
fn get_group_roles(group_identifier: Principal) -> Vec<GroupRole> {
    Store::get_group_roles(group_identifier)
}

// This method is used to update the persmissions of a specific role
#[update]
async fn edit_role_permissions(
    group_identifier: Principal,
    role_name: String,
    post_permissions: Vec<PostPermission>,
    member_identifier: Principal,
) -> Result<bool, ApiError> {
    match Store::can_edit(caller(), group_identifier, member_identifier).await {
        Ok(_caller) => {
            Store::update_role_permissions(_caller, group_identifier, role_name, post_permissions)
        }
        Err(err) => Err(err),
    }
}

// This method is used as an inter canister call to update the member count per canister
// Member count is used for backend filtering
// TODO: distinct member_canister and caller
#[update]
pub fn update_member_count(
    group_identifier: Principal,
    member_canister: Principal,
    member_count: usize,
) -> Result<(), bool> {
    if caller() == member_canister {
        return Store::update_member_count(group_identifier, member_canister, member_count);
    }
    return Err(false);
}
