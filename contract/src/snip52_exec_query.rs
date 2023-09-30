use bech32::{ToBase32,Variant};
use minicbor_ser as cbor;
use hkdf::hmac::{Mac};
use cosmwasm_std::{DepsMut, Env, Addr, StdResult, Response, StdError, to_binary, Api, Storage, CanonicalAddr, Binary, Deps, Uint64};
use crate::snip52_channel::{CHANNELS, CHANNEL_SCHEMATA};
use crate::snip52_signed_doc::{SignedDocument, pubkey_to_account, Document};
use crate::snip52_state::{get_seed, store_seed, get_count};
use crate::snip52_crypto::{sha_256, HmacSha256, cipher_data};
use crate::msg::{ExecuteAnswer, QueryAnswer, ChannelInfo};

pub const DATA_LEN: usize = 256;

/// 
/// Execute UpdateSeed message
/// 
///   Allows clients to set a new shared secret. In order to guarantee the provided 
///   secret has high entropy, clients must submit a signed document params and signature 
///   to be verified before the new shared secret (i.e., the signature) is accepted.
/// 
///   Updates the sender's seed with the signature of the `signed_doc`. The signed doc
///   is validated to make sure:
///   - the signature is verified, 
///   - that the sender was the signer of the doc, 
///   - the `contract` field matches the address of this contract
///   - the `previous_seed` field matches the previous seed stored in the contract
/// 
pub fn update_seed(
    deps: DepsMut,
    env: Env,
    sender: &Addr,
    signed_doc: SignedDocument,
) -> StdResult<Response> {
    let account = validate_signed_doc(deps.api, &signed_doc, None)?;

    if sender.as_str() != account {
        return Err(StdError::generic_err("Signed doc is not signed by sender"));
    }

    if signed_doc.params.contract != env.contract.address.as_str() {
        return Err(StdError::generic_err(
            "Signed doc is not for this contract",
        ));
    }

    let sender_raw = deps.api.addr_canonicalize(sender.as_str())?;

    let previous_seed = get_seed(deps.storage, &sender_raw)?;
    if previous_seed != signed_doc.params.previous_seed {
        return Err(StdError::generic_err("Previous seed does not match previous seed in signed doc"));
    }

    let new_seed = sha_256(&signed_doc.signature.signature.0).to_vec();

    store_seed(deps.storage, &sender_raw, new_seed)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::UpdateSeed {
        seed: signed_doc.signature.signature,
    })?))
}

///
/// ListChannels query
/// 
///   Public query to list all notification channels.
/// 
pub fn query_list_channels(deps: Deps) -> StdResult<Binary> {
    let channels: Vec<String> = CHANNELS
        .iter(deps.storage)?
        .map(|channel| channel.unwrap())
        .collect();
    to_binary(&QueryAnswer::ListChannels { channels })
}

///
/// ChannelInfo query
/// 
///   Authenticated query allows clients to obtain the seed, counter, 
///   and Notification ID of a future event, for a specific channel.
/// 
pub fn query_channel_info(
    deps: Deps,
    env: &Env,
    channels: Vec<String>,
    sender_raw: CanonicalAddr,
) -> StdResult<Binary> {
    let mut channels_result: Vec<ChannelInfo> = vec![];
    for channel in channels {
        let next_id = notification_id(deps.storage, &sender_raw, &channel)?;
        let counter = Uint64::from(get_count(deps.storage, &channel, &sender_raw));
        let schema = CHANNEL_SCHEMATA.get(deps.storage, &channel);
        channels_result.push(
            ChannelInfo {
                seed: get_seed(deps.storage, &sender_raw)?,
                channel,
                counter, 
                next_id, 
                cddl: schema,
            }
        )
    }

    to_binary(&QueryAnswer::ChannelInfo {
        as_of_block: Uint64::from(env.block.height),
        channels: channels_result,
    })
}

///
/// fn validate_signed_doc
/// 
///   Validates a signed doc to verify the signature is correct. Returns the account
///   derived from the public key.
/// 
fn validate_signed_doc(
    api: &dyn Api,
    signed_doc: &SignedDocument,
    hrp: Option<&str>,
) -> StdResult<String> {
    let account_hrp = hrp.unwrap_or("secret");

    // Derive account from pubkey
    let pubkey = &signed_doc.signature.pub_key.value;

    let base32_addr = pubkey_to_account(pubkey).0.as_slice().to_base32();
    let account: String = bech32::encode(account_hrp, base32_addr, bech32::Variant::Bech32).unwrap();

    let signed_bytes = to_binary(&Document::from_params(&signed_doc.params))?;
    let signed_bytes_hash = sha_256(signed_bytes.as_slice());

    let verified = api
        .secp256k1_verify(
            &signed_bytes_hash, 
            &signed_doc.signature.signature.0, 
            &pubkey.0
        ).map_err(|err| StdError::generic_err(err.to_string()))?;
    
    if !verified {
        return Err(StdError::generic_err(
            "Failed to verify signatures for the given signed doc",
        ));
    }

    Ok(account)
}

/// 
/// fn notification_id
/// 
///   Returns a notification id for the given address and channel id.
/// 
/// pseudocode:
/// 
/// fun notificationIDFor(contractOrRecipientAddr, channelId) {
///   // counter reflects the nth notification for the given contract/recipient in the given channel
///   let counter := getCounterFor(contractOrRecipientAddr, channelId)
///
///   // compute notification ID for this event
///   let seed := getSeedFor(contractOrRecipientAddr)
///   let material := concatStrings(channelId, ":", uintToDecimalString(counter))
///   let notificationID := hmac_sha256(key=seed, message=utf8ToBytes(material))
///
///   return notificationID
/// }
/// 
pub fn notification_id(
    storage: &dyn Storage,
    addr: &CanonicalAddr,
    channel: &String,
) -> StdResult<Binary> {
    let counter = get_count(storage, channel, addr);

    // compute notification ID for this event
    let seed = get_seed(storage, addr)?;
    let material = [
        channel.as_bytes(),
        ":".as_bytes(),
        counter.to_string().as_bytes()
    ].concat();

    let mut mac: HmacSha256 = HmacSha256::new_from_slice(seed.0.as_slice()).unwrap();
    mac.update(material.as_slice());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    Ok(Binary::from(code_bytes.as_slice()))
}

/// 
/// fn encrypt_notification_data
/// 
///   Returns encrypted bytes given plaintext bytes, address, and channel id.
/// 
/// pseudocode:
/// 
/// fun encryptNotificationData(recipientAddr, channelId, plaintext, env) {
///   // counter reflects the nth notification for the given recipient in the given channel
///   let counter := getCounterFor(recipientAddr, channelId)
///
///   let seed := getSeedFor(recipientAddr)
///
///   // ChaCha20 expects a 96-bit (12 bytes) nonce
///   // take the first 12 bytes of the channel id's sha256 hash
///   let channelIdBytes := slice(sha256(utf8ToBytes(channelId)), 0, 12)
///
///   // encode uint64 counter in BE and left-pad with 4 bytes of 0x00
///   let counterBytes := concat(zeros(4), uint64BigEndian(counter))
///
///   // produce the nonce by XOR'ing the two previous 12-byte results
///   let nonce := xorBytes(channelIdBytes, counterBytes)
///
///   // right-pad the plaintext with 0x00 bytes until it is of the desired length (keep in mind, payload adds 16 bytes for tag)
///   let message := concat(plaintext, zeros(DATA_LEN - len(plaintext)))
///
///   // construct the additional authenticated data
///   let aad := concatStrings(env.blockHeight, ":", env.senderAddress)
///
///   // encrypt notification data for this event
///   let [ciphertext, tag] := chacha20poly1305_encrypt(key=seed, nonce=nonce, message=message, aad=aad)
///
///   // concatenate ciphertext and 16 bytes of tag (note: crypto libs typically default to doing it this way in `seal`)
///   let payload := concat(ciphertext, tag)
///
///   return payload
/// }
/// 

pub fn encrypt_notification_data(
    storage: &dyn Storage,
    env: &Env,
    sender: &Addr,
    recipient: &CanonicalAddr,
    channel: &String,
    plaintext: Vec<u8>,
) -> StdResult<Binary> {
    let counter = get_count(storage, channel, recipient);
    let mut padded_plaintext = plaintext.clone();
    zero_pad(&mut padded_plaintext, DATA_LEN);

    let seed = get_seed(storage, recipient)?;
    let channel_id_bytes = sha_256(channel.as_bytes())[..12].to_vec();
    let counter_bytes = [&[0_u8, 0_u8, 0_u8, 0_u8], counter.to_be_bytes().as_slice()].concat();
    let nonce: Vec<u8> = channel_id_bytes.iter().zip(counter_bytes.iter()).map(|(&b1, &b2)| b1 ^ b2 ).collect();
    // TODO: add option to use tx hash instead of sender in aad
    //       requires tx hash to be added to `env`
    let aad = format!("{}:{}", env.block.height, sender.to_string());

    // encrypt notification data for this event
    let tag_ciphertext = cipher_data(
        seed.0.as_slice(),
        nonce.as_slice(),
        padded_plaintext.as_slice(),
        aad.as_bytes()
    )?;

    Ok(Binary::from(tag_ciphertext.clone()))
}


/// Take a Vec<u8> and pad it up to a multiple of `block_size`, using 0x00 at the end.
fn zero_pad(message: &mut Vec<u8>, block_size: usize) -> &mut Vec<u8> {
    let len = message.len();
    let surplus = len % block_size;
    if surplus == 0 {
        return message;
    }

    let missing = block_size - surplus;
    message.reserve(missing);
    message.extend(std::iter::repeat(0x00).take(missing));
    message
}