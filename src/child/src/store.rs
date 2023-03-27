use std::{cell::RefCell, collections::HashMap, iter::FromIterator, vec};

use candid::Principal;
use ic_cdk::{
    api::{call, time},
    id,
};
use ic_scalable_canister::store::Data;
use ic_scalable_misc::{
    enums::{
        api_error_type::{ApiError, ApiErrorType},
        privacy_type::{Gated, Privacy},
        whitelist_rights_type::WhitelistRights,
    },
    helpers::{
        error_helper::api_error,
        role_helper::{default_roles, get_group_roles, has_permission},
        token_canister_helper::{dip20_balance_of, ext_balance_of, legacy_dip721_balance_of},
    },
    models::{
        identifier_model::Identifier,
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

                let updated_member = Self::add_invite_or_join_group_to_member(
                    caller,
                    group_identifier,
                    existing_member.clone(),
                    _group_privacy,
                    account_identifier,
                )
                .await;

                match updated_member {
                    Err(err) => Err(err),
                    Ok(_updated_member) => match existing_member {
                        None => {
                            let result = DATA.with(|data| {
                                Data::add_entry(data, _updated_member, Some("mbr".to_string()))
                            });
                            Self::update_member_count_on_group(&group_identifier);
                            result
                        }
                        Some((_identifier, _)) => {
                            let result = DATA.with(|data| {
                                Data::update_entry(data, _identifier, _updated_member)
                            });
                            Self::update_member_count_on_group(&group_identifier);
                            result
                        }
                    },
                }
                // add scaling logic
                // Determine if an entry needs to be updated or added as a new one
            }
        }
    }

    pub fn create_empty_member(
        caller: Principal,
        profile_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        let (_, _, kind) = Identifier::decode(&profile_identifier);
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
            let existing = Self::_get_member_from_caller(caller);

            match existing {
                None => {
                    let empty_member = Member {
                        principal: caller,
                        profile_identifier,
                        joined: vec![],
                        invites: vec![],
                    };
                    let result = DATA
                        .with(|data| Data::add_entry(data, empty_member, Some("mbr".to_string())));
                    match result {
                        Ok((_identfier, _)) => Ok(_identfier),
                        Err(err) => Err(err),
                    }
                }
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

    pub fn leave_group(caller: Principal, group_identifier: Principal) -> Result<(), ApiError> {
        let existing_member = Self::_get_member_from_caller(caller);

        match existing_member {
            None => Err(Self::_member_not_found_error("leave_group", None)),
            Some((_identifier, mut _member)) => {
                let joined: Vec<Join> = _member
                    .joined
                    .iter()
                    .filter(|j| &j.group_identifier != &group_identifier)
                    .cloned()
                    .collect();

                _member.joined = joined;
                let _ = DATA.with(|data| Data::update_entry(data, _identifier, _member));
                Ok(Self::update_member_count_on_group(&group_identifier))
            }
        }
    }

    pub fn remove_invite(caller: Principal, group_identifier: Principal) -> Result<(), ApiError> {
        let existing_member = Self::_get_member_from_caller(caller);
        match existing_member {
            None => Err(Self::_member_not_found_error("leave_group", None)),
            Some((_identifier, mut _member)) => {
                let invites: Vec<Invite> = _member
                    .invites
                    .into_iter()
                    .filter(|j| &j.group_identifier != &group_identifier)
                    .collect();

                _member.invites = invites;
                let _ = DATA.with(|data| Data::update_entry(data, _identifier, _member));
                Ok(())
            }
        }
    }

    pub fn assign_role(
        role: String,
        member_identifier: Principal,
        group_identifier: Principal,
    ) -> Result<(), ()> {
        let member = DATA.with(|data| Data::get_entry(data, member_identifier));
        if let Ok((_identifier, mut _member)) = member {
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

            _member.joined = joined;
            let _ = DATA.with(|data| Data::update_entry(data, _identifier, _member));
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn remove_role(
        role: String,
        member_identifier: Principal,
        group_identifier: Principal,
    ) -> Result<(), ()> {
        let member = DATA.with(|data| Data::get_entry(data, member_identifier));
        if let Ok((_identifier, mut _member)) = member {
            let joined: Vec<Join> = _member
                .joined
                .into_iter()
                .map(|j| {
                    if j.group_identifier == group_identifier {
                        let roles: Vec<String> =
                            j.roles.into_iter().filter(|r| r != &role).collect();
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

            _member.joined = joined;
            let _ = DATA.with(|data| Data::update_entry(data, _identifier, _member));
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn remove_join_from_member(
        caller: Principal,
        member_principal: Principal,
        group_identifier: Principal,
    ) -> Result<(), ApiError> {
        if let Some((_, _member)) = Self::_get_member_from_caller(caller) {
            if _member.joined.iter().any(|j| {
                j.group_identifier == group_identifier && j.roles.contains(&"owner".to_string())
            }) {
                match Self::_get_member_from_caller(member_principal) {
                    None => Err(Self::_member_not_found_error(
                        "remove_join_from_member",
                        None,
                    )),
                    Some((_identifier, mut _member)) => {
                        let joined: Vec<Join> = _member
                            .joined
                            .into_iter()
                            .filter(|j| &j.group_identifier != &group_identifier)
                            .collect();

                        _member.joined = joined;
                        let _ = DATA.with(|data| Data::update_entry(data, _identifier, _member));
                        return Ok(Self::update_member_count_on_group(&group_identifier));
                    }
                }
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
        } else {
            Err(Self::_member_not_found_error(
                "remove_join_from_member",
                None,
            ))
        }
    }

    pub fn remove_invite_from_member(
        caller: Principal,
        group_identifier: Principal,
    ) -> Result<(), ApiError> {
        let existing = Self::_get_member_from_caller(caller);
        match existing {
            None => Err(Self::_member_not_found_error(
                "remove_invite_from_member",
                None,
            )),
            Some((_identifier, mut _member)) => {
                let invites: Vec<Invite> = _member
                    .invites
                    .into_iter()
                    .filter(|j| &j.group_identifier != &group_identifier)
                    .collect();

                _member.invites = invites;
                let _ = DATA.with(|data| Data::update_entry(data, _identifier, _member));
                Ok(())
            }
        }
    }

    async fn add_invite_or_join_group_to_member(
        caller: Principal,
        group_identifier: Principal,
        member: Option<(Principal, Member)>,
        group_privacy: Privacy,
        account_identifier: Option<String>,
    ) -> Result<Member, ApiError> {
        let join = Join {
            group_identifier: group_identifier.clone(),
            roles: vec!["member".to_string()],
            updated_at: time(),
            created_at: time(),
        };

        let invite = Invite {
            group_identifier,
            invite_type: InviteType::UserRequest,
            updated_at: time(),
            created_at: time(),
        };

        use Privacy::*;
        match group_privacy {
            // Create a joined entry based on the group privacy settings
            Public => match member {
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
            },
            // Create a invite entry based on the group privacy settings
            Private => match member {
                None => Ok(Member {
                    principal: caller,
                    profile_identifier: Principal::anonymous(),
                    joined: vec![],
                    invites: vec![invite],
                }),
                Some((_, _member)) => {
                    let mut invites = _member.invites;
                    invites.push(invite);
                    Ok(Member {
                        principal: _member.principal,
                        profile_identifier: _member.profile_identifier,
                        joined: _member.joined,
                        invites,
                    })
                }
            },
            // This method needs a different call to split the logic
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
            Gated(nft_canisters) => {
                let mut is_valid = false;
                for nft_canister in nft_canisters {
                    is_valid =
                        Self::validate_gated(caller, account_identifier.clone(), nft_canister)
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
                        Some((_, _member)) => {
                            let mut joined = _member.joined;
                            joined.push(join);
                            Ok(Member {
                                principal: _member.principal,
                                profile_identifier: _member.profile_identifier,
                                joined,
                                invites: _member.invites,
                            })
                        }
                    }
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

    pub async fn validate_gated(
        principal: Principal,
        account_identifier: Option<String>,
        nft_canister: Gated,
    ) -> bool {
        match nft_canister.standard.as_str() {
            "EXT" => match account_identifier {
                Some(_account_identifier) => {
                    let response =
                        ext_balance_of(nft_canister.principal, _account_identifier).await;
                    response as u64 >= nft_canister.amount
                }
                None => false,
            },
            "DIP20" => {
                let response = dip20_balance_of(nft_canister.principal, principal).await;
                response as u64 >= nft_canister.amount
            }
            "DIP721" => {
                let response = dip20_balance_of(nft_canister.principal, principal).await;
                response as u64 >= nft_canister.amount
            }
            "DIP721_LEGACY" => {
                let response = legacy_dip721_balance_of(nft_canister.principal, principal).await;
                response as u64 >= nft_canister.amount
            }
            _ => false,
        }
    }

    pub fn get_self(caller: Principal) -> Result<(Principal, Member), ApiError> {
        let existing = Self::_get_member_from_caller(caller);
        match existing {
            None => Err(Self::_member_not_found_error("get_self", None)),
            Some(_member) => Ok(_member),
        }
    }

    pub fn get_member_roles(
        member_identifier: Principal,
        group_identifier: Principal,
    ) -> Result<(Principal, Vec<String>), String> {
        let member = DATA.with(|data| Data::get_entry(data, member_identifier));

        match member {
            Ok((_member_identifier, _member)) => {
                if let Some(_join) = _member
                    .joined
                    .iter()
                    .find(|j| j.group_identifier == group_identifier)
                {
                    Ok((_member.principal, _join.roles.clone()))
                } else {
                    Ok((_member.principal, vec![]))
                }
            }
            Err(_) => Err("No member found".to_string()),
        }
    }

    pub fn get_member_roles_by_principal(
        principal: Principal,
        group_identifier: Principal,
    ) -> Result<(Principal, Vec<String>), String> {
        let member = Self::_get_member_from_caller(principal);

        match member {
            Some((_member_identifier, _member)) => {
                if let Some(_join) = _member
                    .joined
                    .iter()
                    .find(|j| j.group_identifier == group_identifier)
                {
                    Ok((_member.principal, _join.roles.clone()))
                } else {
                    Ok((_member.principal, vec![]))
                }
            }
            None => Err("No member found".to_string()),
        }
    }

    // pub async fn has_role(
    //     caller: Principal,
    //     group_identifier: Principal,
    //     permission_name: String,
    //     permission: PermissionActionType,
    // ) -> Result<bool, String> {
    //     if has_default_role(&permission_name, &permission) {
    //         return Ok(true);
    //     }

    //     // destructure the group identifier and check the kind
    //     let (_, _group_canister, _kind) = Identifier::decode(&group_identifier);

    //     //throw error if its the wrong kind
    //     if _kind != "grp".to_string() {
    //         return Err("Wrong principal kind".to_string());
    //     };

    //     // get the existing member
    //     match Self::_get_member_from_caller(caller) {
    //         Some((_existing_principal, _existing_member)) => {
    //             let joined = _existing_member
    //                 .joined
    //                 .iter()
    //                 .find(|j| j.group_identifier == group_identifier);

    //             match joined {
    //                 Some(_existing_joined) => {
    //                     // do inter-canister call to fetch the roles of the group
    //                     let group_roles: Result<(Vec<GroupRole>,), _> =
    //                         call::call(_group_canister, "get_roles", (group_identifier,)).await;

    //                     let mut member_role_with_permissions: Vec<GroupRole> = vec![];

    //                     // iterate over the roles assigned to the member
    //                     if let Ok(_group_role) = group_roles {
    //                         for role in _existing_joined.roles.iter() {
    //                             if let Some(_found_role) =
    //                                 _group_role.0.iter().find(|r| &r.name == role)
    //                             {
    //                                 member_role_with_permissions.push(_found_role.clone());
    //                             }
    //                         }
    //                     }
    //                     use PermissionActionType::*;
    //                     let result = member_role_with_permissions.iter().any(|v| {
    //                         v.permissions.iter().any(|p| {
    //                             p.name == permission_name
    //                                 && match permission {
    //                                     Write => p.actions.write == true,
    //                                     Read => p.actions.read == true,
    //                                     Edit => p.actions.edit == true,
    //                                     Delete => p.actions.delete == true,
    //                                 }
    //                         })
    //                     });
    //                     Ok(result)
    //                 }
    //                 None => Err("User not part of this group".to_string()),
    //             }
    //         }
    //         None => Err("No member found".to_string()),
    //     }
    // }

    pub fn get_group_member_by_user_principal(
        caller: Principal,
        group_identifier: Principal,
    ) -> Result<JoinedMemberResponse, ApiError> {
        DATA.with(|data| {
            let member = Store::_get_member_from_caller(caller);
            match member {
                None => Err(Self::_member_not_found_error(
                    "get_group_member_by_user_principal",
                    None,
                )),
                Some((_identifier, _member)) => {
                    let join = _member
                        .joined
                        .iter()
                        .find(|j| &j.group_identifier == &group_identifier);

                    match join {
                        None => Err(api_error(
                            ApiErrorType::NotFound,
                            "NOT_JOINED",
                            "You have no roles within this group",
                            Data::get_name(data).as_str(),
                            "get_group_member_by_user_principal",
                            None,
                        )),
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

    pub fn get_group_members(group_identifier: Principal) -> Vec<JoinedMemberResponse> {
        DATA.with(|data| {
            let members = Data::get_entries(data);

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

    pub fn get_group_members_count(group_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {
        let mut members_counts: Vec<(Principal, usize)> = vec![];

        DATA.with(|data| {
            let members = Data::get_entries(data);

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
                members_counts.push((group_identifier, count));
            }
        });

        members_counts
    }

    pub fn get_groups_for_members(
        member_identifier: Vec<Principal>,
    ) -> Vec<(Principal, Vec<Principal>)> {
        let mut members_with_groups: Vec<(Principal, Vec<Principal>)> = vec![];

        for _member_identifier in member_identifier {
            let mut groups: Vec<Principal> = vec![];
            let member = DATA.with(|data| Data::get_entry(data, _member_identifier));

            if let Ok((_, _member)) = member {
                for joined in _member.joined.iter() {
                    groups.push(joined.group_identifier.clone());
                }
            }
            members_with_groups.push((_member_identifier, groups));
        }

        members_with_groups
    }

    pub fn get_group_invites_count(group_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {
        let mut members_counts: Vec<(Principal, usize)> = vec![];

        DATA.with(|data| {
            let members = Data::get_entries(data);

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
                members_counts.push((group_identifier, count));
            }
        });

        members_counts
    }

    pub fn get_group_invites(group_identifier: Principal) -> Vec<InviteMemberResponse> {
        DATA.with(|data| {
            let members = Data::get_entries(data);

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

    // fire this after storing the owner
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
                }
            }
        })
    }

    pub fn invite_to_group(
        caller: Principal,
        group_identifier: Principal,
        member_principal: Principal,
    ) -> Result<(Principal, Member), ApiError> {
        DATA.with(|data| {
            let group_owner = Self::get_group_owner(&group_identifier);
            match group_owner {
                None => Err(api_error(
                    ApiErrorType::Unauthorized,
                    "NO_OWNER",
                    "You are not allowed to invite people to the group",
                    Data::get_name(data).as_str(),
                    "invite_to_group",
                    None,
                )),
                Some((_identifier, _owner)) => {
                    if _owner.principal != caller {
                        return Err(api_error(
                            ApiErrorType::Unauthorized,
                            "INVITE_BLOCKED",
                            "You are not allowed to invite people to the group",
                            Data::get_name(data).as_str(),
                            "invite_to_group",
                            None,
                        ));
                    }

                    let existing_member = Self::_get_member_from_caller(member_principal);

                    let invite = Invite {
                        group_identifier,
                        invite_type: InviteType::OwnerRequest,
                        updated_at: time(),
                        created_at: time(),
                    };

                    match existing_member {
                        None => {
                            let member = Member {
                                principal: member_principal,
                                profile_identifier: Principal::anonymous(),
                                joined: vec![],
                                invites: vec![invite],
                            };
                            Data::add_entry(data, member, Some("mbr".to_string()))
                        }
                        Some((_identifier, mut _member)) => {
                            _member.invites.push(invite);
                            Data::update_entry(data, _identifier, _member)
                        }
                    }
                }
            }
        })
    }

    pub fn accept_user_request_group_invite(
        member_principal: Principal,
        group_identifier: Principal,
    ) -> Result<(Principal, Member), ApiError> {
        let member = Self::_get_member_from_caller(member_principal);

        match member {
            None => Err(Self::_member_not_found_error(
                "remove_invite_from_member",
                None,
            )),
            Some((_identifier, mut _member)) => {
                let invite = _member
                    .invites
                    .iter()
                    .find(|i| &i.group_identifier == &group_identifier);

                match invite {
                    None => Err(api_error(
                        ApiErrorType::NotFound,
                        "NO_INVITE_FOUND",
                        "There is no invite found for this group",
                        DATA.with(|data| Data::get_name(data)).as_str(),
                        "accept_user_request_group_invite",
                        None,
                    )),
                    Some(_invite) => {
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

                        _member.invites = _member
                            .invites
                            .into_iter()
                            .filter(|i| &i.group_identifier != &group_identifier)
                            .collect();

                        _member.joined.push(Join {
                            group_identifier,
                            roles: vec!["member".to_string()],
                            updated_at: time(),
                            created_at: time(),
                        });
                        let result =
                            DATA.with(|data| Data::update_entry(data, _identifier, _member));
                        Self::update_member_count_on_group(&group_identifier);
                        result
                    }
                }
            }
        }
    }

    pub fn accept_owner_request_group_invite(
        caller: Principal,
        group_identifier: Principal,
    ) -> Result<(Principal, Member), ApiError> {
        DATA.with(|data| {
            let existing_member = Self::_get_member_from_caller(caller);

            match existing_member {
                None => Err(Self::_member_not_found_error(
                    "remove_invite_from_member",
                    None,
                )),
                Some((_identifier, mut _member)) => {
                    let invite = _member
                        .invites
                        .iter()
                        .find(|i| &i.group_identifier == &group_identifier);
                    match invite {
                        None => Err(api_error(
                            ApiErrorType::NotFound,
                            "NO_INVITE_FOUND",
                            "There is no invite found for this group",
                            Data::get_name(data).as_str(),
                            "accept_owner_request_group_invite",
                            None,
                        )),
                        Some(_invite) => {
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

                            _member.invites = _member
                                .invites
                                .iter()
                                .filter(|i| &i.group_identifier == &group_identifier)
                                .cloned()
                                .collect();

                            _member.joined.push(Join {
                                group_identifier,
                                roles: vec!["member".to_string()],
                                updated_at: time(),
                                created_at: time(),
                            });
                            let result = Data::update_entry(data, _identifier, _member);
                            Self::update_member_count_on_group(&group_identifier);
                            result
                        }
                    }
                }
            }
        })
    }

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

    fn get_group_owner(group_identifier: &Principal) -> Option<(Principal, Member)> {
        DATA.with(|data| {
            let members = Data::get_entries(data);
            let existing_member = members.into_iter().find(|(_identifier, _member)| {
                _member.joined.iter().any(|j| {
                    j.roles.contains(&"owner".to_string())
                        && &j.group_identifier == group_identifier
                })
            });

            existing_member
        })
    }

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

    fn _get_member_from_caller(caller: Principal) -> Option<(Principal, Member)> {
        let members = DATA.with(|data| Data::get_entries(data));
        members
            .into_iter()
            .find(|(_identifier, _member)| _member.principal == caller)
    }

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

    #[allow(unused_must_use)]
    fn update_member_count_on_group(group_identifier: &Principal) -> () {
        let group_member_count_array =
            Self::get_group_members_count(vec![group_identifier.clone()]);
        let mut count = 0;

        if group_member_count_array.len() > 0 {
            count = group_member_count_array[0].1;
        };

        let (_, group_canister, _) = Identifier::decode(group_identifier);
        call::call::<(Principal, Principal, usize), ()>(
            group_canister,
            "update_member_count",
            (group_identifier.clone(), id(), count),
        );
    }

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

    async fn check_permission(
        caller: Principal,
        group_identifier: Principal,
        permission: PermissionActionType,
        permission_type: PermissionType,
    ) -> Result<Principal, ApiError> {
        let group_roles = get_group_roles(group_identifier).await;
        let member_roles = Self::get_member_roles_by_principal(caller, group_identifier);

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

                match group_roles {
                    Ok(mut _group_roles) => {
                        _group_roles.append(&mut default_roles());
                        let has_permission =
                            has_permission(&_roles, &permission_type, &_group_roles, &permission);

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
