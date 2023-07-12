use std::collections::HashMap;

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

pub type GroupIdentifier = Principal;
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Member {
    pub principal: Principal,
    pub profile_identifier: Principal,
    pub joined: HashMap<GroupIdentifier, Join>,
    pub invites: HashMap<GroupIdentifier, Invite>,
}

impl Default for Member {
    fn default() -> Self {
        Self {
            principal: Principal::anonymous(),
            profile_identifier: Principal::anonymous(),
            joined: Default::default(),
            invites: Default::default(),
        }
    }
}

#[derive(CandidType, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Join {
    pub roles: Vec<String>,
    pub updated_at: u64,
    pub created_at: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Invite {
    pub invite_type: InviteType,
    pub updated_at: u64,
    pub created_at: u64,
}

#[derive(CandidType, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InviteType {
    OwnerRequest,
    UserRequest,
}

impl Default for InviteType {
    fn default() -> Self {
        InviteType::UserRequest
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct JoinedMemberResponse {
    pub group_identifier: Principal,
    pub member_identifier: Principal,
    pub principal: Principal,
    pub roles: Vec<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct InviteMemberResponse {
    pub group_identifier: Principal,
    pub member_identifier: Principal,
    pub principal: Principal,
    pub invite: Invite,
}
