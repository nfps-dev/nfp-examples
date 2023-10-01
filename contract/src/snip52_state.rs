use base64::{engine::general_purpose, Engine};
use secret_toolkit_storage::{Keymap, Item};
use cosmwasm_std::{CanonicalAddr, Storage, StdResult, Binary,};
use crate::snip52_crypto::hkdf_sha_256;

pub static INTERNAL_SECRET: Item<Vec<u8>> = Item::new(b"snip52-secret");
pub static COUNTERS: Keymap<CanonicalAddr,u64> = Keymap::new(b"snip52-counters");
pub static SEEDS: Keymap<CanonicalAddr,Vec<u8>> = Keymap::new(b"snip52-seeds");

/// increment counter for a given address
pub fn increment_count(
    storage: &mut dyn Storage,
    channel: &String,
    addr: &CanonicalAddr,
) -> StdResult<u64> {
    let count = COUNTERS.add_suffix(channel.as_bytes()).get(storage, addr).unwrap_or(0_u64);
    let new_count = count.wrapping_add(1_u64);
    COUNTERS.add_suffix(channel.as_bytes()).insert(storage, addr, &new_count)?;
    Ok(new_count)
}

/// get counter for a given address
#[inline]
pub fn get_count(
    storage: &dyn Storage,
    channel: &String,
    addr: &CanonicalAddr,
) -> u64 {
    COUNTERS.add_suffix(channel.as_bytes()).get(storage, addr).unwrap_or(0_u64)
}

/// store the seed for a given address
#[inline]
pub fn store_seed(
    storage: &mut dyn Storage,
    addr: &CanonicalAddr,
    seed: Vec<u8>,
) -> StdResult<()> {
    SEEDS.insert(storage, addr, &seed)
}

/// get the seed for a given address
/// fun getSeedFor(recipientAddr) {
///   // recipient has a shared secret with contract
///   let seed := sharedSecretsTable[recipientAddr]
/// 
///   // no explicit shared secret; derive seed using contract's internal secret
///   if NOT exists(seed):
///     seed := hkdf(ikm=contractInternalSecret, info=canonical(recipientAddr))
///
///   return seed
/// }

pub fn get_seed(
    storage: &dyn Storage,
    addr: &CanonicalAddr,
) -> StdResult<Binary> {
    let may_seed = SEEDS.get(storage, addr);
    if let Some(seed) = may_seed {
        Ok(Binary::from(seed))
    } else {
        let new_seed = hkdf_sha_256(
            &None, 
            INTERNAL_SECRET.load(storage)?.as_slice(), 
            addr.as_slice()
        )?;
        Binary::from_base64(&general_purpose::STANDARD.encode(new_seed))
    }
}
