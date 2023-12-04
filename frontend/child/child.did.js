export const idlFactory = ({ IDL }) => {
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
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : ApiError });
  const ChunkData = IDL.Record({
    'chunk_id' : IDL.Nat64,
    'canister' : IDL.Principal,
    'index' : IDL.Nat64,
  });
  const Manifest = IDL.Record({ 'entries' : IDL.Vec(ChunkData) });
  const CanisterStorage = IDL.Variant({
    'None' : IDL.Null,
    'Manifest' : Manifest,
    'Chunk' : ChunkData,
  });
  const Asset = IDL.Variant({
    'Url' : IDL.Text,
    'None' : IDL.Null,
    'CanisterStorage' : CanisterStorage,
  });
  const NeuronGatedRules = IDL.Variant({
    'IsDisolving' : IDL.Bool,
    'MinStake' : IDL.Nat64,
    'MinAge' : IDL.Nat64,
    'MinDissolveDelay' : IDL.Nat64,
  });
  const NeuronGated = IDL.Record({
    'governance_canister' : IDL.Principal,
    'name' : IDL.Text,
    'description' : IDL.Text,
    'ledger_canister' : IDL.Principal,
    'rules' : IDL.Vec(NeuronGatedRules),
  });
  const TokenGated = IDL.Record({
    'principal' : IDL.Principal,
    'name' : IDL.Text,
    'description' : IDL.Text,
    'amount' : IDL.Nat64,
    'standard' : IDL.Text,
  });
  const GatedType = IDL.Variant({
    'Neuron' : IDL.Vec(NeuronGated),
    'Token' : IDL.Vec(TokenGated),
  });
  const Privacy = IDL.Variant({
    'Gated' : GatedType,
    'Private' : IDL.Null,
    'Public' : IDL.Null,
    'InviteOnly' : IDL.Null,
  });
  const Address = IDL.Record({
    'street' : IDL.Text,
    'country' : IDL.Text,
    'city' : IDL.Text,
    'postal_code' : IDL.Text,
    'label' : IDL.Text,
    'state_or_province' : IDL.Text,
    'house_number' : IDL.Text,
    'house_number_addition' : IDL.Text,
  });
  const PhysicalLocation = IDL.Record({
    'longtitude' : IDL.Float32,
    'address' : Address,
    'lattitude' : IDL.Float32,
  });
  const MultiLocation = IDL.Record({
    'physical' : PhysicalLocation,
    'digital' : IDL.Text,
  });
  const Location = IDL.Variant({
    'None' : IDL.Null,
    'Digital' : IDL.Text,
    'Physical' : PhysicalLocation,
    'MultiLocation' : MultiLocation,
  });
  const PostGroup = IDL.Record({
    'banner_image' : Asset,
    'name' : IDL.Text,
    'matrix_space_id' : IDL.Text,
    'tags' : IDL.Vec(IDL.Nat32),
    'description' : IDL.Text,
    'website' : IDL.Text,
    'privacy' : Privacy,
    'image' : Asset,
    'location' : Location,
  });
  const PermissionActions = IDL.Record({
    'edit' : IDL.Bool,
    'read' : IDL.Bool,
    'delete' : IDL.Bool,
    'write' : IDL.Bool,
  });
  const Permission = IDL.Record({
    'name' : IDL.Text,
    'actions' : PermissionActions,
    'protected' : IDL.Bool,
  });
  const GroupRole = IDL.Record({
    'permissions' : IDL.Vec(Permission),
    'name' : IDL.Text,
    'color' : IDL.Text,
    'protected' : IDL.Bool,
    'index' : IDL.Opt(IDL.Nat64),
  });
  const GroupResponse = IDL.Record({
    'updated_on' : IDL.Nat64,
    'banner_image' : Asset,
    'owner' : IDL.Principal,
    'name' : IDL.Text,
    'matrix_space_id' : IDL.Text,
    'tags' : IDL.Vec(IDL.Nat32),
    'description' : IDL.Text,
    'created_by' : IDL.Principal,
    'created_on' : IDL.Nat64,
    'website' : IDL.Text,
    'privacy' : Privacy,
    'wallets' : IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Text)),
    'image' : Asset,
    'identifier' : IDL.Principal,
    'member_count' : IDL.Nat64,
    'location' : Location,
    'roles' : IDL.Vec(GroupRole),
    'is_deleted' : IDL.Bool,
  });
  const Result_1 = IDL.Variant({ 'Ok' : GroupResponse, 'Err' : ApiError });
  const Result_2 = IDL.Variant({ 'Ok' : GroupRole, 'Err' : ApiError });
  const Group = IDL.Record({
    'updated_on' : IDL.Nat64,
    'banner_image' : Asset,
    'owner' : IDL.Principal,
    'name' : IDL.Text,
    'matrix_space_id' : IDL.Text,
    'tags' : IDL.Vec(IDL.Nat32),
    'description' : IDL.Text,
    'created_by' : IDL.Principal,
    'created_on' : IDL.Nat64,
    'website' : IDL.Text,
    'privacy' : Privacy,
    'wallets' : IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Text)),
    'image' : Asset,
    'member_count' : IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Nat64)),
    'location' : Location,
    'roles' : IDL.Vec(GroupRole),
    'is_deleted' : IDL.Bool,
  });
  const Result_3 = IDL.Variant({ 'Ok' : Group, 'Err' : ApiError });
  const UpdateGroup = IDL.Record({
    'banner_image' : Asset,
    'name' : IDL.Text,
    'tags' : IDL.Vec(IDL.Nat32),
    'description' : IDL.Text,
    'website' : IDL.Text,
    'privacy' : Privacy,
    'image' : Asset,
    'location' : Location,
  });
  const PostPermission = IDL.Record({
    'name' : IDL.Text,
    'actions' : PermissionActions,
  });
  const Result_4 = IDL.Variant({ 'Ok' : IDL.Bool, 'Err' : ApiError });
  const DateRange = IDL.Record({
    'end_date' : IDL.Nat64,
    'start_date' : IDL.Nat64,
  });
  const GroupFilter = IDL.Variant({
    'Tag' : IDL.Nat32,
    'UpdatedOn' : DateRange,
    'MemberCount' : IDL.Tuple(IDL.Nat64, IDL.Nat64),
    'Name' : IDL.Text,
    'Identifiers' : IDL.Vec(IDL.Principal),
    'Owner' : IDL.Principal,
    'CreatedOn' : DateRange,
  });
  const FilterType = IDL.Variant({ 'Or' : IDL.Null, 'And' : IDL.Null });
  const Result_5 = IDL.Variant({
    'Ok' : IDL.Tuple(IDL.Principal, Privacy),
    'Err' : ApiError,
  });
  const SortDirection = IDL.Variant({ 'Asc' : IDL.Null, 'Desc' : IDL.Null });
  const GroupSort = IDL.Variant({
    'UpdatedOn' : SortDirection,
    'MemberCount' : SortDirection,
    'Name' : SortDirection,
    'CreatedOn' : SortDirection,
  });
  const PagedResponse = IDL.Record({
    'total' : IDL.Nat64,
    'data' : IDL.Vec(GroupResponse),
    'page' : IDL.Nat64,
    'limit' : IDL.Nat64,
    'number_of_pages' : IDL.Nat64,
  });
  const Result_6 = IDL.Variant({ 'Ok' : PagedResponse, 'Err' : ApiError });
  const Result_7 = IDL.Variant({
    'Ok' : IDL.Vec(GroupResponse),
    'Err' : ApiError,
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
  const Result_8 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Bool });
  return IDL.Service({
    '__get_candid_interface_tmp_hack' : IDL.Func([], [IDL.Text], ['query']),
    'accept_cycles' : IDL.Func([], [IDL.Nat64], []),
    'add_entry_by_parent' : IDL.Func([IDL.Vec(IDL.Nat8)], [Result], []),
    'add_group' : IDL.Func(
        [PostGroup, IDL.Principal, IDL.Opt(IDL.Text)],
        [Result_1],
        [],
      ),
    'add_role' : IDL.Func(
        [IDL.Principal, IDL.Text, IDL.Text, IDL.Nat64, IDL.Principal],
        [Result_2],
        [],
      ),
    'add_wallet' : IDL.Func(
        [IDL.Principal, IDL.Principal, IDL.Text],
        [Result],
        [],
      ),
    'clear_backup' : IDL.Func([], [], []),
    'delete_group' : IDL.Func([IDL.Principal, IDL.Principal], [Result_3], []),
    'download_chunk' : IDL.Func(
        [IDL.Nat64],
        [IDL.Tuple(IDL.Nat64, IDL.Vec(IDL.Nat8))],
        ['query'],
      ),
    'edit_group' : IDL.Func(
        [IDL.Principal, UpdateGroup, IDL.Principal],
        [Result_1],
        [],
      ),
    'edit_role_permissions' : IDL.Func(
        [IDL.Principal, IDL.Text, IDL.Vec(PostPermission), IDL.Principal],
        [Result_4],
        [],
      ),
    'finalize_upload' : IDL.Func([], [IDL.Text], []),
    'get_chunked_data' : IDL.Func(
        [IDL.Vec(GroupFilter), FilterType, IDL.Nat64, IDL.Nat64],
        [IDL.Vec(IDL.Nat8), IDL.Tuple(IDL.Nat64, IDL.Nat64)],
        ['query'],
      ),
    'get_group' : IDL.Func([IDL.Principal], [Result_1], ['query']),
    'get_group_owner_and_privacy' : IDL.Func(
        [IDL.Principal],
        [Result_5],
        ['query'],
      ),
    'get_group_roles' : IDL.Func(
        [IDL.Principal],
        [IDL.Vec(GroupRole)],
        ['query'],
      ),
    'get_groups' : IDL.Func(
        [
          IDL.Nat64,
          IDL.Nat64,
          IDL.Vec(GroupFilter),
          FilterType,
          GroupSort,
          IDL.Bool,
        ],
        [Result_6],
        ['query'],
      ),
    'get_groups_by_id' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [Result_7],
        ['query'],
      ),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'remove_role' : IDL.Func(
        [IDL.Principal, IDL.Text, IDL.Principal],
        [Result_4],
        [],
      ),
    'remove_wallet' : IDL.Func([IDL.Principal, IDL.Principal], [Result], []),
    'restore_data' : IDL.Func([], [], []),
    'total_chunks' : IDL.Func([], [IDL.Nat64], ['query']),
    'update_member_count' : IDL.Func(
        [IDL.Principal, IDL.Principal, IDL.Nat64],
        [Result_8],
        [],
      ),
    'upload_chunk' : IDL.Func(
        [IDL.Tuple(IDL.Nat64, IDL.Vec(IDL.Nat8))],
        [],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  return [IDL.Principal, IDL.Text, IDL.Nat64];
};
