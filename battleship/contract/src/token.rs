use cosmwasm_std::{CanonicalAddr, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::Permission;
use crate::nfp::{RawData, StoredRawData,};

/// token
#[derive(Serialize, Deserialize)]
pub struct Token {
    /// owner
    pub owner: CanonicalAddr,
    /// permissions granted for this token
    pub permissions: Vec<Permission>,
    /// true if this token has been unwrapped.  If sealed metadata is not enabled, all
    /// tokens are considered unwrapped
    pub unwrapped: bool,
    /// true if this token is transferable
    pub transferable: bool,
}

/// token metadata
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, Debug, Default)]
pub struct Metadata {
    /// optional uri for off-chain metadata.  This should be prefixed with `http://`, `https://`, `ipfs://`, or
    /// `ar://`.  Only use this if you are not using `extension`
    pub token_uri: Option<String>,
    /// optional on-chain metadata.  Only use this if you are not using `token_uri`
    pub extension: Option<Extension>,
}

impl Metadata {
    pub fn into_stored(self) -> StdResult<StoredMetadata> {
        let stored_extension = match self.extension {
            Some(extension) => Some(extension.into_stored()?),
            None => None,
        };
        Ok(StoredMetadata {
            token_uri: self.token_uri,
            extension: stored_extension,
        })
    }
}

/// token metadata
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug, Default)]
pub struct StoredMetadata {
    /// optional uri for off-chain metadata.  This should be prefixed with `http://`, `https://`, `ipfs://`, or
    /// `ar://`.  Only use this if you are not using `extension`
    pub token_uri: Option<String>,
    /// optional on-chain metadata.  Only use this if you are not using `token_uri`
    pub extension: Option<StoredExtension>,
}

impl StoredMetadata {
    pub fn into_humanized(self) -> StdResult<Metadata> {
        let extension = match self.extension {
            Some(extension) => Some(extension.into_humanized()?),
            None => None,
        };
        Ok(Metadata {
            token_uri: self.token_uri,
            extension,
        })
    }
}

/// metadata extension
/// You can add any metadata fields you need here.  These fields are based on
/// https://docs.opensea.io/docs/metadata-standards and are the metadata fields that
/// Stashh uses for robust NFT display.  Urls should be prefixed with `http://`, `https://`, `ipfs://`, or
/// `ar://`
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, Debug, Default)]
pub struct Extension {
    /// url to the image
    pub image: Option<String>,
    /// raw SVG image data (not recommended). Only use this if you're not including the image parameter
    pub image_data: Option<String>,
    /// url to allow users to view the item on your site
    pub external_url: Option<String>,
    /// item description
    pub description: Option<String>,
    /// name of the item
    pub name: Option<String>,
    /// item attributes
    pub attributes: Option<Vec<Trait>>,
    /// background color represented as a six-character hexadecimal without a pre-pended #
    pub background_color: Option<String>,
    /// url to a multimedia attachment
    pub animation_url: Option<String>,
    /// url to a YouTube video
    pub youtube_url: Option<String>,
    /// media files as specified on Stashh that allows for basic authenticatiion and decryption keys.
    /// Most of the above is used for bridging public eth NFT metadata easily, whereas `media` will be used
    /// when minting NFTs on Stashh
    pub media: Option<Vec<MediaFile>>,
    /// a select list of trait_types that are in the private metadata.  This will only ever be used
    /// in public metadata
    pub protected_attributes: Option<Vec<String>>,
    /// token subtypes used by Stashh for display groupings (primarily used for badges, which are specified
    /// by using "badge" as the token_subtype)
    pub token_subtype: Option<String>,
    /// raw data stored on chain
    pub raw_data: Option<Vec<RawData>>,
}

impl Extension {
    pub fn into_stored(self) -> StdResult<StoredExtension> {
        let stored_raw_data = match self.raw_data {
            Some(raw_data) => {
                Some(
                    raw_data
                        .into_iter()
                        .map(|raw_data| raw_data.into_stored().unwrap())
                        .collect()
                )
            },
            None => None,
        };
        Ok(StoredExtension {
            image: self.image,
            image_data: self.image_data,
            external_url: self.external_url,
            description: self.description,
            name: self.name,
            attributes: self.attributes,
            background_color: self.background_color,
            animation_url: self.animation_url,
            youtube_url: self.youtube_url,
            media: self.media,
            protected_attributes: self.protected_attributes,
            token_subtype: self.token_subtype,
            raw_data: stored_raw_data,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug, Default)]
pub struct StoredExtension {
    /// url to the image
    pub image: Option<String>,
    /// raw SVG image data (not recommended). Only use this if you're not including the image parameter
    pub image_data: Option<String>,
    /// url to allow users to view the item on your site
    pub external_url: Option<String>,
    /// item description
    pub description: Option<String>,
    /// name of the item
    pub name: Option<String>,
    /// item attributes
    pub attributes: Option<Vec<Trait>>,
    /// background color represented as a six-character hexadecimal without a pre-pended #
    pub background_color: Option<String>,
    /// url to a multimedia attachment
    pub animation_url: Option<String>,
    /// url to a YouTube video
    pub youtube_url: Option<String>,
    /// media files as specified on Stashh that allows for basic authenticatiion and decryption keys.
    /// Most of the above is used for bridging public eth NFT metadata easily, whereas `media` will be used
    /// when minting NFTs on Stashh
    pub media: Option<Vec<MediaFile>>,
    /// a select list of trait_types that are in the private metadata.  This will only ever be used
    /// in public metadata
    pub protected_attributes: Option<Vec<String>>,
    /// token subtypes used by Stashh for display groupings (primarily used for badges, which are specified
    /// by using "badge" as the token_subtype)
    pub token_subtype: Option<String>,
    /// raw data stored on chain.
    pub raw_data: Option<Vec<StoredRawData>>,
}

impl StoredExtension {
    pub fn into_humanized(self) -> StdResult<Extension> {
        let raw_data = match self.raw_data {
            Some(raw_data) => {
                Some(
                    raw_data
                        .into_iter()
                        .map(|raw_data| raw_data.into_humanized().unwrap())
                        .collect()
                )
            },
            None => None,
        };
        Ok(Extension {
            image: self.image,
            image_data: self.image_data,
            external_url: self.external_url,
            description: self.description,
            name: self.name,
            attributes: self.attributes,
            background_color: self.background_color,
            animation_url: self.animation_url,
            youtube_url: self.youtube_url,
            media: self.media,
            protected_attributes: self.protected_attributes,
            token_subtype: self.token_subtype,
            raw_data,
        })
    }
}

/// attribute trait
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, Debug, Default)]
pub struct Trait {
    /// indicates how a trait should be displayed
    pub display_type: Option<String>,
    /// name of the trait
    pub trait_type: Option<String>,
    /// trait value
    pub value: String,
    /// optional max value for numerical traits
    pub max_value: Option<String>,
}

/// media file
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, Debug, Default)]
pub struct MediaFile {
    /// file type
    /// Stashh currently uses: "image", "video", "audio", "text", "font", "application"
    pub file_type: Option<String>,
    /// file extension
    pub extension: Option<String>,
    /// authentication information
    pub authentication: Option<Authentication>,
    /// url to the file.  Urls should be prefixed with `http://`, `https://`, `ipfs://`, or `ar://`
    pub url: String,
}

/// media file authentication
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, Debug, Default)]
pub struct Authentication {
    /// either a decryption key for encrypted files or a password for basic authentication
    pub key: Option<String>,
    /// username used in basic authentication
    pub user: Option<String>,
}
