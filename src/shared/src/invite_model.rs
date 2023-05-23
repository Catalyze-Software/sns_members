use candid::{CandidType, Deserialize};

#[derive(CandidType, Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Invite {
    pub updated_at: u64,
    pub invite_type: InviteType,
}

impl Default for Invite {
    fn default() -> Self {
        Self {
            updated_at: Default::default(),
            invite_type: Default::default(),
        }
    }
}

#[derive(CandidType, Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum InviteType {
    OwnerRequest,
    UserRequest,
}

impl Default for InviteType {
    fn default() -> Self {
        InviteType::UserRequest
    }
}
