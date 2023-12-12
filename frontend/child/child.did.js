export const idlFactory = ({ IDL }) => {
  const InviteType = IDL.Variant({
    'OwnerRequest' : IDL.Null,
    'UserRequest' : IDL.Null,
  });
  const Invite = IDL.Record({
    'updated_at' : IDL.Nat64,
    'invite_type' : InviteType,
    'created_at' : IDL.Nat64,
  });
  const Join = IDL.Record({
    'updated_at' : IDL.Nat64,
    'created_at' : IDL.Nat64,
    'roles' : IDL.Vec(IDL.Text),
  });
  const Member = IDL.Record({
    'principal' : IDL.Principal,
    'invites' : IDL.Vec(IDL.Tuple(IDL.Principal, Invite)),
    'joined' : IDL.Vec(IDL.Tuple(IDL.Principal, Join)),
    'profile_identifier' : IDL.Principal,
  });
  const ErrorMessage = IDL.Record({
    'tag' : IDL.Text,
    'message' : IDL.Text,
    'inputs' : IDL.Opt(IDL.Vec(IDL.Text)),
    'location' : IDL.Text,
  });
  const ValidationResponse = IDL.Record({
    'field' : IDL.Text,
    'message' : IDL.Text,
  });
  const UpdateMessage = IDL.Record({
    'canister_principal' : IDL.Principal,
    'message' : IDL.Text,
  });
  const ApiError = IDL.Variant({
    'SerializeError' : ErrorMessage,
    'DeserializeError' : ErrorMessage,
    'NotFound' : ErrorMessage,
    'ValidationError' : IDL.Vec(ValidationResponse),
    'CanisterAtCapacity' : ErrorMessage,
    'UpdateRequired' : UpdateMessage,
    'Unauthorized' : ErrorMessage,
    'Unexpected' : ErrorMessage,
    'BadRequest' : ErrorMessage,
  });
  const Result = IDL.Variant({
    'Ok' : IDL.Tuple(IDL.Principal, Member),
    'Err' : ApiError,
  });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : ApiError });
  const Result_2 = IDL.Variant({ 'Ok' : IDL.Principal, 'Err' : ApiError });
  const Result_3 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Null });
  const CanisterStatusType = IDL.Variant({
    'stopped' : IDL.Null,
    'stopping' : IDL.Null,
    'running' : IDL.Null,
  });
  const DefiniteCanisterSettings = IDL.Record({
    'freezing_threshold' : IDL.Nat,
    'controllers' : IDL.Vec(IDL.Principal),
    'memory_allocation' : IDL.Nat,
    'compute_allocation' : IDL.Nat,
  });
  const CanisterStatusResponse = IDL.Record({
    'status' : CanisterStatusType,
    'memory_size' : IDL.Nat,
    'cycles' : IDL.Nat,
    'settings' : DefiniteCanisterSettings,
    'idle_cycles_burned_per_day' : IDL.Nat,
    'module_hash' : IDL.Opt(IDL.Vec(IDL.Nat8)),
  });
  const RejectionCode = IDL.Variant({
    'NoError' : IDL.Null,
    'CanisterError' : IDL.Null,
    'SysTransient' : IDL.Null,
    'DestinationInvalid' : IDL.Null,
    'Unknown' : IDL.Null,
    'SysFatal' : IDL.Null,
    'CanisterReject' : IDL.Null,
  });
  const Result_4 = IDL.Variant({
    'Ok' : IDL.Tuple(CanisterStatusResponse),
    'Err' : IDL.Tuple(RejectionCode, IDL.Text),
  });
  const InviteMemberResponse = IDL.Record({
    'principal' : IDL.Principal,
    'group_identifier' : IDL.Principal,
    'invite' : Invite,
    'member_identifier' : IDL.Principal,
  });
  const Result_5 = IDL.Variant({
    'Ok' : IDL.Vec(InviteMemberResponse),
    'Err' : ApiError,
  });
  const JoinedMemberResponse = IDL.Record({
    'principal' : IDL.Principal,
    'group_identifier' : IDL.Principal,
    'member_identifier' : IDL.Principal,
    'roles' : IDL.Vec(IDL.Text),
  });
  const Result_6 = IDL.Variant({
    'Ok' : JoinedMemberResponse,
    'Err' : ApiError,
  });
  const Result_7 = IDL.Variant({
    'Ok' : IDL.Vec(JoinedMemberResponse),
    'Err' : ApiError,
  });
  const Result_8 = IDL.Variant({
    'Ok' : IDL.Tuple(IDL.Principal, IDL.Vec(IDL.Text)),
    'Err' : IDL.Text,
  });
  const HttpRequest = IDL.Record({
    'url' : IDL.Text,
    'method' : IDL.Text,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
  });
  const HttpHeader = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const HttpResponse = IDL.Record({
    'status' : IDL.Nat,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HttpHeader),
  });
  return IDL.Service({
    '__get_candid_interface_tmp_hack' : IDL.Func([], [IDL.Text], ['query']),
    'accept_cycles' : IDL.Func([], [IDL.Nat64], []),
    'accept_owner_request_group_invite' : IDL.Func(
        [IDL.Principal],
        [Result],
        [],
      ),
    'accept_user_request_group_invite' : IDL.Func(
        [IDL.Principal, IDL.Principal],
        [Result],
        [],
      ),
    'add_entry_by_parent' : IDL.Func([IDL.Vec(IDL.Nat8)], [Result_1], []),
    'add_owner' : IDL.Func([IDL.Principal, IDL.Principal], [Result_2], []),
    'assign_role' : IDL.Func(
        [IDL.Text, IDL.Principal, IDL.Principal],
        [Result_3],
        [],
      ),
    'canister_backup_data' : IDL.Func([], [IDL.Text, IDL.Text], []),
    'canister_clear_backup' : IDL.Func([], [], []),
    'canister_finalize_upload' : IDL.Func(
        [IDL.Vec(IDL.Nat8), IDL.Vec(IDL.Nat8)],
        [],
        [],
      ),
    'canister_restore_data' : IDL.Func([], [], []),
    'canister_status' : IDL.Func([], [Result_4], []),
    'clear_backup' : IDL.Func([], [], []),
    'create_empty_member' : IDL.Func(
        [IDL.Principal, IDL.Principal],
        [Result_2],
        [],
      ),
    'download_chunk' : IDL.Func(
        [IDL.Nat64],
        [IDL.Tuple(IDL.Nat64, IDL.Vec(IDL.Nat8))],
        ['query'],
      ),
    'download_entries_chunk' : IDL.Func(
        [IDL.Nat64],
        [IDL.Tuple(IDL.Nat64, IDL.Vec(IDL.Nat8))],
        ['query'],
      ),
    'download_stable_data_chunk' : IDL.Func(
        [IDL.Nat64],
        [IDL.Tuple(IDL.Nat64, IDL.Vec(IDL.Nat8))],
        ['query'],
      ),
    'finalize_upload' : IDL.Func([], [IDL.Text], []),
    'get_chunked_invite_data' : IDL.Func(
        [IDL.Principal, IDL.Nat64, IDL.Nat64],
        [IDL.Vec(IDL.Nat8), IDL.Tuple(IDL.Nat64, IDL.Nat64)],
        ['query'],
      ),
    'get_chunked_join_data' : IDL.Func(
        [IDL.Principal, IDL.Nat64, IDL.Nat64],
        [IDL.Vec(IDL.Nat8), IDL.Tuple(IDL.Nat64, IDL.Nat64)],
        ['query'],
      ),
    'get_group_invites' : IDL.Func([IDL.Principal], [Result_5], []),
    'get_group_invites_count' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Nat64))],
        ['query'],
      ),
    'get_group_member' : IDL.Func(
        [IDL.Principal, IDL.Principal],
        [Result_6],
        ['query'],
      ),
    'get_group_members' : IDL.Func([IDL.Principal], [Result_7], ['query']),
    'get_group_members_count' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Nat64))],
        ['query'],
      ),
    'get_groups_for_members' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Vec(IDL.Principal)))],
        ['query'],
      ),
    'get_member_roles' : IDL.Func(
        [IDL.Principal, IDL.Principal],
        [Result_8],
        ['query'],
      ),
    'get_self' : IDL.Func([], [Result], ['query']),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'invite_to_group' : IDL.Func([IDL.Principal, IDL.Principal], [Result], []),
    'join_group' : IDL.Func([IDL.Principal, IDL.Opt(IDL.Text)], [Result], []),
    'leave_group' : IDL.Func([IDL.Principal], [Result_1], []),
    'remove_invite' : IDL.Func([IDL.Principal], [Result_1], []),
    'remove_member_from_group' : IDL.Func(
        [IDL.Principal, IDL.Principal],
        [Result_1],
        [],
      ),
    'remove_member_invite_from_group' : IDL.Func(
        [IDL.Principal, IDL.Principal],
        [Result_1],
        [],
      ),
    'remove_role' : IDL.Func(
        [IDL.Text, IDL.Principal, IDL.Principal],
        [Result_3],
        [],
      ),
    'restore_data' : IDL.Func([], [], []),
    'set_roles' : IDL.Func(
        [IDL.Vec(IDL.Text), IDL.Principal, IDL.Principal],
        [Result_3],
        [],
      ),
    'total_chunks' : IDL.Func([], [IDL.Nat64], ['query']),
    'total_entries_chunks' : IDL.Func([], [IDL.Nat64], ['query']),
    'total_stable_data_chunks' : IDL.Func([], [IDL.Nat64], ['query']),
    'upload_chunk' : IDL.Func(
        [IDL.Tuple(IDL.Nat64, IDL.Vec(IDL.Nat8))],
        [],
        [],
      ),
    'upload_entries_chunk' : IDL.Func(
        [IDL.Tuple(IDL.Nat64, IDL.Vec(IDL.Nat8))],
        [],
        [],
      ),
    'upload_stable_data_chunk' : IDL.Func(
        [IDL.Tuple(IDL.Nat64, IDL.Vec(IDL.Nat8))],
        [],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  return [IDL.Principal, IDL.Text, IDL.Nat64];
};
