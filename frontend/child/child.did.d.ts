import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export type ApiError = { 'SerializeError' : ErrorMessage } |
  { 'DeserializeError' : ErrorMessage } |
  { 'NotFound' : ErrorMessage } |
  { 'ValidationError' : Array<ValidationResponse> } |
  { 'CanisterAtCapacity' : ErrorMessage } |
  { 'UpdateRequired' : UpdateMessage } |
  { 'Unauthorized' : ErrorMessage } |
  { 'Unexpected' : ErrorMessage } |
  { 'BadRequest' : ErrorMessage };
export interface CanisterStatusResponse {
  'status' : CanisterStatusType,
  'memory_size' : bigint,
  'cycles' : bigint,
  'settings' : DefiniteCanisterSettings,
  'idle_cycles_burned_per_day' : bigint,
  'module_hash' : [] | [Uint8Array | number[]],
}
export type CanisterStatusType = { 'stopped' : null } |
  { 'stopping' : null } |
  { 'running' : null };
export interface DefiniteCanisterSettings {
  'freezing_threshold' : bigint,
  'controllers' : Array<Principal>,
  'memory_allocation' : bigint,
  'compute_allocation' : bigint,
}
export interface ErrorMessage {
  'tag' : string,
  'message' : string,
  'inputs' : [] | [Array<string>],
  'location' : string,
}
export interface HttpHeader { 'value' : string, 'name' : string }
export interface HttpRequest {
  'url' : string,
  'method' : string,
  'body' : Uint8Array | number[],
  'headers' : Array<[string, string]>,
}
export interface HttpResponse {
  'status' : bigint,
  'body' : Uint8Array | number[],
  'headers' : Array<HttpHeader>,
}
export interface Invite {
  'updated_at' : bigint,
  'invite_type' : InviteType,
  'created_at' : bigint,
}
export interface InviteMemberResponse {
  'principal' : Principal,
  'group_identifier' : Principal,
  'invite' : Invite,
  'member_identifier' : Principal,
}
export type InviteType = { 'OwnerRequest' : null } |
  { 'UserRequest' : null };
export interface Join {
  'updated_at' : bigint,
  'created_at' : bigint,
  'roles' : Array<string>,
}
export interface JoinedMemberResponse {
  'principal' : Principal,
  'group_identifier' : Principal,
  'member_identifier' : Principal,
  'roles' : Array<string>,
}
export interface Member {
  'principal' : Principal,
  'invites' : Array<[Principal, Invite]>,
  'joined' : Array<[Principal, Join]>,
  'profile_identifier' : Principal,
}
export type RejectionCode = { 'NoError' : null } |
  { 'CanisterError' : null } |
  { 'SysTransient' : null } |
  { 'DestinationInvalid' : null } |
  { 'Unknown' : null } |
  { 'SysFatal' : null } |
  { 'CanisterReject' : null };
export type Result = { 'Ok' : [Principal, Member] } |
  { 'Err' : ApiError };
export type Result_1 = { 'Ok' : null } |
  { 'Err' : ApiError };
export type Result_2 = { 'Ok' : Principal } |
  { 'Err' : ApiError };
export type Result_3 = { 'Ok' : null } |
  { 'Err' : null };
export type Result_4 = { 'Ok' : [CanisterStatusResponse] } |
  { 'Err' : [RejectionCode, string] };
export type Result_5 = { 'Ok' : Array<InviteMemberResponse> } |
  { 'Err' : ApiError };
export type Result_6 = { 'Ok' : JoinedMemberResponse } |
  { 'Err' : ApiError };
export type Result_7 = { 'Ok' : Array<JoinedMemberResponse> } |
  { 'Err' : ApiError };
export type Result_8 = { 'Ok' : [Principal, Array<string>] } |
  { 'Err' : string };
export interface UpdateMessage {
  'canister_principal' : Principal,
  'message' : string,
}
export interface ValidationResponse { 'field' : string, 'message' : string }
export interface _SERVICE {
  '__get_candid_interface_tmp_hack' : ActorMethod<[], string>,
  'accept_cycles' : ActorMethod<[], bigint>,
  'accept_owner_request_group_invite' : ActorMethod<[Principal], Result>,
  'accept_user_request_group_invite' : ActorMethod<
    [Principal, Principal],
    Result
  >,
  'add_entry_by_parent' : ActorMethod<[Uint8Array | number[]], Result_1>,
  'add_owner' : ActorMethod<[Principal, Principal], Result_2>,
  'assign_role' : ActorMethod<[string, Principal, Principal], Result_3>,
  'canister_backup_data' : ActorMethod<[], [string, string]>,
  'canister_clear_backup' : ActorMethod<[], undefined>,
  'canister_finalize_upload' : ActorMethod<
    [Uint8Array | number[], Uint8Array | number[]],
    undefined
  >,
  'canister_restore_data' : ActorMethod<[], undefined>,
  'canister_status' : ActorMethod<[], Result_4>,
  'clear_backup' : ActorMethod<[], undefined>,
  'create_empty_member' : ActorMethod<[Principal, Principal], Result_2>,
  'download_chunk' : ActorMethod<[bigint], [bigint, Uint8Array | number[]]>,
  'download_entries_chunk' : ActorMethod<
    [bigint],
    [bigint, Uint8Array | number[]]
  >,
  'download_stable_data_chunk' : ActorMethod<
    [bigint],
    [bigint, Uint8Array | number[]]
  >,
  'finalize_upload' : ActorMethod<[], string>,
  'get_chunked_invite_data' : ActorMethod<
    [Principal, bigint, bigint],
    [Uint8Array | number[], [bigint, bigint]]
  >,
  'get_chunked_join_data' : ActorMethod<
    [Principal, bigint, bigint],
    [Uint8Array | number[], [bigint, bigint]]
  >,
  'get_group_invites' : ActorMethod<[Principal], Result_5>,
  'get_group_invites_count' : ActorMethod<
    [Array<Principal>],
    Array<[Principal, bigint]>
  >,
  'get_group_member' : ActorMethod<[Principal, Principal], Result_6>,
  'get_group_members' : ActorMethod<[Principal], Result_7>,
  'get_group_members_count' : ActorMethod<
    [Array<Principal>],
    Array<[Principal, bigint]>
  >,
  'get_groups_for_members' : ActorMethod<
    [Array<Principal>],
    Array<[Principal, Array<Principal>]>
  >,
  'get_member_roles' : ActorMethod<[Principal, Principal], Result_8>,
  'get_self' : ActorMethod<[], Result>,
  'hashes' : ActorMethod<[], [string, string]>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'invite_to_group' : ActorMethod<[Principal, Principal], Result>,
  'join_group' : ActorMethod<[Principal, [] | [string]], Result>,
  'leave_group' : ActorMethod<[Principal], Result_1>,
  'remove_invite' : ActorMethod<[Principal], Result_1>,
  'remove_member_from_group' : ActorMethod<[Principal, Principal], Result_1>,
  'remove_member_invite_from_group' : ActorMethod<
    [Principal, Principal],
    Result_1
  >,
  'remove_role' : ActorMethod<[string, Principal, Principal], Result_3>,
  'restore_data' : ActorMethod<[], undefined>,
  'set_roles' : ActorMethod<[Array<string>, Principal, Principal], Result_3>,
  'total_chunks' : ActorMethod<[], bigint>,
  'total_entries_chunks' : ActorMethod<[], bigint>,
  'total_stable_data_chunks' : ActorMethod<[], bigint>,
  'upload_chunk' : ActorMethod<[[bigint, Uint8Array | number[]]], undefined>,
  'upload_entries_chunk' : ActorMethod<
    [[bigint, Uint8Array | number[]]],
    undefined
  >,
  'upload_stable_data_chunk' : ActorMethod<
    [[bigint, Uint8Array | number[]]],
    undefined
  >,
}
