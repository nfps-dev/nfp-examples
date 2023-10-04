#![allow(clippy::large_enum_variant)]

use cosmwasm_std::{Addr, Binary, Coin, Timestamp, Uint64};
use schemars::JsonSchema;
use secret_toolkit::permit::Permit;
use serde::{Deserialize, Serialize};

use crate::battleship::ListedGame;
use crate::expiration::Expiration;
use crate::mint_run::{MintRunInfo, SerialNumber};
use crate::nfp::RawData;
use crate::royalties::{DisplayRoyaltyInfo, RoyaltyInfo};
use crate::nfp::{PackageVersion, PackageVersionInfo};
use crate::snip52_signed_doc::SignedDocument;
use crate::token::{Extension, Metadata};

/// Instantiation message
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    /// name of token contract
    pub name: String,
    /// token contract symbol
    pub symbol: String,
    /// optional admin address, info.sender if missing
    pub admin: Option<String>,
    /// entropy used for prng seed
    pub entropy: String,
    /// optional royalty information to use as default when RoyaltyInfo is not provided to a
    /// minting function
    pub royalty_info: Option<RoyaltyInfo>,
    /// optional privacy configuration for the contract
    pub config: Option<InstantiateConfig>,
    /// optional callback message to execute after instantiation.  This will
    /// most often be used to have the token contract provide its address to a
    /// contract that instantiated it, but it could be used to execute any
    /// contract
    pub post_init_callback: Option<PostInstantiateCallback>,

    /// Battleship template
    pub template: Option<String>,
}

/// This type represents optional configuration values.
/// All values are optional and have defaults which are more private by default,
/// but can be overridden if necessary
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct InstantiateConfig {
    /// indicates whether the token IDs and the number of tokens controlled by the contract are
    /// public.  If the token supply is private, only minters can view the token IDs and
    /// number of tokens controlled by the contract
    /// default: False
    pub public_token_supply: Option<bool>,
    /// indicates whether token ownership is public or private.  A user can still change whether the
    /// ownership of their tokens is public or private
    /// default: False
    pub public_owner: Option<bool>,
    /// indicates whether sealed metadata should be enabled.  If sealed metadata is enabled, the
    /// private metadata is not viewable by anyone, not even the owner, until the owner calls the
    /// Reveal function.  When Reveal is called, the sealed metadata is irreversibly moved to the
    /// public metadata (as default).  if unwrapped_metadata_is_private is set to true, it will
    /// remain as private metadata, but the owner will now be able to see it.  Anyone will be able
    /// to query the token to know that it has been unwrapped.  This simulates buying/selling a
    /// wrapped card that no one knows which card it is until it is unwrapped. If sealed metadata
    /// is not enabled, all tokens are considered unwrapped
    /// default:  False
    pub enable_sealed_metadata: Option<bool>,
    /// indicates if the Reveal function should keep the sealed metadata private after unwrapping
    /// This config value is ignored if sealed metadata is not enabled
    /// default: False
    pub unwrapped_metadata_is_private: Option<bool>,
    /// indicates whether a minter is permitted to update a token's metadata
    /// default: True
    pub minter_may_update_metadata: Option<bool>,
    /// indicates whether the owner of a token is permitted to update a token's metadata
    /// default: False
    pub owner_may_update_metadata: Option<bool>,
    /// Indicates whether burn functionality should be enabled
    /// default: False
    pub enable_burn: Option<bool>,

    /// NFP
    /// Indicates whether minter can modify token storage with `storage_token_put`
    /// default: False
    pub minter_may_put_token_storage: Option<bool>,
    /// Indicates whether minter can modify global storage with `storage_global_put`
    /// default: False
    pub minter_may_put_global_storage: Option<bool>,
}

impl Default for InstantiateConfig {
    fn default() -> Self {
        InstantiateConfig {
            public_token_supply: Some(false),
            public_owner: Some(false),
            enable_sealed_metadata: Some(false),
            unwrapped_metadata_is_private: Some(false),
            minter_may_update_metadata: Some(true),
            owner_may_update_metadata: Some(false),
            enable_burn: Some(false),

            //NFP
            minter_may_put_token_storage: Some(false),
            minter_may_put_global_storage: Some(false),
        }
    }
}

/// info needed to perform a callback message after instantiation
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct PostInstantiateCallback {
    /// the callback message to execute
    pub msg: Binary,
    /// address of the contract to execute
    pub contract_address: String,
    /// code hash of the contract to execute
    pub code_hash: String,
    /// list of native Coin to send with the callback message
    pub send: Vec<Coin>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// mint new token
    MintNft {
        /// optional token id. if omitted, use current token index
        token_id: Option<String>,
        /// optional owner address. if omitted, owned by the message sender
        owner: Option<String>,
        /// optional public metadata that can be seen by everyone
        public_metadata: Option<Metadata>,
        /// optional private metadata that can only be seen by the owner and whitelist
        private_metadata: Option<Metadata>,
        /// optional serial number for this token
        serial_number: Option<SerialNumber>,
        /// optional royalty information for this token.  This will be ignored if the token is
        /// non-transferable
        royalty_info: Option<RoyaltyInfo>,
        /// optionally true if the token is transferable.  Defaults to true if omitted
        transferable: Option<bool>,
        /// optional memo for the tx
        memo: Option<String>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// set the public and/or private metadata.  This can be called by either the token owner or
    /// a valid minter if they have been given this power by the appropriate config values
    SetMetadata {
        /// id of the token whose metadata should be updated
        token_id: String,
        /// the optional new public metadata
        public_metadata: Option<Metadata>,
        /// the optional new private metadata
        private_metadata: Option<Metadata>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// if a contract was instantiated to make ownership public by default, this will allow
    /// an address to make the ownership of their tokens private.  The address can still use
    /// SetGlobalApproval to make ownership public either inventory-wide or for a specific token
    MakeOwnershipPrivate {
        /// optional message length padding
        padding: Option<String>,
    },
    /// add/remove approval(s) for a specific address on the token(s) you own.  Any permissions
    /// that are omitted will keep the current permission setting for that whitelist address
    SetWhitelistedApproval {
        /// address being granted/revoked permission
        address: String,
        /// optional token id to apply approval/revocation to
        token_id: Option<String>,
        /// optional permission level for viewing the owner
        view_owner: Option<AccessLevel>,
        /// optional permission level for viewing private metadata
        view_private_metadata: Option<AccessLevel>,
        /// optional permission level for transferring
        transfer: Option<AccessLevel>,
        /// optional expiration
        expires: Option<Expiration>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// gives the spender permission to transfer the specified token.  If you are the owner
    /// of the token, you can use SetWhitelistedApproval to accomplish the same thing.  If
    /// you are an operator, you can only use Approve
    Approve {
        /// address being granted the permission
        spender: String,
        /// id of the token that the spender can transfer
        token_id: String,
        /// optional expiration for this approval
        expires: Option<Expiration>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// revokes the spender's permission to transfer the specified token.  If you are the owner
    /// of the token, you can use SetWhitelistedApproval to accomplish the same thing.  If you
    /// are an operator, you can only use Revoke, but you can not revoke the transfer approval
    /// of another operator
    Revoke {
        /// address whose permission is revoked
        spender: String,
        /// id of the token that the spender can no longer transfer
        token_id: String,
        /// optional message length padding
        padding: Option<String>,
    },
    /// provided for cw721 compliance, but can be done with SetWhitelistedApproval...
    /// gives the operator permission to transfer all of the message sender's tokens
    ApproveAll {
        /// address being granted permission to transfer
        operator: String,
        /// optional expiration for this approval
        expires: Option<Expiration>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// provided for cw721 compliance, but can be done with SetWhitelistedApproval...
    /// revokes the operator's permission to transfer any of the message sender's tokens
    RevokeAll {
        /// address whose permissions are revoked
        operator: String,
        /// optional message length padding
        padding: Option<String>,
    },
    /// transfer a token if it is transferable
    TransferNft {
        /// recipient of the transfer
        recipient: String,
        /// id of the token to transfer
        token_id: String,
        /// optional memo for the tx
        memo: Option<String>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// transfer many tokens and fails if any are non-transferable
    BatchTransferNft {
        /// list of transfers to perform
        transfers: Vec<Transfer>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// send a token if it is transferable and call the receiving contract's (Batch)ReceiveNft
    SendNft {
        /// address to send the token to
        contract: String,
        /// optional code hash and BatchReceiveNft implementation status of the recipient contract
        receiver_info: Option<ReceiverInfo>,
        /// id of the token to send
        token_id: String,
        /// optional message to send with the (Batch)RecieveNft callback
        msg: Option<Binary>,
        /// optional memo for the tx
        memo: Option<String>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// send many tokens and call the receiving contracts' (Batch)ReceiveNft.  Fails if any tokens are
    /// non-transferable
    BatchSendNft {
        /// list of sends to perform
        sends: Vec<Send>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// register that the message sending contract implements ReceiveNft and possibly
    /// BatchReceiveNft.  If a contract implements BatchReceiveNft, SendNft will always
    /// call BatchReceiveNft even if there is only one token transferred (the token_ids
    /// Vec will only contain one ID)
    RegisterReceiveNft {
        /// receving contract's code hash
        code_hash: String,
        /// optionally true if the contract also implements BatchReceiveNft.  Defaults
        /// to false if not specified
        also_implements_batch_receive_nft: Option<bool>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// create a viewing key
    CreateViewingKey {
        /// entropy String used in random key generation
        entropy: String,
        /// optional message length padding
        padding: Option<String>,
    },
    /// set viewing key
    SetViewingKey {
        /// desired viewing key
        key: String,
        /// optional message length padding
        padding: Option<String>,
    },
    /// add addresses with minting authority
    AddMinters {
        /// list of addresses that can now mint
        minters: Vec<String>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// revoke minting authority from addresses
    RemoveMinters {
        /// list of addresses no longer allowed to mint
        minters: Vec<String>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// define list of addresses with minting authority
    SetMinters {
        /// list of addresses with minting authority
        minters: Vec<String>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// change address with administrative power
    ChangeAdmin {
        /// address with admin authority
        address: String,
        /// optional message length padding
        padding: Option<String>,
    },
    /// set contract status level to determine which functions are allowed.  StopTransactions
    /// status prevent mints, burns, sends, and transfers, but allows all other functions
    SetContractStatus {
        /// status level
        level: ContractStatus,
        /// optional message length padding
        padding: Option<String>,
    },
    /// disallow the use of a permit
    RevokePermit {
        /// name of the permit that is no longer valid
        permit_name: String,
        /// optional message length padding
        padding: Option<String>,
    },

    /// Standard NFP funcs
    /// 
    /// set a key-value pair in owner storage
    ///    callable by anyone (as 'owner')
    StorageOwnerPut {
        /// vector of key, value pairs to set
        data: Vec<KeyValuePair>,
        /// optional owner (in case called by delegate)
        ///   if both `owner` and `token_id` are Some it will throw an error
        owner: Option<String>,
        /// optional token id (in case called by delegate of current owner of token id)
        token_id: Option<String>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// sets key-value pairs in token storage
    ///    only callable by admin 
    StorageTokenPut {
        /// vector of key, value pairs to set
        data: Vec<KeyValuePair>,
        /// id of token
        token_id: String,
        /// optional message length padding
        padding: Option<String>,
    },
    /// sets key-value pairs in global storage
    ///    only callable by admin 
    StorageGlobalPut {
        /// vector of key, value pairs to set
        data: Vec<KeyValuePair>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// Approves a given address to perform ApplicationActions on behalf 
    /// of the transaction’s sender for ANY TOKEN owned by that sender.
    ApproveOwnerDelegate {
        /// address being given owner delegate approval
        address: String,
        /// optional message length padding
        padding: Option<String>,
    },
    /// Approves a given address to perform ApplicationActions on behalf 
    /// of the transaction’s sender for a particular token, as long as it 
    /// is owned by that sender.
    ApproveTokenDelegate {
        /// address being given owner delegate approval
        address: String,
        /// token ids being given approval for
        token_ids: Vec<String>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// Revokes a given delegate by address, regardless of whether it 
    /// delegates for owner or token(s). In other words, this will undo 
    /// all previous ApproveOwnerDelegate and ApproveTokenDelegate 
    /// executions that were made by the sender for the given delegate address.
    RevokeDelegate {
        /// address being revoked delegate for sender
        address: String,
        /// optional message length padding
        padding: Option<String>,
    },
    /// Revokes all delegates previously approved by the transaction sender.
    RevokeAllDelegates {
        /// optional message length padding
        padding: Option<String>,
    },
    /// adds a code version for a package
    ///   admin-only function
    UploadPackageVersion {
        /// package id
        ///   this is the id shared by all versions
        package_id: String,
        /// script data
        data: RawData,
        /// tags to set to this package, must not already be in use for the package
        tags: Option<Vec<String>>,
        /// optional metadata
        metadata: Option<String>,
        /// indicates if package version access is public, owners, or cleared
        access: String,
        /// optional message length padding
        padding: Option<String>,
    },
    /// adds tags to an existing version of a package
    AddPackageTags {
        /// package id
        ///   this is the id shared by all versions
        package_id: String,
        /// index of the version you want to add tags to
        index: u32,
        /// tags to set to this package, must not already be in use for the package
        tags: Vec<String>,
        /// optional message length padding
        padding: Option<String>,
    },

    ///
    /// Battleship functions
    /// 
    /// Creates a new listed game in the lobby
    NewGame {
        token_id: String,
        title: String,
        padding: Option<String>,
    },
    
    /// Joins a new game that is currently waiting for another player
    JoinGame {
        token_id: String,
        game_id: String,
        padding: Option<String>,
    },
    
    /// Player submits their board setup
    SubmitSetup {
        token_id: String,
        game_id: String,
        cells: Vec<u8>,
        padding: Option<String>,
    },
    
    /// Player submits their move attacking an opponent's cell `w = x + (y * 10)` where w is in [0,99]
    AttackCell {
        token_id: String,
        game_id: String,
        cell: u8,
        padding: Option<String>,
    },
    
    /// Allows a player to claim victory once their opponent has exceeded their turn timer
    ClaimVictory {
        token_id: String,
        game_id: String,
        padding: Option<String>,
    },

    /// SNIP-52
    /// Updates the seed with a new document signature
    UpdateSeed {
        /// signed doc
        signed_doc: SignedDocument,
        /// optional message length padding
        padding: Option<String>,
    },
}

/// permission access level
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AccessLevel {
    /// approve permission only for the specified token
    ApproveToken,
    /// grant permission for all tokens
    All,
    /// revoke permission only for the specified token
    RevokeToken,
    /// remove all permissions for this address
    None,
}

/// token mint info used when doing a BatchMint
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Mint {
    /// optional token id, if omitted, use current token index
    pub token_id: Option<String>,
    /// optional owner address, owned by the minter otherwise
    pub owner: Option<String>,
    /// optional public metadata that can be seen by everyone
    pub public_metadata: Option<Metadata>,
    /// optional private metadata that can only be seen by owner and whitelist
    pub private_metadata: Option<Metadata>,
    /// optional serial number for this token
    pub serial_number: Option<SerialNumber>,
    /// optional royalty information for this token.  This will be ignored if the token is
    /// non-transferable
    pub royalty_info: Option<RoyaltyInfo>,
    /// optionally true if the token is transferable.  Defaults to true if omitted
    pub transferable: Option<bool>,
    /// optional memo for the tx
    pub memo: Option<String>,
}

/// token transfer info used when doing a BatchTransferNft
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Transfer {
    /// recipient of the transferred tokens
    pub recipient: String,
    /// tokens being transferred
    pub token_ids: Vec<String>,
    /// optional memo for the tx
    pub memo: Option<String>,
}

/// send token info used when doing a BatchSendNft
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Send {
    /// recipient of the sent tokens
    pub contract: String,
    /// optional code hash and BatchReceiveNft implementation status of the recipient contract
    pub receiver_info: Option<ReceiverInfo>,
    /// tokens being sent
    pub token_ids: Vec<String>,
    /// optional message to send with the (Batch)RecieveNft callback
    pub msg: Option<Binary>,
    /// optional memo for the tx
    pub memo: Option<String>,
}

/// Storage key value pair
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct KeyValuePair {
    /// key
    pub key: String,
    /// value
    pub value: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteAnswer {
    /// MintNft will also display the minted token's ID in the log attributes under the
    /// key `minted` in case minting was done as a callback message
    MintNft {
        token_id: String,
    },
    SetMetadata {
        status: ResponseStatus,
    },
    MakeOwnershipPrivate {
        status: ResponseStatus,
    },
    Approve {
        status: ResponseStatus,
    },
    Revoke {
        status: ResponseStatus,
    },
    ApproveAll {
        status: ResponseStatus,
    },
    RevokeAll {
        status: ResponseStatus,
    },
    SetWhitelistedApproval {
        status: ResponseStatus,
    },
    TransferNft {
        status: ResponseStatus,
    },
    BatchTransferNft {
        status: ResponseStatus,
    },
    SendNft {
        status: ResponseStatus,
    },
    BatchSendNft {
        status: ResponseStatus,
    },
    RegisterReceiveNft {
        status: ResponseStatus,
    },
    /// response from both setting and creating a viewing key
    ViewingKey {
        key: String,
    },
    AddMinters {
        status: ResponseStatus,
    },
    RemoveMinters {
        status: ResponseStatus,
    },
    SetMinters {
        status: ResponseStatus,
    },
    ChangeAdmin {
        status: ResponseStatus,
    },
    SetContractStatus {
        status: ResponseStatus,
    },
    RevokePermit {
        status: ResponseStatus,
    },

    /// Standard NFP funcs
    StorageOwnerPut {
        status: ResponseStatus,
    },
    StorageTokenPut {
        status: ResponseStatus,
    },
    StorageGlobalPut {
        status: ResponseStatus,
    },
    UploadPackageVersion {
        index: u32,
    },
    AddPackageTags {
        status: ResponseStatus,
    },
    ApproveOwnerDelegate {
        status: ResponseStatus,
    },
    ApproveTokenDelegate {
        status: ResponseStatus,
    },
    RevokeDelegate {
        status: ResponseStatus,
    },
    RevokeAllDelegates {
        status: ResponseStatus,
    },

    /// Battleship
    /// Creates a new listed game in the lobby
    NewGame {
        game: ListedGame,
    },
    
    /// Joins a new game that is currently waiting for another player
    JoinGame {
        status: ResponseStatus,
    },
    
    /// Player submits their board setup
    SubmitSetup {
        status: ResponseStatus,
    },
    
    /// Player submits their move attacking an opponent's cell `w = x + (y * 10)` where w is in [0,99]
    AttackCell {
        away: Vec<u8>,
        turn: u8,
    },
    
    /// Allows a player to claim victory once their opponent has exceeded their turn timer
    ClaimVictory {
        status: ResponseStatus,
    },

    ///SNIP-52
    UpdateSeed {
        seed: Binary,
    },
}

/// the address and viewing key making an authenticated query request
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ViewerInfo {
    /// querying address
    pub address: String,
    /// authentication key string
    pub viewing_key: String,
}

/// the address and viewing key making an authenticated query request
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ViewerInfoAddrOpt {
    /// querying address
    pub address: Option<String>,
    /// authentication key string
    pub viewing_key: String,
}

/// a recipient contract's code hash and whether it implements BatchReceiveNft
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ReceiverInfo {
    /// recipient's code hash
    pub recipient_code_hash: String,
    /// true if the contract also implements BacthReceiveNft.  Defaults to false
    /// if not specified
    pub also_implements_batch_receive_nft: Option<bool>,
}

/// tx type and specifics
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TxAction {
    /// transferred token ownership
    Transfer {
        /// previous owner
        from: Addr,
        /// optional sender if not owner
        sender: Option<Addr>,
        /// new owner
        recipient: Addr,
    },
    /// minted new token
    Mint {
        /// minter's address
        minter: Addr,
        /// token's first owner
        recipient: Addr,
    },
    /// burned a token
    Burn {
        /// previous owner
        owner: Addr,
        /// burner's address if not owner
        burner: Option<Addr>,
    },
}

/// tx for display
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Tx {
    /// tx id
    pub tx_id: u64,
    /// the block containing this tx
    pub block_height: u64,
    /// the time (in seconds since 01/01/1970) of the block containing this tx
    pub block_time: u64,
    /// token id
    pub token_id: String,
    /// tx type and specifics
    pub action: TxAction,
    /// optional memo
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// display the contract's name and symbol
    ContractInfo {},
    /// display the contract's configuration
    ContractConfig {},
    /// display the list of authorized minters
    Minters {},
    /// display the number of tokens controlled by the contract.  The token supply must
    /// either be public, or the querier must be an authenticated minter
    NumTokens {
        /// optional address and key requesting to view the number of tokens
        viewer: Option<ViewerInfo>,
    },
    /// display an optionally paginated list of all the tokens controlled by the contract.
    /// The token supply must either be public, or the querier must be an authenticated
    /// minter
    AllTokens {
        /// optional address and key requesting to view the list of tokens
        viewer: Option<ViewerInfo>,
        /// paginate by providing the last token_id received in the previous query
        start_after: Option<String>,
        /// optional number of token ids to display
        limit: Option<u32>,
    },
    /// display the owner of the specified token if authorized to view it.  If the requester
    /// is also the token's owner, the response will also include a list of any addresses
    /// that can transfer this token.  The transfer approval list is for CW721 compliance,
    /// but the NftDossier query will be more complete by showing viewing approvals as well
    OwnerOf {
        token_id: String,
        /// optional address and key requesting to view the token owner
        viewer: Option<ViewerInfo>,
        /// optionally include expired Approvals in the response list.  If ommitted or
        /// false, expired Approvals will be filtered out of the response
        include_expired: Option<bool>,
    },
    /// displays the public metadata of a token
    NftInfo { token_id: String },
    /// displays all the information contained in the OwnerOf and NftInfo queries
    AllNftInfo {
        token_id: String,
        /// optional address and key requesting to view the token owner
        viewer: Option<ViewerInfo>,
        /// optionally include expired Approvals in the response list.  If ommitted or
        /// false, expired Approvals will be filtered out of the response
        include_expired: Option<bool>,
    },
    /// displays the private metadata if permitted to view it
    PrivateMetadata {
        token_id: String,
        /// optional address and key requesting to view the private metadata
        viewer: Option<ViewerInfo>,
    },
    /// displays all the information about a token that the viewer has permission to
    /// see.  This may include the owner, the public metadata, the private metadata, royalty
    /// information, mint run information, whether the token is unwrapped, whether the token is
    /// transferable, and the token and inventory approvals
    NftDossier {
        token_id: String,
        /// optional address and key requesting to view the token information
        viewer: Option<ViewerInfo>,
        /// optionally include expired Approvals in the response list.  If ommitted or
        /// false, expired Approvals will be filtered out of the response
        include_expired: Option<bool>,
    },
    /// displays all the information about multiple tokens that the viewer has permission to
    /// see.  This may include the owner, the public metadata, the private metadata, royalty
    /// information, mint run information, whether the token is unwrapped, whether the token is
    /// transferable, and the token and inventory approvals
    BatchNftDossier {
        token_ids: Vec<String>,
        /// optional address and key requesting to view the token information
        viewer: Option<ViewerInfo>,
        /// optionally include expired Approvals in the response list.  If ommitted or
        /// false, expired Approvals will be filtered out of the response
        include_expired: Option<bool>,
    },
    /// list all the approvals in place for a specified token if given the owner's viewing
    /// key
    TokenApprovals {
        token_id: String,
        /// the token owner's viewing key
        viewing_key: String,
        /// optionally include expired Approvals in the response list.  If ommitted or
        /// false, expired Approvals will be filtered out of the response
        include_expired: Option<bool>,
    },
    /// list all the inventory-wide approvals in place for the specified address if given the
    /// the correct viewing key for the address
    InventoryApprovals {
        address: String,
        /// the viewing key
        viewing_key: String,
        /// optionally include expired Approvals in the response list.  If ommitted or
        /// false, expired Approvals will be filtered out of the response
        include_expired: Option<bool>,
    },
    /// displays a list of all the CW721-style operators (any address that was granted
    /// approval to transfer all of the owner's tokens).  This query is provided to maintain
    /// CW721 compliance, however, approvals are private on secret network, so only the
    /// owner's viewing key will authorize the ability to see the list of operators
    ApprovedForAll {
        owner: String,
        /// optional viewing key to authenticate this query.  It is "optional" only in the
        /// sense that a CW721 query does not have this field.  However, not providing the
        /// key will always result in an empty list
        viewing_key: Option<String>,
        /// optionally include expired Approvals in the response list.  If ommitted or
        /// false, expired Approvals will be filtered out of the response
        include_expired: Option<bool>,
    },
    /// displays a list of all the tokens belonging to the input owner in which the viewer
    /// has view_owner permission
    Tokens {
        owner: String,
        /// optional address of the querier if different from the owner
        viewer: Option<String>,
        /// optional viewing key
        viewing_key: Option<String>,
        /// paginate by providing the last token_id received in the previous query
        start_after: Option<String>,
        /// optional number of token ids to display
        limit: Option<u32>,
    },
    /// displays the number of tokens that the querier has permission to see the owner and that
    /// belong to the specified address
    NumTokensOfOwner {
        owner: String,
        /// optional address of the querier if different from the owner
        viewer: Option<String>,
        /// optional viewing key
        viewing_key: Option<String>,
    },
    /// display the transaction history for the specified address in reverse
    /// chronological order
    TransactionHistory {
        address: String,
        /// viewing key
        viewing_key: String,
        /// optional page to display
        page: Option<u32>,
        /// optional number of transactions per page
        page_size: Option<u32>,
    },
    /// display the code hash a contract has registered with the token contract and whether
    /// the contract implements BatchReceivenft
    RegisteredCodeHash {
        /// the contract whose receive registration info you want to view
        contract: String,
    },
    /// display the contract's creator
    ContractCreator {},

    /// NFP
    /// get values from owner storage
    StorageOwnerGet {
        keys: Vec<String>,
        viewer: ViewerInfo,
    },
    /// get values from token storage
    StorageTokenGet {
        keys: Vec<String>,
        token_id: String,
        viewer: ViewerInfo,
    },
    /// get values from global storage
    StorageGlobalGet {
        keys: Vec<String>,
    },
    /// get paginated owner delegate approvals for given viewer
    OwnerDelegateApprovals {
        /// page
        page: Option<u32>,
        /// page size
        page_size: Option<u32>,
        /// viewer
        viewer: ViewerInfo,
    },
    /// get paginated token delegate approvals for given viewer
    TokenDelegateApprovals {
        /// token id
        token_id: String,
        /// viewer
        viewer: ViewerInfo,
    },
    /// get a package version by tag or version (index)
    PackageVersion {
        /// package id
        package_id: String,
        /// optional tag reference (must have tag or version index but not both)
        tag: Option<String>,
        /// index of the version
        index: Option<u32>,
        /// token id that address is owner of
        token_id: String,
        /// if address is not given in viewer, it will default to token_id's owner
        viewer: Option<ViewerInfoAddrOpt>,
    },
    /// get paginated info about the versions of a package
    PackageInfo {
        /// package id
        package_id: String,
        /// page
        page: Option<u32>,
        /// page size
        page_size: Option<u32>,
        /// optional viewer
        viewer: Option<ViewerInfo>,
    },

    ///
    /// Battleship queries
    /// 
    /// Fetches a list of active games in the lobby
    ListGames {
        page_size: Option<u32>,
        page: Option<u32>,
        token_id: String,
        viewer: ViewerInfo,
    },

    /// Gets the list of active games the token is party to
    ActiveGames {
        token_id: String,
        viewer: ViewerInfo,
    },
    
    /// Fetches the current game state
    GameState {
        token_id: String,
        game_id: String,
        viewer: ViewerInfo,
    },

    /// SNIP-52
    /// Public query to list all notification channels
    ListChannels {},
    /// Authenticated query allows clients to obtain the seed, counter, and 
    ///   Notification ID of a future event, for a specific channels.
    ChannelInfo {
        channels: Vec<String>,
        viewer: ViewerInfo,
    },

    /// perform queries by passing permits instead of viewing keys
    WithPermit {
        /// permit used to verify querier identity
        permit: Permit,
        /// query to perform
        query: QueryWithPermit,
    },
}

/// SNIP721 Approval
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Snip721Approval {
    /// whitelisted address
    pub address: Addr,
    /// optional expiration if the address has view owner permission
    pub view_owner_expiration: Option<Expiration>,
    /// optional expiration if the address has view private metadata permission
    pub view_private_metadata_expiration: Option<Expiration>,
    /// optional expiration if the address has transfer permission
    pub transfer_expiration: Option<Expiration>,
}

/// CW721 Approval
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Cw721Approval {
    /// address that can transfer the token
    pub spender: Addr,
    /// expiration of this approval
    pub expires: Expiration,
}

/// response of CW721 OwnerOf
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Cw721OwnerOfResponse {
    /// Owner of the token if permitted to view it
    pub owner: Option<Addr>,
    /// list of addresses approved to transfer this token
    pub approvals: Vec<Cw721Approval>,
}

/// the token id and nft dossier info of a single token response in a batch query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct BatchNftDossierElement {
    pub token_id: String,
    pub owner: Option<Addr>,
    pub public_metadata: Option<Metadata>,
    pub private_metadata: Option<Metadata>,
    pub display_private_metadata_error: Option<String>,
    pub royalty_info: Option<DisplayRoyaltyInfo>,
    pub mint_run_info: Option<MintRunInfo>,
    /// true if this token is transferable
    pub transferable: bool,
    /// true if this token is unwrapped (returns true if the contract does not have selaed metadata enabled)
    pub unwrapped: bool,
    pub owner_is_public: bool,
    pub public_ownership_expiration: Option<Expiration>,
    pub private_metadata_is_public: bool,
    pub private_metadata_is_public_expiration: Option<Expiration>,
    pub token_approvals: Option<Vec<Snip721Approval>>,
    pub inventory_approvals: Option<Vec<Snip721Approval>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct TokenDelegateApproval {
    pub address: String,
    pub tokens: Vec<String>,
}

/// channel mode
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChannelMode {
    Counter,
    Txhash,
}

/// Channel info struct
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct ChannelInfo {
    /// shared secret in base64
    pub seed: Binary,
    /// same as query input
    pub channel: String,
    /// either "counter" or "txhash"
    pub mode: ChannelMode,
    /// current counter value
    pub counter: Uint64,
    /// the next Notification ID
    pub next_id: Binary,
    /// optional CDDL schema definition string for the CBOR-encoded notification data
    pub cddl: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    ContractInfo {
        name: String,
        symbol: String,
    },
    ContractConfig {
        token_supply_is_public: bool,
        owner_is_public: bool,
        sealed_metadata_is_enabled: bool,
        unwrapped_metadata_is_private: bool,
        minter_may_update_metadata: bool,
        owner_may_update_metadata: bool,
        burn_is_enabled: bool,
        implements_non_transferable_tokens: bool,
        implements_token_subtype: bool,
    },
    Minters {
        minters: Vec<Addr>,
    },
    NumTokens {
        count: u32,
    },
    TokenList {
        tokens: Vec<String>,
    },
    OwnerOf {
        owner: Addr,
        approvals: Vec<Cw721Approval>,
    },
    TokenApprovals {
        owner_is_public: bool,
        public_ownership_expiration: Option<Expiration>,
        private_metadata_is_public: bool,
        private_metadata_is_public_expiration: Option<Expiration>,
        token_approvals: Vec<Snip721Approval>,
    },
    InventoryApprovals {
        owner_is_public: bool,
        public_ownership_expiration: Option<Expiration>,
        private_metadata_is_public: bool,
        private_metadata_is_public_expiration: Option<Expiration>,
        inventory_approvals: Vec<Snip721Approval>,
    },
    NftInfo {
        token_uri: Option<String>,
        extension: Option<Extension>,
    },
    PrivateMetadata {
        token_uri: Option<String>,
        extension: Option<Extension>,
    },
    AllNftInfo {
        access: Cw721OwnerOfResponse,
        info: Option<Metadata>,
    },
    NftDossier {
        owner: Option<Addr>,
        public_metadata: Option<Metadata>,
        private_metadata: Option<Metadata>,
        display_private_metadata_error: Option<String>,
        royalty_info: Option<DisplayRoyaltyInfo>,
        mint_run_info: Option<MintRunInfo>,
        transferable: bool,
        unwrapped: bool,
        owner_is_public: bool,
        public_ownership_expiration: Option<Expiration>,
        private_metadata_is_public: bool,
        private_metadata_is_public_expiration: Option<Expiration>,
        token_approvals: Option<Vec<Snip721Approval>>,
        inventory_approvals: Option<Vec<Snip721Approval>>,
    },
    BatchNftDossier {
        nft_dossiers: Vec<BatchNftDossierElement>,
    },
    ApprovedForAll {
        operators: Vec<Cw721Approval>,
    },
    TransactionHistory {
        /// total transaction count
        total: u64,
        txs: Vec<Tx>,
    },
    RegisteredCodeHash {
        code_hash: Option<String>,
        also_implements_batch_receive_nft: bool,
    },
    ContractCreator {
        creator: Option<Addr>,
    },
    // NFP
    StorageOwnerGet {
        data: Vec<KeyValuePair>,
    },
    StorageTokenGet {
        data: Vec<KeyValuePair>,
    },
    StorageGlobalGet {
        data: Vec<KeyValuePair>,
    },
    OwnerDelegateApprovals {
        addresses: Vec<String>,
    },
    TokenDelegateApprovals {
        addresses: Vec<String>,
    },
    PackageVersion {
        package: Option<PackageVersion>,
    },
    PackageInfo {
        info: Vec<PackageVersionInfo>,
        version_count: u32,
    },

    // Battleship query answers
    /// Fetches a list of active games in the lobby
    ListGames {
        games: Vec<ListedGame>,
    },

    /// Gets the list of active games the token is party to
    ActiveGames {
        game_ids: Vec<String>,
    },
    
    /// Fetches the current game state
    GameState {
        game_id: String,
        wager: Coin,
        title: String,
        created: Timestamp,
        // 
        role: u8,
        turn: u8,
        home: Vec<u8>,
        away: Vec<u8>,
    },

    /// SNIP-52
    ListChannels {
        channels: Vec<String>,
    },
    ChannelInfo {
        /// scopes validity of this response
        as_of_block: Uint64,
        channels: Vec<ChannelInfo>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatus {
    Normal,
    StopTransactions,
    StopAll,
}

impl ContractStatus {
    /// Returns u8 representation of the ContractStatus
    pub fn to_u8(&self) -> u8 {
        match self {
            ContractStatus::Normal => 0,
            ContractStatus::StopTransactions => 1,
            ContractStatus::StopAll => 2,
        }
    }
}

/// queries using permits instead of viewing keys
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryWithPermit {
    /// displays the private metadata if permitted to view it
    PrivateMetadata { token_id: String },
    /// displays all the information about a token that the viewer has permission to
    /// see.  This may include the owner, the public metadata, the private metadata, royalty
    /// information, mint run information, whether the token is unwrapped, whether the token is
    /// transferable, and the token and inventory approvals
    NftDossier {
        token_id: String,
        /// optionally include expired Approvals in the response list.  If ommitted or
        /// false, expired Approvals will be filtered out of the response
        include_expired: Option<bool>,
    },
    /// displays all the information about multiple tokens that the viewer has permission to
    /// see.  This may include the owner, the public metadata, the private metadata, royalty
    /// information, mint run information, whether the token is unwrapped, whether the token is
    /// transferable, and the token and inventory approvals
    BatchNftDossier {
        token_ids: Vec<String>,
        /// optionally include expired Approvals in the response list.  If ommitted or
        /// false, expired Approvals will be filtered out of the response
        include_expired: Option<bool>,
    },
    /// display the owner of the specified token if authorized to view it.  If the requester
    /// is also the token's owner, the response will also include a list of any addresses
    /// that can transfer this token.  The transfer approval list is for CW721 compliance,
    /// but the NftDossier query will be more complete by showing viewing approvals as well
    OwnerOf {
        token_id: String,
        /// optionally include expired Approvals in the response list.  If ommitted or
        /// false, expired Approvals will be filtered out of the response
        include_expired: Option<bool>,
    },
    /// displays all the information contained in the OwnerOf and NftInfo queries
    AllNftInfo {
        token_id: String,
        /// optionally include expired Approvals in the response list.  If ommitted or
        /// false, expired Approvals will be filtered out of the response
        include_expired: Option<bool>,
    },
    /// list all the inventory-wide approvals in place for the permit creator
    InventoryApprovals {
        /// optionally include expired Approvals in the response list.  If ommitted or
        /// false, expired Approvals will be filtered out of the response
        include_expired: Option<bool>,
    },
    /// display the transaction history for the permit creator in reverse
    /// chronological order
    TransactionHistory {
        /// optional page to display
        page: Option<u32>,
        /// optional number of transactions per page
        page_size: Option<u32>,
    },
    /// display the number of tokens controlled by the contract.  The token supply must
    /// either be public, or the querier must be an authenticated minter
    NumTokens {},
    /// display an optionally paginated list of all the tokens controlled by the contract.
    /// The token supply must either be public, or the querier must be an authenticated
    /// minter
    AllTokens {
        /// paginate by providing the last token_id received in the previous query
        start_after: Option<String>,
        /// optional number of token ids to display
        limit: Option<u32>,
    },
    /// list all the approvals in place for a specified token if given the owner's permit
    TokenApprovals {
        token_id: String,
        /// optionally include expired Approvals in the response list.  If ommitted or
        /// false, expired Approvals will be filtered out of the response
        include_expired: Option<bool>,
    },
    /// displays a list of all the CW721-style operators (any address that was granted
    /// approval to transfer all of the owner's tokens).  This query is provided to maintain
    /// CW721 compliance
    ApprovedForAll {
        /// optionally include expired Approvals in the response list.  If ommitted or
        /// false, expired Approvals will be filtered out of the response
        include_expired: Option<bool>,
    },
    /// displays a list of all the tokens belonging to the input owner in which the permit
    /// creator has view_owner permission
    Tokens {
        owner: String,
        /// paginate by providing the last token_id received in the previous query
        start_after: Option<String>,
        /// optional number of token ids to display
        limit: Option<u32>,
    },
    /// displays the number of tokens that the querier has permission to see the owner and that
    /// belong to the specified address
    NumTokensOfOwner { owner: String },
    /// NFP
    /// get values from owner storage
    /// only readable by owner (sender)
    StorageOwnerGet {
        keys: Vec<String>,
    },
    /// get values from token storage
    /// only readable by token owner
    StorageTokenGet {
        keys: Vec<String>,
        token_id: String,
    },
    /// get paginated owner delegate approvals for given owner
    OwnerDelegateApprovals {
        /// page
        page: Option<u32>,
        /// page size
        page_size: Option<u32>,
    },
    /// get paginated token delegate approvals for given token
    TokenDelegateApprovals {
        /// token id
        token_id: String,
    },
    /// get a package version by tag or version (index)
    PackageVersion {
        /// package id
        package_id: String,
        /// optional tag reference (must have tag or version index but not both)
        tag: Option<String>,
        /// index of the version
        index: Option<u32>,
        /// token id that address is owner of
        token_id: String,
    },
    /// get paginated info about the versions of a package
    PackageInfo {
        /// package id
        package_id: String,
        /// page
        page: Option<u32>,
        /// page size
        page_size: Option<u32>,
    },

    ///
    /// Battleship queries
    /// 
    /// Fetches a list of active games in the lobby
    ListGames {
        page_size: Option<u32>,
        page: Option<u32>,
        token_id: String,
    },

    /// Gets the list of active games the token is party to
    ActiveGames {
        token_id: String,
    },
    
    /// Fetches the current game state
    GameState {
        token_id: String,
        game_id: String,
    },

    /// SNIP-52
    ChannelInfo {
        channels: Vec<String>,
    },
}
