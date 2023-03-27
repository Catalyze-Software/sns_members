use candid::{candid_method, Principal};
use ic_cdk::caller;
use ic_cdk_macros::{query, update};
use ic_scalable_misc::enums::api_error_type::ApiError;

use shared::member_model::{InviteMemberResponse, JoinedMemberResponse, Member};

use super::store::Store;

#[update]
#[candid_method(update)]
async fn join_group(
    group_identifier: Principal,
    account_identifier: Option<String>,
) -> Result<(Principal, Member), ApiError> {
    Store::join_group(caller(), group_identifier, account_identifier).await
}

#[update]
#[candid_method(update)]
async fn create_empty_member(
    caller: Principal,
    profile_identifier: Principal,
) -> Result<Principal, ApiError> {
    Store::create_empty_member(caller, profile_identifier)
}

#[update]
#[candid_method(update)]
async fn invite_to_group(
    member_principal: Principal,
    group_identifier: Principal,
) -> Result<(Principal, Member), ApiError> {
    match Store::can_write_invite(caller(), group_identifier).await {
        Ok(_caller) => Store::invite_to_group(_caller, group_identifier, member_principal),
        Err(err) => Err(err),
    }
}

#[update]
#[candid_method(update)]
async fn accept_user_request_group_invite(
    member_principal: Principal,
    group_identifier: Principal,
) -> Result<(Principal, Member), ApiError> {
    match Store::can_write_invite(caller(), group_identifier).await {
        Ok(_) => Store::accept_user_request_group_invite(member_principal, group_identifier),
        Err(err) => Err(err),
    }
}

#[update]
#[candid_method(update)]
async fn accept_owner_request_group_invite(
    group_identifier: Principal,
) -> Result<(Principal, Member), ApiError> {
    Store::accept_owner_request_group_invite(caller(), group_identifier)
}

#[update]
#[candid_method(update)]
async fn add_owner(
    owner_principal: Principal,
    group_identifier: Principal,
) -> Result<Principal, ApiError> {
    Store::add_owner(owner_principal, group_identifier).await
}

#[update]
#[candid_method(update)]
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

#[update]
#[candid_method(update)]
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
#[candid_method(query)]
fn get_group_member(
    principal: Principal,
    group_identifier: Principal,
) -> Result<JoinedMemberResponse, ApiError> {
    Store::get_group_member_by_user_principal(principal, group_identifier)
}

#[query]
#[candid_method(query)]
fn get_group_members_count(group_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {
    Store::get_group_members_count(group_identifiers)
}

#[query]
#[candid_method(query)]
fn get_groups_for_members(member_identifiers: Vec<Principal>) -> Vec<(Principal, Vec<Principal>)> {
    Store::get_groups_for_members(member_identifiers)
}

#[query]
#[candid_method(query)]
fn get_group_invites_count(group_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {
    Store::get_group_invites_count(group_identifiers)
}

#[query]
#[candid_method(query)]
fn get_group_members(group_identifier: Principal) -> Result<Vec<JoinedMemberResponse>, ApiError> {
    Ok(Store::get_group_members(group_identifier))
}

#[query]
#[candid_method(query)]
fn get_self() -> Result<(Principal, Member), ApiError> {
    Store::get_self(caller())
}

#[query]
#[candid_method(query)]
fn get_member_roles(
    member_identifier: Principal,
    group_identifier: Principal,
) -> Result<(Principal, Vec<String>), String> {
    Store::get_member_roles(member_identifier, group_identifier)
}

#[update]
#[candid_method(update)]
fn leave_group(group_identifier: Principal) -> Result<(), ApiError> {
    Store::leave_group(caller(), group_identifier)
}

#[update]
#[candid_method(update)]
fn remove_invite(group_identifier: Principal) -> Result<(), ApiError> {
    Store::remove_invite(caller(), group_identifier)
}

#[update]
#[candid_method(update)]
async fn remove_member_from_group(
    principal: Principal,
    group_identifier: Principal,
) -> Result<(), ApiError> {
    match Store::can_delete_member(caller(), group_identifier).await {
        Ok(_caller) => Store::remove_join_from_member(_caller, principal, group_identifier),
        Err(err) => Err(err),
    }
}

#[update]
#[candid_method(update)]
async fn remove_member_invite_from_group(
    principal: Principal,
    group_identifier: Principal,
) -> Result<(), ApiError> {
    match Store::can_delete_invite(caller(), group_identifier).await {
        Ok(_caller) => Store::remove_invite_from_member(principal, group_identifier),
        Err(err) => Err(err),
    }
}

#[update]
#[candid_method(update)]
async fn get_group_invites(
    group_identifier: Principal,
) -> Result<Vec<InviteMemberResponse>, ApiError> {
    match Store::can_read_invite(caller(), group_identifier).await {
        Ok(_caller) => Ok(Store::get_group_invites(group_identifier)),
        Err(err) => Err(err),
    }
}
