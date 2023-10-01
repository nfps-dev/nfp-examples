use std::collections::HashSet;
use schemars::JsonSchema;
use secret_toolkit::storage::{AppendStore, Keymap, Keyset};
use serde::{Deserialize, Serialize};
use cosmwasm_std::{StdResult, StdError, Storage, CanonicalAddr, Binary};

/// NFP prefixes
/// prefix for raw data storage
pub const PREFIX_RAW_DATA_CHUNK: &[u8] = b"chunks";
/// prefix for owner storage
pub const PREFIX_STORAGE_OWNER: &[u8] = b"storageowner";
/// prefix for token storage
pub const PREFIX_STORAGE_TOKEN: &[u8] = b"storagetoken";
/// prefix for global storage
pub const PREFIX_STORAGE_GLOBAL: &[u8] = b"storageglobal";
/// prefix for package manager
pub const PREFIX_PACKAGE_MANAGER: &[u8] = b"packagemanager";
/// prefix for package tags
pub const PREFIX_PACKAGE_TAGS: &[u8] = b"packagetags";
/// prefix for set of delegates that have ANY token delegation (for an owner)
pub const PREFIX_ANY_DELEGATES: &[u8] = b"anydel";
/// prefix for set of delegate address to owner address mappings
pub const PREFIX_ANY_DELEGATES_INVERSE: &[u8] = b"anydel-1";
/// prefix for map of delegates to token vec (for an owner)
pub const PREFIX_TOKEN_DELEGATES: &[u8] = b"tokdel";
/// prefix for set of delegate address to owner address + tokens mappings
pub const PREFIX_TOKEN_DELEGATES_INVERSE: &[u8] = b"tokendel-1";

/// raw data on chain
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, Debug, Default)]
pub struct RawData {
    /// data bytes in base64
    pub bytes: Binary,
    /// mime-type of the data, default image/svg+xml
    pub content_type: Option<String>,
    /// encoding of the data, default gzip
    pub content_encoding: Option<String>,
    /// optional metadata
    pub metadata: Option<String>,
}

impl RawData {
    pub fn into_stored(self) -> StdResult<StoredRawData> {
        Ok(StoredRawData {
            bytes: self.bytes.0,
            content_type: self.content_type,
            content_encoding: self.content_encoding,
            metadata: self.metadata,
        })
    }
}

/// stored raw data on chain
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, Debug, Default)]
pub struct StoredRawData {
    /// data bytes
    pub bytes: Vec<u8>,
    /// mime-type of the data, default image/svg+xml
    pub content_type: Option<String>,
    /// encoding of the data, default gzip
    pub content_encoding: Option<String>,
    /// optional metadata
    pub metadata: Option<String>,
}

impl StoredRawData {
    pub fn into_humanized(self) -> StdResult<RawData> {
        Ok(RawData {
            bytes: Binary::from(self.bytes),
            content_type: self.content_type,
            content_encoding: self.content_encoding,
            metadata: self.metadata,
        })
    }
}

// NFP state

pub const ACCESS_PUBLIC: u8 = 0;
pub const ACCESS_OWNERS: u8 = 1;
pub const ACCESS_CLEARED: u8 = 2;
pub const ACCESS_PUBLIC_STRING: &str = "public";
pub const ACCESS_OWNERS_STRING: &str = "owners";
pub const ACCESS_CLEARED_STRING: &str = "cleared";

// Token storage reserved keys
pub const KEY_CLEARED_PACKAGES: &str = "#!cleared";

fn access_val_to_string(access: u8) -> StdResult<String> {
    match access {
        ACCESS_PUBLIC => Ok(String::from(ACCESS_PUBLIC_STRING)),
        ACCESS_OWNERS => Ok(String::from(ACCESS_OWNERS_STRING)),
        ACCESS_CLEARED => Ok(String::from(ACCESS_CLEARED_STRING)),
        _ => { return Err(StdError::generic_err("Invalid access type for package")); }
    }
}

fn access_string_to_val(access: String) -> StdResult<u8> {
    match access.as_str() {
        ACCESS_PUBLIC_STRING => Ok(ACCESS_PUBLIC),
        ACCESS_OWNERS_STRING => Ok(ACCESS_OWNERS),
        ACCESS_CLEARED_STRING => Ok(ACCESS_CLEARED),
        _ => { return Err(StdError::generic_err("Invalid access type for package")); }
    }
}

/// a version of a package
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredPackageVersion {
    /// package data
    pub data: StoredRawData,
    /// tags for this package
    pub tags: Option<Vec<String>>,
    /// optional metadata
    pub metadata: Option<String>,
    /// indicates access type for package: 
    /// public = 0, owners = 1, cleared = 2
    pub access: u8,
}

impl StoredPackageVersion {
    pub fn into_humanized(self) -> StdResult<PackageVersion> {
        Ok(PackageVersion { 
            data: self.data.into_humanized()?, 
            tags: self.tags, 
            metadata: self.metadata,
            access: access_val_to_string(self.access)?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct PackageVersion {
    /// package data
    pub data: RawData,
    /// tags for this package
    pub tags: Option<Vec<String>>,
    /// optional metadata
    pub metadata: Option<String>,
    /// indicates access type for package: public, owners, cleared
    pub access: String,
}

impl PackageVersion {
    pub fn into_stored(self) -> StdResult<StoredPackageVersion> {
        Ok(StoredPackageVersion { 
            data: self.data.into_stored()?,
            tags: self.tags, 
            metadata: self.metadata, 
            access: access_string_to_val(self.access)?,
        })
    }
}

/// package version info without data
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct PackageVersionInfo {
    /// package version index in package appendstore
    pub index: u32,
    /// tags for this package
    pub tags: Option<Vec<String>>,
    /// optional metadata
    pub metadata: Option<String>,
    /// indicates if package version is public
    pub access: String,
}

/// packagemanager:[package-id] -> AppendStore<StoredRawData>
/// packagetags:[package-id] [tag] -> index of package in above AppendStore 
pub static PACKAGE_MANAGER_STORE: AppendStore<StoredPackageVersion> = AppendStore::new(PREFIX_PACKAGE_MANAGER);
pub static PACKAGE_TAGS_MAP: Keymap<String, u32> = Keymap::new(PREFIX_PACKAGE_TAGS);

pub fn store_package_version(
    storage: &mut dyn Storage,
    package_id: String,
    raw_data: RawData,
    tags: Option<Vec<String>>,
    metadata: Option<String>,
    access: String,
) -> StdResult<u32> {
    let package_store = PACKAGE_MANAGER_STORE.add_suffix(package_id.as_bytes());

    if package_store.get_len(storage)? > 0 {
        // make sure access is the same as first version
        let first_package = package_store.get_at(storage, 0)?;
        let access_to_match = first_package.into_humanized()?.access;
        if access != access_to_match {
            return Err(StdError::generic_err(format!("access of new package version MUST be {}", access_to_match)));
        }
    }

    package_store.push(storage, &PackageVersion {
        data: raw_data,
        tags: tags.clone(),
        metadata,
        access,
    }.into_stored()?)?;
    let last = package_store.get_len(storage)? - 1;

    if let Some(tags) = tags {
        tags.into_iter().try_for_each(|tag| {
            //if get_package_index_from_tag(storage, package_id.clone(), tag.clone()).is_some() {
            //    return Err(StdError::generic_err(format!("`{}` is an existing tag for package id {}", tag, package_id)));
            //}
            set_tag(storage, package_id.clone(), tag, last)?;
            Ok::<(), StdError>(())
        })?;
    }
    Ok(last)
}

pub fn add_tags_to_package_version(
    storage: &mut dyn Storage,
    package_id: String,
    index: u32,
    tags: Vec<String>,
) -> StdResult<()> {
    let package_store = PACKAGE_MANAGER_STORE.add_suffix(package_id.as_bytes());
    let mut package_version = package_store.get_at(storage, index)?;
    let old_tags = package_version.tags.unwrap_or(vec![]);
    package_version.tags = Some(old_tags.into_iter().chain(tags.clone().into_iter()).collect());
    package_store.set_at(storage, index, &package_version)?;

    tags.into_iter().try_for_each(|tag| {
        //if get_package_index_from_tag(storage, package_id.clone(), tag.clone()).is_some() {
        //    return Err(StdError::generic_err(format!("`{}` is an existing tag for package id {}", tag, package_id)));
        //}
        set_tag(storage, package_id.clone(), tag, index)?;
        Ok::<(), StdError>(())
    })?;
    Ok(())
}

pub fn get_package_versions_info(
    storage: &dyn Storage,
    package_id: String,
    page: u32,
    page_size: u32,
) -> StdResult<(Vec<PackageVersionInfo>, u32)> {
    let package_store = PACKAGE_MANAGER_STORE.add_suffix(package_id.as_bytes());
    let package_versions = package_store.paging(storage, page, page_size)?
        .into_iter()
        .enumerate()
        .map(|(idx, package)| 
            PackageVersionInfo {
                index: page * page_size + idx as u32,
                tags: package.tags,
                metadata: package.metadata,
                access: access_val_to_string(package.access).unwrap(),
            }
        )
        .collect();
    let len = package_store.get_len(storage)?;
    Ok((package_versions, len))
}

pub fn set_tag(
    storage: &mut dyn Storage,
    package_id: String,
    tag: String,
    version: u32,
) -> StdResult<()> {
    let tags_store = PACKAGE_TAGS_MAP.add_suffix(package_id.as_bytes());
    tags_store.insert(storage, &tag, &version)
}

pub fn get_package_index_from_tag(
    storage: &dyn Storage,
    package_id: String,
    tag: String,
) -> Option<u32> {
    let tags_store = PACKAGE_TAGS_MAP.add_suffix(package_id.as_bytes());
    tags_store.get(storage, &tag)
}

pub fn get_package_version_by_index(
    storage: &dyn Storage,
    package_id: String,
    index: u32,
) -> StdResult<PackageVersion> {
    let package_store = PACKAGE_MANAGER_STORE.add_suffix(package_id.as_bytes());
    package_store.get_at(storage, index)?.into_humanized()
}

pub fn get_latest_package_version(
    storage: &dyn Storage,
    package_id: String,
) -> StdResult<Option<PackageVersion>> {
    let package_store = PACKAGE_MANAGER_STORE.add_suffix(package_id.as_bytes());
    let len = package_store.get_len(storage)?;
    if len == 0 {
        return Ok(None);
    }
    Ok(Some(get_package_version_by_index(storage, package_id, len - 1)?))
}

pub fn get_package_version_by_tag(
    storage: &dyn Storage,
    package_id: String,
    tag: String,
) -> StdResult<Option<PackageVersion>> {
    let package_store = PACKAGE_MANAGER_STORE.add_suffix(package_id.as_bytes());
    if let Some(index) = get_package_index_from_tag(storage, package_id, tag) {
        let package_version = package_store.get_at(storage, index)?.into_humanized()?;
        return Ok(Some(package_version));
    }
    Ok(None)
}

/// set of addresses that are a delegate for ANY token for an owner
///   used with add_suffix(owner.as_slice())
pub static ANY_DELEGATES: Keyset<CanonicalAddr> = Keyset::new(PREFIX_ANY_DELEGATES);
/// set of delegate address to owner address mappings
///   used with add_suffix(delegate.as_slice())
//pub static ANY_DELEGATES_INVERSE: Keyset<CanonicalAddr> = Keyset::new(PREFIX_ANY_DELEGATES_INVERSE);
/// set of addresses that are a delegate for specific tokens for an owner
///   used with add_suffix(owner.as_slice())
pub static TOKEN_DELEGATES: Keymap<CanonicalAddr, HashSet<String>> = Keymap::new(PREFIX_TOKEN_DELEGATES);
/// set of mappings with all delegated addresses for a token
///   used with add_suffix(owner.as_slice())
pub static TOKEN_DELEGATES_INVERSE: Keymap<String, HashSet<CanonicalAddr>> = Keymap::new(PREFIX_TOKEN_DELEGATES_INVERSE);

pub fn add_any_delegate(
    storage: &mut dyn Storage,
    owner: CanonicalAddr,
    address: CanonicalAddr,
) -> StdResult<bool> {
    ANY_DELEGATES
        .add_suffix(owner.as_slice())
        .insert(storage, &address)
}

pub fn remove_any_delegate(
    storage: &mut dyn Storage,
    owner: CanonicalAddr,
    address: CanonicalAddr,
) -> StdResult<()> {
    ANY_DELEGATES
        .add_suffix(owner.as_slice())
        .remove(storage, &address)
}

pub fn remove_all_any_delegates(
    storage: &mut dyn Storage,
    owner: CanonicalAddr,
) -> StdResult<()> {
    let owner_key = owner.as_slice();
    let delegates: Vec<CanonicalAddr> = ANY_DELEGATES
        .add_suffix(owner_key)
        .iter(storage)?
        .map(|x| x.unwrap())
        .collect();
    for delegate in delegates {
        ANY_DELEGATES
            .add_suffix(owner_key)
            .remove(storage, &delegate)?;
        //ANY_DELEGATES_INVERSE
        //    .add_suffix(delegate.as_slice())
        //    .remove(storage, &owner)?;
    }

    Ok(())
}

pub fn add_token_delegate(
    storage: &mut dyn Storage,
    owner: CanonicalAddr,
    address: CanonicalAddr,
    token_ids: Vec<String>,
) -> StdResult<()> {
    let owner_key = owner.as_slice();

    let token_delegate = TOKEN_DELEGATES
        .add_suffix(owner_key)
        .get(storage, &address);

    if let Some(mut token_set) = token_delegate {
        token_set.extend(token_ids.iter().cloned());
        TOKEN_DELEGATES
            .add_suffix(owner_key)
            .insert(storage, &address, &token_set)?;
    } else {
        let token_set: HashSet<String> = HashSet::from_iter(token_ids.iter().cloned());
        TOKEN_DELEGATES
            .add_suffix(owner_key)
            .insert(storage, &address, &token_set)?;
    }

    for token_id in token_ids {
        let token_delegate_inverse = TOKEN_DELEGATES_INVERSE
            .add_suffix(owner_key)
            .get(storage, &token_id);
        if let Some(mut address_set) = token_delegate_inverse {
            address_set.insert(address.clone());
            TOKEN_DELEGATES_INVERSE
                .add_suffix(owner_key)
                .insert(storage, &token_id, &address_set)?;
        } else {
            let mut address_set: HashSet<CanonicalAddr> = HashSet::new();
            address_set.insert(address.clone());
            TOKEN_DELEGATES_INVERSE
                .add_suffix(owner_key)
                .insert(storage, &token_id, &address_set)?;
        }
    }
    Ok(())
}

pub fn remove_token_delegate(
    storage: &mut dyn Storage,
    owner: CanonicalAddr,
    address: CanonicalAddr,
) -> StdResult<()> {
    let owner_key = owner.as_slice();

    let token_ids = TOKEN_DELEGATES
        .add_suffix(owner_key)
        .get(storage, &address);

    if let Some(token_set) = token_ids {
        for token in token_set {
            let addresses = TOKEN_DELEGATES_INVERSE
                .add_suffix(owner_key)
                .get(storage, &token);
            if let Some(mut addresses) = addresses {
                if addresses.contains(&address) {
                    addresses.remove(&address);
                    if addresses.len() > 0 {
                        TOKEN_DELEGATES_INVERSE
                            .add_suffix(owner_key)
                            .insert(storage, &token, &addresses)?;
                    } else {
                        TOKEN_DELEGATES_INVERSE
                            .add_suffix(owner_key)
                            .remove(storage, &token)?;
                    }
                }
            }
        }
    }

    TOKEN_DELEGATES
        .add_suffix(owner_key)
        .remove(storage, &address)
}

pub fn remove_all_token_delegates(
    storage: &mut dyn Storage,
    owner: CanonicalAddr,
) -> StdResult<()> {
    let owner_key = owner.as_slice();
    let delegates: Vec<CanonicalAddr> = TOKEN_DELEGATES
        .add_suffix(owner_key)
        .iter_keys(storage)?
        .map(|x| x.unwrap())
        .collect();
    for delegate in delegates {
        remove_token_delegate(storage, owner.clone(), delegate)?;
    }
    Ok(())
}
