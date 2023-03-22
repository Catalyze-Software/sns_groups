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

#[update]
#[candid_method(update)]
async fn add_group(
    post_group: PostGroup,
    member_canister: Principal,
) -> Result<GroupResponse, ApiError> {
    Store::add_group(caller(), post_group, member_canister).await
}

#[query]
#[candid_method(query)]
fn get_group(identifier: Principal) -> Result<GroupResponse, ApiError> {
    Store::get_group(identifier)
}

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

#[query]
#[candid_method(query)]
fn get_group_owner_and_privacy(
    group_identifier: Principal,
) -> Result<(Principal, Privacy), ApiError> {
    Store::get_group_owner_and_privacy(group_identifier)
}

#[query]
#[candid_method(query)]
fn get_groups_by_id(group_identifiers: Vec<Principal>) -> Result<Vec<GroupResponse>, ApiError> {
    Ok(Store::get_groups_by_id(group_identifiers))
}

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

#[query]
#[candid_method(query)]
fn get_group_roles(group_identifier: Principal) -> Vec<GroupRole> {
    Store::get_group_roles(group_identifier)
}

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
