use std::{collections::HashMap, iter::FromIterator};

use candid::Principal;
use ic_cdk::{caller, query, update};
use ic_scalable_misc::enums::api_error_type::ApiError;

use shared::member_model::{InviteMemberResponse, JoinedMemberResponse, Member};

use crate::store::{DATA, ENTRIES};

use super::store::Store;

#[update]
pub fn migration_add_members(members: Vec<(Principal, Member)>) -> () {
    if caller()
        == Principal::from_text("ledm3-52ncq-rffuv-6ed44-hg5uo-iicyu-pwkzj-syfva-heo4k-p7itq-aqe")
            .unwrap()
    {
        DATA.with(|data| {
            data.borrow_mut().current_entry_id = members.clone().len() as u64;
            data.borrow_mut().entries = HashMap::from_iter(members);
        })
    }
}

// This method is used to join an existing group
// The method is async because checks if the group exists and optionally creates a new canister
#[update]
async fn join_group(
    group_identifier: Principal,
    account_identifier: Option<String>,
) -> Result<(Principal, Member), ApiError> {
    Store::join_group(caller(), group_identifier, account_identifier).await
}

// This method is used to create an empty member when a profile is created (inter-canister call)
#[update]
async fn create_empty_member(
    caller: Principal,
    profile_identifier: Principal,
) -> Result<Principal, ApiError> {
    Store::create_empty_member(caller, profile_identifier)
}

// This method is used to invite a user to a group
#[update]
async fn invite_to_group(
    member_principal: Principal,
    group_identifier: Principal,
) -> Result<(Principal, Member), ApiError> {
    match Store::can_write_invite(caller(), group_identifier).await {
        Ok(_caller) => Store::invite_to_group(group_identifier, member_principal),
        Err(err) => Err(err),
    }
}

// This method is used to accept an invite to a group as a admin
#[update]
async fn accept_user_request_group_invite(
    member_principal: Principal,
    group_identifier: Principal,
) -> Result<(Principal, Member), ApiError> {
    match Store::can_write_invite(caller(), group_identifier).await {
        Ok(_) => Store::accept_user_request_group_invite(member_principal, group_identifier),
        Err(err) => Err(err),
    }
}

// This method is used to accept an invite to a group as a user
#[update]
async fn accept_owner_request_group_invite(
    group_identifier: Principal,
) -> Result<(Principal, Member), ApiError> {
    Store::accept_owner_request_group_invite(caller(), group_identifier)
}

// This method is used a to add an owner to the member entry when a group is created (inter-canister call)
#[update]
async fn add_owner(
    owner_principal: Principal,
    group_identifier: Principal,
) -> Result<Principal, ApiError> {
    Store::add_owner(owner_principal, group_identifier).await
}

// Method to assign a role to a specific group member
#[update]
async fn assign_role(
    role: String,
    member_identifier: Principal,
    group_identifier: Principal,
) -> Result<(), ()> {
    match Store::can_write_member(caller(), group_identifier).await {
        Ok(_) => Store::assign_role(role, member_identifier, group_identifier),
        Err(_) => Err(()),
    }
}

// Method to remove a role from a specific group member
#[update]
async fn remove_role(
    role: String,
    member_identifier: Principal,
    group_identifier: Principal,
) -> Result<(), ()> {
    match Store::can_write_member(caller(), group_identifier).await {
        Ok(_) => Store::remove_role(role, member_identifier, group_identifier),
        Err(_) => Err(()),
    }
}

#[query]
fn get_member(member_identifier: Principal) -> Option<Member> {
    ENTRIES.with(|entries| entries.borrow().get(&member_identifier.to_string()))
}

// Method to assign a role to a specific group member
#[update]
async fn set_roles(
    roles: Vec<String>,
    member_identifier: Principal,
    group_identifier: Principal,
) -> Result<(), ()> {
    match Store::can_write_member(caller(), group_identifier).await {
        Ok(_) => Store::set_roles(roles, member_identifier, group_identifier),
        Err(_) => Err(()),
    }
}

// Method to fetch a specific group member by user principal
#[query]
fn get_group_member(
    principal: Principal,
    group_identifier: Principal,
) -> Result<JoinedMemberResponse, ApiError> {
    Store::get_group_member_by_user_principal(principal, group_identifier)
}

// Method to get the amount of members of specific groups
#[query]
fn get_group_members_count(group_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {
    Store::get_group_members_count(group_identifiers)
}

// Method to get the groups specific members are member of
#[query]
fn get_groups_for_members(member_identifiers: Vec<Principal>) -> Vec<(Principal, Vec<Principal>)> {
    Store::get_groups_for_members(member_identifiers)
}

// Method to get the amount of invites of specific groups
#[query]
fn get_group_invites_count(group_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {
    Store::get_group_invites_count(group_identifiers)
}

// Method to get all members of a specific group
#[query]
fn get_group_members(group_identifier: Principal) -> Result<Vec<JoinedMemberResponse>, ApiError> {
    Ok(Store::get_group_members(group_identifier))
}

// Method to get the caller member entry
#[query]
fn get_self() -> Result<(Principal, Member), ApiError> {
    Store::get_self(caller())
}

// Get the roles of a specific member within a specific group
#[query]
fn get_member_roles(
    member_identifier: Principal,
    group_identifier: Principal,
) -> Result<(Principal, Vec<String>), String> {
    Store::get_member_roles(member_identifier, group_identifier)
}

// Method to let the caller leave a group
#[update]
fn leave_group(group_identifier: Principal) -> Result<(), ApiError> {
    Store::leave_group(caller(), group_identifier)
}

// Method to remove an outstanding invite for a group as a user
#[update]
fn remove_invite(group_identifier: Principal) -> Result<(), ApiError> {
    Store::remove_invite(caller(), group_identifier)
}

// Method to remove a member from a group
#[update]
async fn remove_member_from_group(
    principal: Principal,
    group_identifier: Principal,
) -> Result<(), ApiError> {
    match Store::can_delete_member(caller(), group_identifier).await {
        Ok(_caller) => Store::remove_join_from_member(_caller, principal, group_identifier),
        Err(err) => Err(err),
    }
}

// Method to remove an outstanding invite for a group as a admin
#[update]
async fn remove_member_invite_from_group(
    principal: Principal,
    group_identifier: Principal,
) -> Result<(), ApiError> {
    match Store::can_delete_invite(caller(), group_identifier).await {
        Ok(_caller) => Store::remove_invite_from_member(principal, group_identifier),
        Err(err) => Err(err),
    }
}

// Method to get all group invites
#[update]
async fn get_group_invites(
    group_identifier: Principal,
) -> Result<Vec<InviteMemberResponse>, ApiError> {
    match Store::can_read_invite(caller(), group_identifier).await {
        Ok(_caller) => Ok(Store::get_group_invites(group_identifier)),
        Err(err) => Err(err),
    }
}

// COMPOSITE_QUERY PREPARATION
// This methods is used by the parent canister to get members the (this) child canister
// Data serialized and send as byte array chunks ` (bytes, (start_chunk, end_chunk)) `
// The parent canister can then deserialize the data and pass it to the frontend
#[query]
fn get_chunked_join_data(
    group_identifier: Principal,
    chunk: usize,
    max_bytes_per_chunk: usize,
) -> (Vec<u8>, (usize, usize)) {
    if caller() != DATA.with(|data| data.borrow().parent) {
        return (vec![], (0, 0));
    }

    Store::get_chunked_join_data(&group_identifier, chunk, max_bytes_per_chunk)
}

// COMPOSITE_QUERY PREPARATION
// This methods is used by the parent canister to get members the (this) child canister
// Data serialized and send as byte array chunks ` (bytes, (start_chunk, end_chunk)) `
// The parent canister can then deserialize the data and pass it to the frontend
#[query]
fn get_chunked_invite_data(
    group_identifier: Principal,
    chunk: usize,
    max_bytes_per_chunk: usize,
) -> (Vec<u8>, (usize, usize)) {
    if caller() != DATA.with(|data| data.borrow().parent) {
        return (vec![], (0, 0));
    }

    Store::get_chunked_invite_data(&group_identifier, chunk, max_bytes_per_chunk)
}
