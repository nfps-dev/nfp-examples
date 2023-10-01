use cosmwasm_std::{Storage, StdResult,};
use secret_toolkit_storage::{Keyset, Keymap};
use serde::{Serialize, Deserialize};

pub static CHANNELS: Keyset<String> = Keyset::new(b"channel-ids");
pub static CHANNEL_SCHEMATA: Keymap<String,String> = Keymap::new(b"channel-schemata");

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Channel {
    pub id: String,
    pub schema: Option<String>,
}

impl Channel {
    pub fn store(self, storage: &mut dyn Storage) -> StdResult<()> {
        CHANNELS.insert(storage, &self.id)?;
        if let Some(schema) = self.schema {
            CHANNEL_SCHEMATA.insert(storage, &self.id, &schema)?;
        } else if CHANNEL_SCHEMATA.get(storage, &self.id).is_some() { 
            // double check it does not already have a schema stored, and if 
            //   it does remove it.
            CHANNEL_SCHEMATA.remove(storage, &self.id)?;
        }
        Ok(())
    }
}

// Battleship channels

/// id for the `game_listed` channel
pub const GAME_LISTED_CHANNEL_ID: &str = "game_listed";
/// CDDL Schema for game listed data
pub const GAME_LISTED_CHANNEL_SCHEMA: &str = "game_listed=[game_id:text,title:text,wager_uscrt:biguint]";

/// id for `game_updated` channel
pub const GAME_UPDATED_CHANNEL_ID: &str = "game_updated";
/// CDDL Schema for the game updated data
pub const GAME_UPDATED_CHANNEL_SCHEMA: &str = "game_updated=[game_id:text,home:[100 uint],turn:uint]";

