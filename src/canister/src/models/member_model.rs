use candid::Principal;
use ic_cdk::export::candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Member {
    pub principal: Principal,
    pub profile_identifier: Principal,
    pub joined: Vec<Join>,
    pub invites: Vec<Invite>,
}

impl Default for Member {
    fn default() -> Self {
        Self {
            principal: Principal::anonymous(),
            profile_identifier: Principal::anonymous(),
            joined: Vec::default(),
            invites: Vec::default(),
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Join {
    pub group_identifier: Principal,
    pub roles: Vec<String>,
    pub updated_at: u64,
    pub created_at: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Invite {
    pub group_identifier: Principal,
    pub invite_type: InviteType,
    pub updated_at: u64,
    pub created_at: u64,
}

#[derive(CandidType, Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
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
