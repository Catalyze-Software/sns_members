use std::{cell::RefCell, collections::HashMap, vec};

use candid::Principal;
use ic_cdk::{
    api::{call, time},
    id,
};
use ic_scalable_canister::store::Data;
use ic_scalable_misc::{
    enums::{
        api_error_type::{ApiError, ApiErrorType},
        privacy_type::{GatedType, NeuronGatedRules, Privacy, TokenGated},
    },
    helpers::{
        error_helper::api_error,
        role_helper::{default_roles, get_group_roles, has_permission},
        serialize_helper::serialize,
        token_canister_helper::{
            dip20_balance_of, dip721_balance_of, ext_balance_of, legacy_dip721_balance_of,
        },
    },
    models::{
        identifier_model::Identifier,
        neuron_models::{DissolveState, ListNeurons, ListNeuronsResponse, NeuronId},
        permissions_models::{PermissionActionType, PermissionType},
    },
};

use shared::member_model::{
    Invite, InviteMemberResponse, InviteType, Join, JoinedMemberResponse, Member,
};

thread_local! {
    pub static DATA: RefCell<Data<Member>>  = RefCell::new(Data::default());
}

pub struct Store;

impl Store {
    // Method used to join a group
    pub async fn join_group(
        caller: Principal,
        group_identifier: Principal,
        account_identifier: Option<String>,
    ) -> Result<(Principal, Member), ApiError> {
        // Get the group owner and privacy from an inter-canister call
        let group_owner_and_privacy =
            Self::get_group_owner_and_privacy(group_identifier.clone()).await;

        match group_owner_and_privacy {
            // if the call fails return an error
            Err(err) => Err(err),
            Ok((_group_owner, _group_privacy)) => {
                let existing_member = Self::_get_member_from_caller(caller);

                match existing_member.clone() {
                    // If there is no exisiting member
                    None => {}
                    Some((_identifier, _exisiting_member)) => {
                        if _exisiting_member.principal != caller {
                            return Err(api_error(
                                ApiErrorType::BadRequest,
                                "UNAUTHORIZED",
                                "You are not authorized to perform this action",
                                DATA.with(|data| Data::get_name(data)).as_str(),
                                "join_group",
                                None,
                            ));
                        }
                        // if the group identifier is already found in the joined array, throw an error
                        if _exisiting_member
                            .joined
                            .iter()
                            .any(|m| &m.group_identifier == &group_identifier)
                        {
                            return Err(api_error(
                                ApiErrorType::BadRequest,
                                "ALREADY_JOINED",
                                "You are already part of this group",
                                DATA.with(|data| Data::get_name(data)).as_str(),
                                "join_group",
                                None,
                            ));
                        }
                        // if the group identifier is already found in the invites array, throw an error
                        if _exisiting_member
                            .invites
                            .iter()
                            .any(|m| &m.group_identifier == &group_identifier)
                        {
                            return Err(api_error(
                                ApiErrorType::BadRequest,
                                "PENDING_INVITE",
                                "There is already a pending invite for this group",
                                DATA.with(|data| Data::get_name(data)).as_str(),
                                "join_group",
                                None,
                            ));
                        }
                    }
                };

                // get the updated member
                let updated_member = Self::add_invite_or_join_group_to_member(
                    caller,
                    group_identifier,
                    existing_member.clone(),
                    _group_privacy,
                    account_identifier,
                )
                .await;

                // update the member
                match updated_member {
                    // if the call fails return an error
                    Err(err) => Err(err),
                    // if the call succeeds, continue
                    Ok(_updated_member) => match existing_member {
                        None => {
                            // if there is no existing member, add a new one
                            let result = DATA.with(|data| {
                                Data::add_entry(
                                    data,
                                    _updated_member.clone(),
                                    Some("mbr".to_string()),
                                )
                            });
                            // fire and forget inter canister call to update the group member count on the group canister
                            Self::update_member_count_on_group(&group_identifier);
                            match result {
                                // The group was not added to the data store because the canister is at capacity
                                Err(err) => match err {
                                    ApiError::CanisterAtCapacity(message) => {
                                        let _data = DATA.with(|v| v.borrow().clone());
                                        // Spawn a sibling canister and pass the group data to it
                                        // TODO: work on logic to split member data
                                        match Data::spawn_sibling(_data, _updated_member).await {
                                            Ok(_) => Err(ApiError::CanisterAtCapacity(message)),
                                            Err(err) => Err(err),
                                        }
                                    }
                                    _ => Err(err),
                                },
                                Ok((_identifier, _member_data)) => Ok((_identifier, _member_data)),
                            }
                        }
                        // if there is an existing member, update the existing one
                        Some((_identifier, _)) => {
                            // update the member
                            let result = DATA.with(|data| {
                                Data::update_entry(data, _identifier, _updated_member)
                            });
                            // fire and forget inter canister call to update the group member count on the group canister
                            Self::update_member_count_on_group(&group_identifier);
                            result
                        }
                    },
                }
            }
        }
    }

    // Method to create an empty member
    pub fn create_empty_member(
        caller: Principal,
        profile_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        // Decode the profile identifier
        let (_, _, kind) = Identifier::decode(&profile_identifier);

        // If the kind is not pfe, throw an error
        if kind != "pfe".to_string() {
            return Err(api_error(
                ApiErrorType::NotFound,
                "INVALID TYPE",
                format!("'{}' is not supported", kind).as_str(),
                DATA.with(|data| Data::get_name(data)).as_str(),
                "create_empty_member",
                None,
            ));
        } else {
            // If the kind is pfe, continue
            match Self::_get_member_from_caller(caller) {
                None => {
                    // If there is no existing member, create a new one
                    let empty_member = Member {
                        principal: caller,
                        profile_identifier,
                        joined: vec![],
                        invites: vec![],
                    };
                    // Add the new member
                    let result = DATA
                        .with(|data| Data::add_entry(data, empty_member, Some("mbr".to_string())));
                    match result {
                        Ok((_identfier, _)) => Ok(_identfier),
                        Err(err) => Err(err),
                    }
                }
                // If there is an existing member, throw an error
                Some(_) => Err(api_error(
                    ApiErrorType::BadRequest,
                    "ALREADY_MEMBER",
                    "You already have an entry",
                    DATA.with(|data| Data::get_name(data)).as_str(),
                    "create_empty_member",
                    None,
                )),
            }
        }
    }

    // Method to leave a group
    pub fn leave_group(caller: Principal, group_identifier: Principal) -> Result<(), ApiError> {
        // Get the existing member
        let existing_member = Self::_get_member_from_caller(caller);

        match existing_member {
            // If there is no existing member, throw an error
            None => Err(Self::_member_not_found_error("leave_group", None)),

            // If there is an existing member, continue
            Some((_identifier, mut _member)) => {
                // Filter out the group identifier from the joined array
                let joined: Vec<Join> = _member
                    .joined
                    .iter()
                    .filter(|j| &j.group_identifier != &group_identifier)
                    .cloned()
                    .collect();

                // Update the joined array
                _member.joined = joined;
                // Update the member
                let _ = DATA.with(|data| Data::update_entry(data, _identifier, _member));
                Ok(Self::update_member_count_on_group(&group_identifier))
            }
        }
    }

    // Method to remove an invite from a member
    pub fn remove_invite(caller: Principal, group_identifier: Principal) -> Result<(), ApiError> {
        // Get the existing member
        let existing_member = Self::_get_member_from_caller(caller);
        match existing_member {
            // If there is no existing member, throw an error
            None => Err(Self::_member_not_found_error("leave_group", None)),
            // If there is an existing member, continue
            Some((_identifier, mut _member)) => {
                // Filter out the group identifier from the invites array
                let invites: Vec<Invite> = _member
                    .invites
                    .into_iter()
                    .filter(|j| &j.group_identifier != &group_identifier)
                    .collect();

                // Update the invites array
                _member.invites = invites;
                // Update the member
                let _ = DATA.with(|data| Data::update_entry(data, _identifier, _member));
                Ok(())
            }
        }
    }

    // Method to assign a group role to a member
    pub fn assign_role(
        role: String,
        member_identifier: Principal,
        group_identifier: Principal,
    ) -> Result<(), ()> {
        // Get the existing member
        let member = DATA.with(|data| Data::get_entry(data, member_identifier));

        if let Ok((_identifier, mut _member)) = member {
            // Get the existing roles for the group
            let existing_roles = _member
                .joined
                .iter()
                .filter(|j| &j.group_identifier == &group_identifier)
                .map(|j| j.roles.clone())
                .flatten()
                .collect::<Vec<String>>();

            if existing_roles.contains(&role) {
                return Err(());
            }

            // Update the joined array
            let joined: Vec<Join> = _member
                .joined
                .into_iter()
                .map(|j| {
                    if j.group_identifier == group_identifier {
                        let mut roles = j.roles;
                        roles.push(role.clone());
                        Join {
                            group_identifier: j.group_identifier,
                            roles,
                            updated_at: time(),
                            created_at: time(),
                        }
                    } else {
                        j
                    }
                })
                .collect();

            // Update the member
            _member.joined = joined;
            let _ = DATA.with(|data| Data::update_entry(data, _identifier, _member));
            Ok(())
        } else {
            Err(())
        }
    }

    // method to remove a role from a member
    pub fn remove_role(
        role: String,
        member_identifier: Principal,
        group_identifier: Principal,
    ) -> Result<(), ()> {
        // Get the existing member
        let member = DATA.with(|data| Data::get_entry(data, member_identifier));
        if let Ok((_identifier, mut _member)) = member {
            // Update the joined array
            let joined: Vec<Join> = _member
                .joined
                .into_iter()
                .map(|j| {
                    // If the group identifier matches, remove the role
                    if j.group_identifier == group_identifier {
                        let roles: Vec<String> =
                            j.roles.into_iter().filter(|r| r != &role).collect();
                        Join {
                            group_identifier: j.group_identifier,
                            roles,
                            updated_at: time(),
                            created_at: time(),
                        }
                        // If the group identifier does not match, return the original join
                    } else {
                        j
                    }
                })
                .collect();

            // Update the member
            _member.joined = joined;
            let _ = DATA.with(|data| Data::update_entry(data, _identifier, _member));
            Ok(())
        } else {
            Err(())
        }
    }

    // Method to remove a join from a member
    pub fn remove_join_from_member(
        caller: Principal,
        member_principal: Principal,
        group_identifier: Principal,
    ) -> Result<(), ApiError> {
        // Get the existing member
        if let Some((_, _member)) = Self::_get_member_from_caller(caller) {
            if _member.joined.iter().any(|j| {
                // Check if the member is an owner of the group
                j.group_identifier == group_identifier && j.roles.contains(&"owner".to_string())
            }) {
                // Get the member to remove the join from
                match Self::_get_member_from_caller(member_principal) {
                    None => Err(Self::_member_not_found_error(
                        "remove_join_from_member",
                        None,
                    )),
                    Some((_identifier, mut _member)) => {
                        // Filter out the group identifier from the joined array
                        let joined: Vec<Join> = _member
                            .joined
                            .into_iter()
                            .filter(|j| &j.group_identifier != &group_identifier)
                            .collect();

                        // Update the joined array
                        _member.joined = joined;
                        let _ = DATA.with(|data| Data::update_entry(data, _identifier, _member));
                        return Ok(Self::update_member_count_on_group(&group_identifier));
                    }
                }
                // If the member is not an owner, throw an error
            } else {
                return Err(api_error(
                    ApiErrorType::BadRequest,
                    "UNAUTHORIZED",
                    "You are not authorized to perform this action",
                    DATA.with(|data| Data::get_name(data)).as_str(),
                    "join_group",
                    None,
                ));
            }
            // If the member does not exist, throw an error
        } else {
            Err(Self::_member_not_found_error(
                "remove_join_from_member",
                None,
            ))
        }
    }

    // Method to remove an invite from a member
    pub fn remove_invite_from_member(
        caller: Principal,
        group_identifier: Principal,
    ) -> Result<(), ApiError> {
        // Get the existing member
        match Self::_get_member_from_caller(caller) {
            // If the member does not exist, throw an error
            None => Err(Self::_member_not_found_error(
                "remove_invite_from_member",
                None,
            )),
            // If the member exists, remove the invite
            Some((_identifier, mut _member)) => {
                // Filter out the group identifier from the invites array
                let invites: Vec<Invite> = _member
                    .invites
                    .into_iter()
                    .filter(|j| &j.group_identifier != &group_identifier)
                    .collect();
                // Update the invites array
                _member.invites = invites;
                let _ = DATA.with(|data| Data::update_entry(data, _identifier, _member));
                Ok(())
            }
        }
    }

    // Method to add a join or invite to a member
    async fn add_invite_or_join_group_to_member(
        caller: Principal,
        group_identifier: Principal,
        member: Option<(Principal, Member)>,
        group_privacy: Privacy,
        // is used for NFT gated groups
        account_identifier: Option<String>,
    ) -> Result<Member, ApiError> {
        // Create a join entry based on the group privacy settings and set the default role
        let join = Join {
            group_identifier: group_identifier.clone(),
            roles: vec!["member".to_string()],
            updated_at: time(),
            created_at: time(),
        };

        // Create an invite entry based on the group privacy settings
        let invite = Invite {
            group_identifier,
            invite_type: InviteType::UserRequest,
            updated_at: time(),
            created_at: time(),
        };

        use Privacy::*;
        match group_privacy {
            // If the group is public, add the member to the group
            Public => match member {
                // If the member does not exist, create a new member
                None => Ok(Member {
                    principal: caller,
                    profile_identifier: Principal::anonymous(),
                    joined: vec![join],
                    invites: vec![],
                }),
                // If the member exists, add the join to the member
                Some((_, mut _member)) => {
                    _member.joined.push(join);
                    Ok(_member)
                }
            },
            // If the group is private, add the invite to the member
            Private => match member {
                // If the member does not exist, create a new member
                None => Ok(Member {
                    principal: caller,
                    profile_identifier: Principal::anonymous(),
                    joined: vec![],
                    invites: vec![invite],
                }),
                // If the member exists, add the invite to the member
                Some((_, mut _member)) => {
                    _member.invites.push(invite);
                    Ok(_member)
                }
            },
            // If the group is invite only, throw an error
            InviteOnly => {
                return Err(api_error(
                    ApiErrorType::BadRequest,
                    "UNSUPPORTED",
                    "This type of invite isnt supported through this call",
                    DATA.with(|data| Data::get_name(data)).as_str(),
                    "add_invite_or_join_group_to_member",
                    None,
                ))
            }
            // Self::validate_neuron(caller, neuron_canister.governance_canister, neuron_canister.rules).await
            // If the group is gated, check if the caller owns a specific NFT
            Gated(gated_type) => {
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
                            match member {
                                None => Ok(Member {
                                    principal: caller,
                                    profile_identifier: Principal::anonymous(),
                                    joined: vec![join],
                                    invites: vec![],
                                }),
                                Some((_, mut _member)) => {
                                    _member.joined.push(join);
                                    Ok(_member)
                                }
                            }
                            // If the caller does not own the neuron, throw an error
                        } else {
                            return Err(api_error(
                                ApiErrorType::Unauthorized,
                                "NOT_OWNING_NEURON",
                                "You are not owning this neuron required to join this group",
                                DATA.with(|data| Data::get_name(data)).as_str(),
                                "add_invite_or_join_group_to_member",
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
                            match member {
                                None => Ok(Member {
                                    principal: caller,
                                    profile_identifier: Principal::anonymous(),
                                    joined: vec![join],
                                    invites: vec![],
                                }),
                                Some((_, mut _member)) => {
                                    _member.joined.push(join);
                                    Ok(_member)
                                }
                            }
                            // If the caller does not own the NFT, throw an error
                        } else {
                            return Err(api_error(
                                ApiErrorType::Unauthorized,
                                "NOT_OWNING_NFT",
                                "You are not owning NFT / token required to join this group",
                                DATA.with(|data| Data::get_name(data)).as_str(),
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

    // Method to get the member entry from the caller
    pub fn get_self(caller: Principal) -> Result<(Principal, Member), ApiError> {
        // Get the member entry from the caller
        let existing = Self::_get_member_from_caller(caller);
        match existing {
            None => Err(Self::_member_not_found_error("get_self", None)),
            Some(_member) => Ok(_member),
        }
    }

    // Method to get the roles assigned to the member in a specific group
    pub fn get_member_roles(
        member_identifier: Principal,
        group_identifier: Principal,
    ) -> Result<(Principal, Vec<String>), String> {
        // Get the member entry from the member identifier
        let member = DATA.with(|data| Data::get_entry(data, member_identifier));

        match member {
            // If the member exists, return the roles
            Ok((_member_identifier, _member)) => {
                if let Some(_join) = _member
                    .joined
                    .iter()
                    .find(|j| j.group_identifier == group_identifier)
                {
                    Ok((_member.principal, _join.roles.clone()))
                // If the member does not exist in the group, return an empty array
                } else {
                    Ok((_member.principal, vec![]))
                }
            }
            // If the member does not exist, throw an error
            Err(_) => Err("No member found".to_string()),
        }
    }

    // Method to get the roles assigned to the caller principal in a specific group
    pub fn get_member_roles_by_principal(
        principal: Principal,
        group_identifier: Principal,
    ) -> Result<(Principal, Vec<String>), String> {
        // Get the member entry from the caller principal
        let member = Self::_get_member_from_caller(principal);

        match member {
            // If the member exists, return the roles
            Some((_member_identifier, _member)) => {
                if let Some(_join) = _member
                    .joined
                    .iter()
                    .find(|j| j.group_identifier == group_identifier)
                {
                    Ok((_member.principal, _join.roles.clone()))
                // If the member does not exist in the group, return an empty array
                } else {
                    Ok((_member.principal, vec![]))
                }
            }
            // If the member does not exist, throw an error
            None => Err("No member found".to_string()),
        }
    }

    pub fn get_group_member_by_user_principal(
        caller: Principal,
        group_identifier: Principal,
    ) -> Result<JoinedMemberResponse, ApiError> {
        DATA.with(|data| {
            // Get the member entry from the caller principal
            let member = Store::_get_member_from_caller(caller);
            match member {
                // If the member does not exist, throw an error
                None => Err(Self::_member_not_found_error(
                    "get_group_member_by_user_principal",
                    None,
                )),
                // If the member exists, continue
                Some((_identifier, _member)) => {
                    let join = _member
                        .joined
                        .iter()
                        .find(|j| &j.group_identifier == &group_identifier);

                    match join {
                        // If the member does not exist in the group, return an error
                        None => Err(api_error(
                            ApiErrorType::NotFound,
                            "NOT_JOINED",
                            "You have no roles within this group",
                            Data::get_name(data).as_str(),
                            "get_group_member_by_user_principal",
                            None,
                        )),
                        // If the member exists in the group, return the joined member response
                        Some(_join) => Ok(JoinedMemberResponse {
                            group_identifier: group_identifier,
                            member_identifier: _identifier,
                            principal: caller,
                            roles: _join.roles.clone(),
                        }),
                    }
                }
            }
        })
    }

    // Method to get the members of the group
    pub fn get_group_members(group_identifier: Principal) -> Vec<JoinedMemberResponse> {
        // Get all members
        DATA.with(|data| {
            let members = Data::get_entries(data);
            // Filter the members that are in the group
            members
                .iter()
                .filter(|(_identifier, _member)| {
                    _member
                        .joined
                        .iter()
                        .any(|j| &j.group_identifier == &group_identifier)
                })
                .map(|(_identifier, _member)| {
                    Self::map_member_to_joined_member_response(
                        _identifier,
                        _member,
                        group_identifier.clone(),
                    )
                })
                .collect()
        })
    }

    // Method to get the total member in a specific range of groups
    pub fn get_group_members_count(group_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {
        // Initialize the members count array
        let mut members_counts: Vec<(Principal, usize)> = vec![];

        DATA.with(|data| {
            // Get all members
            let members = Data::get_entries(data);

            // For each group, count the members that are in the group
            for group_identifier in group_identifiers {
                let count = members
                    .iter()
                    .filter(|(_identifier, member)| {
                        member
                            .joined
                            .iter()
                            .any(|j| &j.group_identifier == &group_identifier)
                    })
                    .count();
                // Push the group identifier and the count to the members count array
                members_counts.push((group_identifier, count));
            }
        });

        members_counts
    }

    // Method to get the groups that the member is in
    pub fn get_groups_for_members(
        member_identifier: Vec<Principal>,
    ) -> Vec<(Principal, Vec<Principal>)> {
        // Initialize an empty members with groups array
        let mut members_with_groups: Vec<(Principal, Vec<Principal>)> = vec![];

        // For each member, get the groups that the member is in
        for _member_identifier in member_identifier {
            // Initialize an empty groups array
            let mut groups: Vec<Principal> = vec![];
            // Get the member entry
            let member = DATA.with(|data| Data::get_entry(data, _member_identifier));

            // If the member exists, get the groups that the member is in
            if let Ok((_, _member)) = member {
                for joined in _member.joined.iter() {
                    // Push the group identifier to the groups array
                    groups.push(joined.group_identifier.clone());
                }
            }
            // Push the member identifier and the groups array to the members with groups array
            members_with_groups.push((_member_identifier, groups));
        }

        members_with_groups
    }

    // Method to get the total invites in a specific range of groups
    pub fn get_group_invites_count(group_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {
        // Initialize the invite count array
        let mut members_counts: Vec<(Principal, usize)> = vec![];

        DATA.with(|data| {
            // Get all members
            let members = Data::get_entries(data);

            // For each group, count the invites that are in the group
            for group_identifier in group_identifiers {
                let count = members
                    .iter()
                    .filter(|(_identifier, member)| {
                        member
                            .invites
                            .iter()
                            .any(|j| &j.group_identifier == &group_identifier)
                    })
                    .count();
                // Push the group identifier and the count to the invite count array
                members_counts.push((group_identifier, count));
            }
        });

        members_counts
    }

    // Method to get the invites of the group
    pub fn get_group_invites(group_identifier: Principal) -> Vec<InviteMemberResponse> {
        DATA.with(|data| {
            // Get all members
            let members = Data::get_entries(data);

            // Filter the members invites that are in the group
            members
                .iter()
                .filter(|(_identifier, _member)| {
                    _member
                        .invites
                        .iter()
                        .any(|j| &j.group_identifier == &group_identifier)
                })
                .map(|(_identifier, _member)| {
                    Self::map_member_to_invite_member_response(
                        _identifier,
                        _member,
                        group_identifier.clone(),
                    )
                })
                .collect()
        })
    }

    // Method that is called when a group is created
    pub async fn add_owner(
        owner_principal: Principal,
        group_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        // Get the group owner and privacy from an inter-canister call
        let group_owner_and_privacy =
            Self::get_group_owner_and_privacy(group_identifier.clone()).await;

        DATA.with(|data| {
            match group_owner_and_privacy {
                // if the call fails return an error
                Err(err) => Err(err),
                Ok((_group_owner, _group_privacy)) => {
                    // Check if the caller is the owner of the group
                    if _group_owner != owner_principal {
                        return Err(api_error(
                            ApiErrorType::BadRequest,
                            "CANT_SET_OWNER",
                            "You are not the owner of this group",
                            Data::get_name(data).as_str(),
                            "add_owner",
                            None,
                        ));
                    }
                    // iterate over the members and get the existing member
                    let existing_member = Self::_get_member_from_caller(owner_principal);

                    match existing_member {
                        // If there is no exisiting member, do nothing
                        None => {
                            let new_member = Member {
                                principal: owner_principal,
                                profile_identifier: Principal::anonymous(),
                                joined: vec![Join {
                                    group_identifier,
                                    roles: vec!["owner".to_string()],
                                    updated_at: time(),
                                    created_at: time(),
                                }],
                                invites: vec![],
                            };

                            let response =
                                Data::add_entry(data, new_member, Some("mbr".to_string()));
                            match response {
                                Err(err) => Err(err),
                                Ok((_identifier, _member)) => Ok(_identifier),
                            }
                        }
                        Some((_identifier, mut _member)) => {
                            // if the group identifier is already found in the joined array, throw an error
                            if _member
                                .joined
                                .iter()
                                .any(|m| &m.group_identifier == &group_identifier)
                            {
                                return Err(api_error(
                                    ApiErrorType::BadRequest,
                                    "ALREADY_JOINED",
                                    "You are already part of this group",
                                    Data::get_name(data).as_str(),
                                    "add_owner",
                                    None,
                                ));
                            }

                            // Add the group identifier to the joined array
                            _member.joined.push(Join {
                                group_identifier,
                                roles: vec!["owner".to_string()],
                                updated_at: time(),
                                created_at: time(),
                            });

                            let response = Data::update_entry(data, _identifier, _member);
                            match response {
                                Err(err) => Err(err),
                                Ok((_identifier, _member)) => Ok(_identifier),
                            }
                        }
                    }
                } // 1710994309 / 100 = 17109943.09
            }
        })
    }

    // Method to invite a member to a group
    pub fn invite_to_group(
        group_identifier: Principal,
        member_principal: Principal,
    ) -> Result<(Principal, Member), ApiError> {
        DATA.with(|data| {
            // Get the existing member
            let existing_member = Self::_get_member_from_caller(member_principal);

            // Create the invite
            let invite = Invite {
                group_identifier,
                // Set the invite type to owner request
                invite_type: InviteType::OwnerRequest,
                updated_at: time(),
                created_at: time(),
            };

            match existing_member {
                None => {
                    // If there is no existing member, create a new member
                    let member = Member {
                        principal: member_principal,
                        profile_identifier: Principal::anonymous(),
                        joined: vec![],
                        invites: vec![invite],
                    };
                    // Add the member to the members array
                    Data::add_entry(data, member, Some("mbr".to_string()))
                }
                Some((_identifier, mut _member)) => {
                    // If there is an existing member, add the invite to the invites array
                    _member.invites.push(invite);
                    // Update the member
                    Data::update_entry(data, _identifier, _member)
                }
            }
        })
    }

    // Method to accept a user request to join a group
    pub fn accept_user_request_group_invite(
        member_principal: Principal,
        group_identifier: Principal,
    ) -> Result<(Principal, Member), ApiError> {
        // Get the existing member
        match Self::_get_member_from_caller(member_principal) {
            // If there is no member, throw an error
            None => Err(Self::_member_not_found_error(
                "remove_invite_from_member",
                None,
            )),
            // If there is a member, continue
            Some((_identifier, mut _member)) => {
                // Find the invite in the invites array
                let invite = _member
                    .invites
                    .iter()
                    .find(|i| &i.group_identifier == &group_identifier);

                match invite {
                    // If there is no invite, throw an error
                    None => Err(api_error(
                        ApiErrorType::NotFound,
                        "NO_INVITE_FOUND",
                        "There is no invite found for this group",
                        DATA.with(|data| Data::get_name(data)).as_str(),
                        "accept_user_request_group_invite",
                        None,
                    )),
                    // If there is an invite, continue
                    Some(_invite) => {
                        // Check if the invite type is user request
                        if _invite.invite_type != InviteType::UserRequest {
                            return Err(api_error(
                                ApiErrorType::BadRequest,
                                "INVALID_TYPE",
                                "Invalid invite type",
                                DATA.with(|data| Data::get_name(data)).as_str(),
                                "accept_user_request_group_invite",
                                None,
                            ));
                        }

                        // Remove the invite from the invites array
                        _member.invites = _member
                            .invites
                            .into_iter()
                            .filter(|i| &i.group_identifier != &group_identifier)
                            .collect();

                        // Add a new Join to the joined array
                        _member.joined.push(Join {
                            group_identifier,
                            roles: vec!["member".to_string()],
                            updated_at: time(),
                            created_at: time(),
                        });

                        // Update the member
                        let result =
                            DATA.with(|data| Data::update_entry(data, _identifier, _member));

                        // Update the member count on the group canister (inter-canister call)
                        Self::update_member_count_on_group(&group_identifier);
                        result
                    }
                }
            }
        }
    }

    // Method to accept an owner request to join a group
    pub fn accept_owner_request_group_invite(
        caller: Principal,
        group_identifier: Principal,
    ) -> Result<(Principal, Member), ApiError> {
        DATA.with(|data| {
            // Get the existing member
            match Self::_get_member_from_caller(caller) {
                // If there is no member, throw an error
                None => Err(Self::_member_not_found_error(
                    "remove_invite_from_member",
                    None,
                )),
                // If there is a member, continue
                Some((_identifier, mut _member)) => {
                    // Find the invite in the invites array
                    let invite = _member
                        .invites
                        .iter()
                        .find(|i| &i.group_identifier == &group_identifier);

                    match invite {
                        // If there is no invite, throw an error
                        None => Err(api_error(
                            ApiErrorType::NotFound,
                            "NO_INVITE_FOUND",
                            "There is no invite found for this group",
                            Data::get_name(data).as_str(),
                            "accept_owner_request_group_invite",
                            None,
                        )),
                        // If there is an invite, continue
                        Some(_invite) => {
                            // Check if the invite type is owner request
                            if _invite.invite_type != InviteType::OwnerRequest {
                                return Err(api_error(
                                    ApiErrorType::BadRequest,
                                    "INVALID_TYPE",
                                    "Invalid invite type",
                                    Data::get_name(data).as_str(),
                                    "accept_owner_request_group_invite",
                                    None,
                                ));
                            }

                            // Remove the invite from the invites array
                            _member.invites = _member
                                .invites
                                .iter()
                                .filter(|i| &i.group_identifier == &group_identifier)
                                .cloned()
                                .collect();

                            // Add a new Join to the joined array
                            _member.joined.push(Join {
                                group_identifier,
                                roles: vec!["member".to_string()],
                                updated_at: time(),
                                created_at: time(),
                            });
                            // Update the member
                            let result = Data::update_entry(data, _identifier, _member);

                            // Update the member count on the group canister (inter-canister call)
                            Self::update_member_count_on_group(&group_identifier);
                            result
                        }
                    }
                }
            }
        })
    }

    // Method to get the group owner and privacy from the group canister (inter-canister call)
    async fn get_group_owner_and_privacy(
        group_identifier: Principal,
    ) -> Result<(Principal, Privacy), ApiError> {
        let group_privacy_response: Result<(Result<(Principal, Privacy), ApiError>,), _> =
            call::call(
                Identifier::decode(&group_identifier).1,
                "get_group_owner_and_privacy",
                (group_identifier,),
            )
            .await;

        DATA.with(|data| match group_privacy_response {
            Err(err) => Err(api_error(
                ApiErrorType::BadRequest,
                "INTER_CANISTER_CALL_FAILED",
                err.1.as_str(),
                Data::get_name(data).as_str(),
                "get_group",
                None,
            )),
            Ok((_group_privacy,)) => match _group_privacy {
                Err(err) => Err(err),
                Ok(__group_privacy) => Ok(__group_privacy),
            },
        })
    }

    // Method to map a member to a joined member response
    fn map_member_to_joined_member_response(
        identifier: &Principal,
        member: &Member,
        group_identifier: Principal,
    ) -> JoinedMemberResponse {
        let mut roles: Vec<String> = vec![];
        let joined_group = member
            .joined
            .iter()
            .find(|m| &m.group_identifier == &group_identifier);

        match joined_group {
            None => {}
            Some(_join) => roles = _join.roles.clone(),
        }

        JoinedMemberResponse {
            group_identifier: group_identifier,
            member_identifier: identifier.clone(),
            principal: member.principal,
            roles,
        }
    }

    // Method to map a member to an invite member response
    fn map_member_to_invite_member_response(
        identifier: &Principal,
        member: &Member,
        group_identifier: Principal,
    ) -> InviteMemberResponse {
        let invite = member
            .invites
            .iter()
            .find(|m| &m.group_identifier == &group_identifier);

        InviteMemberResponse {
            group_identifier: group_identifier,
            member_identifier: identifier.clone(),
            principal: member.principal,
            invite: invite.unwrap().clone(),
        }
    }

    // Method to get a member by caller principal
    fn _get_member_from_caller(caller: Principal) -> Option<(Principal, Member)> {
        let members = DATA.with(|data| Data::get_entries(data));
        members
            .into_iter()
            .find(|(_identifier, _member)| _member.principal == caller)
    }

    // Method to get the member count for a specific group
    fn _get_member_count_for_group(group_identifier: &Principal) -> usize {
        let members = DATA.with(|data| Data::get_entries(data));
        members
            .iter()
            .filter(|(_identifier, member)| {
                member
                    .joined
                    .iter()
                    .any(|j| &j.group_identifier == group_identifier)
            })
            .count()
    }

    // Default not found error
    fn _member_not_found_error(method_name: &str, inputs: Option<Vec<String>>) -> ApiError {
        api_error(
            ApiErrorType::NotFound,
            "MEMBER_NOT_FOUND",
            "Member not found",
            DATA.with(|data| Data::get_name(data)).as_str(),
            method_name,
            inputs,
        )
    }

    // [fire and forget]
    // Method to update the member count on the group canister (inter-canister call)
    #[allow(unused_must_use)]
    fn update_member_count_on_group(group_identifier: &Principal) -> () {
        // Get the member count for the group
        let group_member_count_array =
            Self::get_group_members_count(vec![group_identifier.clone()]);
        let mut count = 0;

        // If the group has members, set the count to the length of the array
        if group_member_count_array.len() > 0 {
            count = group_member_count_array[0].1;
        };

        let (_, group_canister, _) = Identifier::decode(group_identifier);
        // Call the update_member_count method on the group canister and send the total amount of members of the group with it
        call::call::<(Principal, Principal, usize), ()>(
            group_canister,
            "update_member_count",
            (group_identifier.clone(), id(), count),
        );
    }

    // Method to check if a member has a specific permission
    pub async fn can_write_member(
        caller: Principal,
        group_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        Self::check_permission(
            caller,
            group_identifier,
            PermissionActionType::Write,
            PermissionType::Member(None),
        )
        .await
    }

    // Method to check if a member has a specific permission
    pub async fn can_read_member(
        caller: Principal,
        group_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        Self::check_permission(
            caller,
            group_identifier,
            PermissionActionType::Read,
            PermissionType::Member(None),
        )
        .await
    }

    // Method to check if a member has a specific permission
    pub async fn can_edit_member(
        caller: Principal,
        group_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        Self::check_permission(
            caller,
            group_identifier,
            PermissionActionType::Edit,
            PermissionType::Member(None),
        )
        .await
    }

    // Method to check if a member has a specific permission
    pub async fn can_delete_member(
        caller: Principal,
        group_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        Self::check_permission(
            caller,
            group_identifier,
            PermissionActionType::Delete,
            PermissionType::Member(None),
        )
        .await
    }

    // Method to check if a member has a specific permission
    pub async fn can_write_invite(
        caller: Principal,
        group_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        Self::check_permission(
            caller,
            group_identifier,
            PermissionActionType::Write,
            PermissionType::Invite(None),
        )
        .await
    }

    // Method to check if a member has a specific permission
    pub async fn can_read_invite(
        caller: Principal,
        group_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        Self::check_permission(
            caller,
            group_identifier,
            PermissionActionType::Read,
            PermissionType::Invite(None),
        )
        .await
    }

    // Method to check if a member has a specific permission
    pub async fn can_edit_invite(
        caller: Principal,
        group_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        Self::check_permission(
            caller,
            group_identifier,
            PermissionActionType::Edit,
            PermissionType::Invite(None),
        )
        .await
    }

    // Method to check if a member has a specific permission
    pub async fn can_delete_invite(
        caller: Principal,
        group_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        Self::check_permission(
            caller,
            group_identifier,
            PermissionActionType::Delete,
            PermissionType::Invite(None),
        )
        .await
    }

    // Method to check if a member has a specific permission
    async fn check_permission(
        caller: Principal,
        group_identifier: Principal,
        permission: PermissionActionType,
        permission_type: PermissionType,
    ) -> Result<Principal, ApiError> {
        // Get the roles of the group (inter-canister call)
        let group_roles = get_group_roles(group_identifier).await;
        // Get the roles of the member
        let member_roles = Self::get_member_roles_by_principal(caller, group_identifier);

        match member_roles {
            // If the member has roles
            Ok((_principal, _roles)) => {
                // Check if the caller is the principal of the member, if not, return an unauthorized error
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

                match group_roles {
                    // If the group has roles, check if the member has the permission
                    Ok(mut _group_roles) => {
                        _group_roles.append(&mut default_roles());
                        let has_permission =
                            has_permission(&_roles, &permission_type, &_group_roles, &permission);

                        // If the member doesn't have the permission, return an unauthorized error
                        if !has_permission {
                            return Err(api_error(
                                ApiErrorType::Unauthorized,
                                "NO_PERMISSION",
                                "No permission",
                                DATA.with(|data| Data::get_name(data)).as_str(),
                                "check_permission",
                                None,
                            ));
                        }
                        // If the member has the permission, return the principal
                        Ok(caller)
                    }
                    // If the group doesn't have roles, return an unauthorized error
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
            // If the member doesn't have roles, return an unauthorized error
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

    // Used for composite_query calls from the parent canister
    //
    // Method to get filtered members serialized and chunked
    pub fn get_chunked_join_data(
        group_identifier: &Principal,
        chunk: usize,
        max_bytes_per_chunk: usize,
    ) -> (Vec<u8>, (usize, usize)) {
        let members = DATA.with(|data| Data::get_entries(data));
        // Get members for filtering
        let mapped_members: Vec<JoinedMemberResponse> = members
            .iter()
            // Filter members that have joined the group
            .filter(|(_identifier, _member_data)| {
                _member_data
                    .joined
                    .iter()
                    .any(|j| &j.group_identifier == group_identifier)
            })
            // Map member to joined member response
            .map(|(_identifier, _group_data)| {
                Self::map_member_to_joined_member_response(
                    _identifier,
                    _group_data,
                    group_identifier.clone(),
                )
            })
            .collect();

        if let Ok(bytes) = serialize(&mapped_members) {
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

    // Used for composite_query calls from the parent canister
    //
    // Method to get filtered members serialized and chunked
    pub fn get_chunked_invite_data(
        group_identifier: &Principal,
        chunk: usize,
        max_bytes_per_chunk: usize,
    ) -> (Vec<u8>, (usize, usize)) {
        let members = DATA.with(|data| Data::get_entries(data));
        // Get members for filtering
        let mapped_members: Vec<InviteMemberResponse> = members
            .iter()
            // Filter members that have joined the group
            .filter(|(_identifier, _member_data)| {
                _member_data
                    .invites
                    .iter()
                    .any(|j| &j.group_identifier == group_identifier)
            })
            // Map member to joined member response
            .map(|(_identifier, _group_data)| {
                Self::map_member_to_invite_member_response(
                    _identifier,
                    _group_data,
                    group_identifier.clone(),
                )
            })
            .collect();

        if let Ok(bytes) = serialize(&mapped_members) {
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
}
