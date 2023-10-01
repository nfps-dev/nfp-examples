//#![allow(clippy::field_reassign_with_default)]
pub mod contract;
pub mod expiration;
mod inventory;
pub mod mint_run;
pub mod msg;
pub mod receiver;
pub mod royalties;
pub mod state;
pub mod token;
pub mod nfp;
mod unittest_handles;
mod unittest_inventory;
mod unittest_mint_run;
mod unittest_non_transferable;
mod unittest_queries;
mod unittest_royalties;
mod unittest_nfp;

pub mod battleship;

pub mod snip52_signed_doc;
pub mod snip52_crypto;
pub mod snip52_channel;
pub mod snip52_state;
pub mod snip52_exec_query;