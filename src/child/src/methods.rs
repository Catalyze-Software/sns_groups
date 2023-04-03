use candid::{candid_method, Principal};
use ic_cdk::caller;
use ic_cdk_macros::{query, update};

use ic_scalable_misc::{
    enums::{api_error_type::ApiError, filter_type::FilterType, privacy_type::Privacy},
    models::{
        group_role::GroupRole, paged_response_models::PagedResponse,
        permissions_models::PostPermission,
    },
};
use shared::group_model::{Group, GroupFilter, GroupResponse, GroupSort, PostGroup, UpdateGroup};

use super::store::{Store, DATA};

// This method is used to add a group to the canister,
// The method is async because it optionally creates a new canister is created
#[update]
#[candid_method(update)]
async fn add_group(
    post_group: PostGroup,
    member_canister: Principal,
    account_identifier: Option<String>,
) -> Result<GroupResponse, ApiError> {
    Store::add_group(caller(), post_group, member_canister, account_identifier).await
}

// This method is used to get a group from the canister
#[query]
#[candid_method(query)]
fn get_group(identifier: Principal) -> Result<GroupResponse, ApiError> {
    Store::get_group(identifier)
}

// This method is used to get groups filtered and sorted with pagination
#[query]
#[candid_method(query)]
fn get_groups(
    limit: usize,
    page: usize,
    filters: Vec<GroupFilter>,
    filter_type: FilterType,
    sort: GroupSort,
) -> Result<PagedResponse<GroupResponse>, ApiError> {
    Ok(Store::get_groups(limit, page, filters, filter_type, sort))
}

// This method is used to edit a group
#[update]
#[candid_method(update)]
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
#[candid_method(query)]
fn get_chunked_data(
    filters: Vec<GroupFilter>,
    filter_type: FilterType,
    chunk: usize,
    max_bytes_per_chunk: usize,
) -> (Vec<u8>, (usize, usize)) {
    if caller() != DATA.with(|data| data.borrow().parent) {
        return (vec![], (0, 0));
    }

    Store::get_chunked_data(filters, filter_type, chunk, max_bytes_per_chunk)
}

// This method is used to get the owner and privacy of a group
// This is used for inter-canister calls to determine is a user can do a group specific action
#[query]
#[candid_method(query)]
fn get_group_owner_and_privacy(
    group_identifier: Principal,
) -> Result<(Principal, Privacy), ApiError> {
    Store::get_group_owner_and_privacy(group_identifier)
}

// Get multiple groups by their identifiers
#[query]
#[candid_method(query)]
fn get_groups_by_id(group_identifiers: Vec<Principal>) -> Result<Vec<GroupResponse>, ApiError> {
    Ok(Store::get_groups_by_id(group_identifiers))
}

// This method is used to (soft) delete a group
#[update]
#[candid_method(update)]
async fn delete_group(
    group_identifier: Principal,
    member_identifier: Principal,
) -> Result<Group, ApiError> {
    match Store::can_delete(caller(), group_identifier, member_identifier).await {
        Ok(_caller) => Store::delete_group(_caller, group_identifier),
        Err(err) => Err(err),
    }
}

// This method is used to add a custom role to a group
#[update]
#[candid_method(update)]
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
#[candid_method(update)]
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
#[candid_method(query)]
fn get_group_roles(group_identifier: Principal) -> Vec<GroupRole> {
    Store::get_group_roles(group_identifier)
}

// This method is used to update the persmissions of a specific role
#[update]
#[candid_method(update)]
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
#[candid_method(update)]
pub fn update_member_count(
    group_identifier: Principal,
    member_canister: Principal,
    member_count: usize,
) -> Result<(), bool> {
    let _caller = caller();
    if _caller == member_canister {
        return Store::update_member_count(group_identifier, member_canister, member_count);
    }
    return Err(false);
}
