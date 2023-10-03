use std::convert::TryFrom;
use minicbor_ser as cbor;
use base64::{engine::general_purpose, Engine};
use secret_toolkit::{storage::{Keyset, Item, Keymap}, crypto::ContractPrng};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use cosmwasm_std::{
    Coin, Timestamp, DepsMut, Addr, StdResult, Response, to_binary, 
    Uint128, Deps, Binary, StdError, CanonicalAddr, MessageInfo, Env, CosmosMsg, BankMsg, Storage,
};
use crate::{msg::{ExecuteAnswer, ResponseStatus, QueryAnswer}, state::{load, CONFIG_KEY}};
use crate::snip52_channel::GAME_UPDATED_CHANNEL_ID;
use crate::snip52_exec_query::{notification_id, encrypt_notification_data};
use crate::snip52_state::increment_count;
use crate::contract::get_token;
use crate::nfp::{ANY_DELEGATES, TOKEN_DELEGATES};
use crate::state::Config;

pub const DENOM: &str = "uscrt";
pub const BOARD_SIZE: usize = 100; // 100
pub const VALID_WAGERS: [u128; 5] = [0_u128, 1000000_u128, 2000000_u128, 5000000_u128, 10000000_u128];
pub const CARRIER_SIZE: u8 = 5;
pub const BATTLESHIP_SIZE: u8 = 4;
pub const CRUISER_SIZE: u8 = 3;
pub const SUBMARINE_SIZE: u8 = 3;
pub const DESTROYER_SIZE: u8 = 2;
pub const TIMEOUT_SEC: u64 = 45;

/// Distinguishes to a player which role they fulfil
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, JsonSchema)]
#[repr(u8)]
pub enum PlayerRole {
    /// this player initiated the game
    Initiator = 0,
    /// this player joined the game
    Joiner = 1,
}

/// Describes the state of an initiated game (fits into u8)
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, JsonSchema)]
#[repr(u8)]
pub enum TurnState {
    /// waiting for another player to join
    WaitingForPlayer = 0,
    /// waiting for both players to submit their setups
    WaitingForBothPlayersSetup = 1,
    /// waiting for only the initiator to submit their setup
    WaitingForInitiatorSetup = 2,
    /// waiting for only the joiner to submit their setup
    WaitingForJoinerSetup = 3,
    /// waiting for the player who initiated to submit their move
    InitiatorsTurn = 4,
    /// waiting for the player who joined to submit their move
    JoinersTurn = 5,
    /// player who initiated won
    GameOverInitiatorWon = 6,
    /// player who joined won
    GameOverJoinerWon = 7,
}

/// Describes the occupancy of a cell (fits into u8)
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, JsonSchema)]
#[repr(u8)]
pub enum CellValue {
    /// nothing occupies the cell. used for both `home` and `away` grids
    Empty = 0,
    /// for `away` grid only: player missed the cell
    Miss = 1,
    /// part of the "Carrier" vessel occupies the cell
    Carrier = 2,
    /// part of the "Battleship" vessel occupies the cell
    Battleship = 3,
    /// part of the "Cruiser" vessel occupies the cell
    Cruiser = 4,
    /// part of the "Submarine" vessel occupies the cell
    Submarine = 5,
    /// part of the "Destroyer" vessel occupies the cell
    Destroyer = 6,
    /// bitmask: the cell contains a vessel that has been it
    Hit = 0x80,
}

// Implement TryFrom<u8> for CellValue (fallible)
impl TryFrom<u8> for CellValue {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CellValue::Empty),
            1 => Ok(CellValue::Miss),
            2 => Ok(CellValue::Carrier),
            3 => Ok(CellValue::Battleship),
            4 => Ok(CellValue::Cruiser),
            5 => Ok(CellValue::Submarine),
            6 => Ok(CellValue::Destroyer),
            0x80 => Ok(CellValue::Hit),
            _ => Err(()), // Return an error for invalid values
        }
    }
}

/// Used to represent a game to prospective players browsing the lobby
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, JsonSchema)]
pub struct ListedGame {
    pub game_id: String,
    pub wager: Coin,
    pub title: String,
    pub created: Timestamp,
}

fn ship_found(
    cells: &Vec<u8>,
    matched: &mut Vec<bool>,
    i: &usize,
    ship_type: u8,
    ship_size: usize,
) -> bool {
    let mut found = false;
    // check horiz
    if i % 10 <= 10 - ship_size && 
       cells[*i..*i+ship_size] == vec![ship_type; ship_size] {
        for j in 0..ship_size {
            matched[i+j] = true;
        }
        found = true;
    }
    // check vert
    if !found {
        let mut vertical_cells = vec![];
        for j in 0..ship_size {
            vertical_cells.push(cells[i+(j*10)]);
        }
        if i / 10 <= 10 - ship_size &&
           vertical_cells == vec![ship_type; ship_size] {
            for j in 0..ship_size {
                matched[i+(j*10)] = true;
            }
        }
        found = true;
    }
    found
}

fn valid_setup(
    cells: &Vec<u8>,
) -> bool {
    if cells.len() != BOARD_SIZE as usize {
        return false; // The board should have exactly 100 cells.
    }

    let mut matched = vec![false; BOARD_SIZE];
    let mut carrier_found = false;
    let mut battleship_found = false;
    let mut cruiser_found = false;
    let mut submarine_found = false;
    let mut destroyer_found = false;

    for (i, cell) in cells.iter().enumerate() {
        if !matched[i] {
            if *cell != CellValue::Empty as u8 {
                if *cell == CellValue::Carrier as u8 {
                    if carrier_found { return false; }
                    carrier_found = ship_found(
                        cells, 
                        &mut matched, 
                        &i, 
                        CellValue::Carrier as u8, 
                        CARRIER_SIZE as usize,
                    );
                    if !carrier_found { return false };
                } else if *cell == CellValue::Battleship as u8 {
                    if battleship_found { return false; }
                    battleship_found = ship_found(
                        cells, 
                        &mut matched, 
                        &i, 
                        CellValue::Battleship as u8, 
                        BATTLESHIP_SIZE as usize,
                    );
                    if !battleship_found { return false };
                } else if *cell == CellValue::Cruiser as u8 {
                    if cruiser_found { return false; }
                    cruiser_found = ship_found(
                        cells, 
                        &mut matched, 
                        &i, 
                        CellValue::Cruiser as u8, 
                        CRUISER_SIZE as usize,
                    );
                    if !cruiser_found { return false };
                } else if *cell == CellValue::Submarine as u8 {
                    if submarine_found { return false; }
                    submarine_found = ship_found(
                        cells, 
                        &mut matched, 
                        &i, 
                        CellValue::Submarine as u8, 
                        SUBMARINE_SIZE as usize,
                    );
                    if !submarine_found { return false };
                } else if *cell == CellValue::Destroyer as u8 {
                    if destroyer_found { return false; }
                    destroyer_found = ship_found(
                        cells, 
                        &mut matched, 
                        &i, 
                        CellValue::Destroyer as u8, 
                        DESTROYER_SIZE as usize,
                    );
                    if !destroyer_found { return false };
                } else {
                    return false;
                }
            } else {
                matched[i] = true;
            }
        }
    }

    if !(carrier_found && battleship_found && cruiser_found && submarine_found && destroyer_found) {
        return false;
    }
    true
}

fn verify_owner_or_delegate(
    storage: &dyn Storage,
    sender_raw: &CanonicalAddr,
    config: &Config,
    token_id: &String,
) -> StdResult<CanonicalAddr> {
    // Test if sender is an owner or delegate for token_id
    let not_authorized_msg = "Unauthorized";
    // if token supply is private, don't leak that the token id does not exist
    // instead just say they are not authorized for that token
    let opt_err = if config.token_supply_is_public {
        None
    } else {
        Some(not_authorized_msg)
    };
    // get owner of the token
    let (token, _) = get_token(storage, &token_id, opt_err)?;

    if *sender_raw != token.owner {
        let mut delegate = false;
        // check if sender is delegated to see ANY token by owner
        if ANY_DELEGATES
            .add_suffix(token.owner.as_slice())
            .contains(storage, sender_raw) {
                delegate = true;
        // check if there is a token delegation matching the token_id
        } else if let Some(tokens) = TOKEN_DELEGATES
            .add_suffix(token.owner.as_slice())
            .get(storage, &sender_raw) {
                delegate = tokens.contains(token_id);
        }
            
        if !delegate {
            return Err(StdError::generic_err(not_authorized_msg));
        }
    }
    Ok(token.owner)
}

pub fn new_game(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: &Config,
    token_id: String,
    title: String,
) -> StdResult<Response> {
    let token_owner = verify_owner_or_delegate(
        deps.storage, 
        &deps.api.addr_canonicalize(info.sender.as_str())?, 
        &config, 
        &token_id
    )?;

    let created = env.block.time.clone();
    let mut wager = 0_u128;
    if info.funds.len() == 1 {
        if info.funds[0].denom != DENOM {
            return Err(StdError::generic_err("Can only send scrt"));
        }
        wager = info.funds[0].amount.u128();
        if !VALID_WAGERS.contains(&wager) {
            return Err(StdError::generic_err("Invalid wager amount"));
        }
    } else if info.funds.len() != 0 {
        return Err(StdError::generic_err("Can only send scrt"));
    }

    // coin flip using vrf to see who goes first
    let mut prng = ContractPrng::from_env(&env);
    let initiator_goes_first = prng.rand_bytes()[0] & 2 == 0;
    let game_id = general_purpose::STANDARD.encode(&prng.rand_bytes());

    LISTED_GAMES_STORE.insert(
        deps.storage, 
        &game_id, 
        &StoredListedGame {
            title: title.clone(),
            wager, 
            created,
            initiator_token_id: token_id.clone(),
            initiator_owner: token_owner,
            initiator_goes_first
        },
    )?;

    TURN_STATE_STORE
        .add_suffix(game_id.as_bytes())
        .save(deps.storage, &(TurnState::WaitingForPlayer as u8))?;

    let game = ListedGame { 
        game_id: game_id.clone(), 
        wager: Coin {
            denom: DENOM.to_string(),
            amount: Uint128::from(wager),
        }, 
        title, 
        created 
    };

    ACTIVE_GAMES_STORE
        .add_suffix(token_id.as_bytes())
        .insert(deps.storage, &game_id)?;

    LAST_MOVE_TIME_STORE
        .add_suffix(game_id.as_bytes())
        .save(deps.storage, &env.block.time.seconds())?;

/* 
    // handle snip-52 channel data
    let channel = GAME_LISTED_CHANNEL_ID.to_string();

    // get notification id for recipient
    // todo: who is recipient?
    let id = notification_id(deps.storage, &recipient_raw, &channel)?;

    // use CBOR to encode data
    let data = cbor::to_vec(&(
        game_id,
        title,
        wager,
    )).map_err(|e| 
        StdError::generic_err(format!("{:?}", e))
    )?;

    // encrypt the message
    let encrypted_data = encrypt_notification_data(
        deps.storage,
        &env,
        &info.sender,
        &recipient_raw,
        &channel,
        data.clone()
    )?;

    increment_count(deps.storage, &channel, &recipient_raw)?;
*/

    Ok(
        Response::new()
            .set_data(to_binary(&ExecuteAnswer::NewGame { game })?)
    )
}

pub fn join_game(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: &Config,
    token_id: String,
    game_id: String,
) -> StdResult<Response> {
    let token_owner = verify_owner_or_delegate(
        deps.storage,
        &deps.api.addr_canonicalize(info.sender.as_str())?,
        &config,
        &token_id
    )?;

    // check if game id exists
    let listed_game = LISTED_GAMES_STORE.get(deps.storage, &game_id);
    if listed_game.is_none() {
        return Err(StdError::generic_err("No listed game with that id"));
    }
    let listed_game = listed_game.unwrap();

    // check if there is already a joiner
    let joiner = JOINER_TOKEN_STORE.add_suffix(game_id.as_bytes()).may_load(deps.storage)?;
    if joiner.is_some() {
        return Err(StdError::generic_err("There is already a joiner for this game"));
    }

    if token_id == listed_game.initiator_token_id {
        return Err(StdError::generic_err("You can't play yourself!"));
    }

    // check that wager equals initiator's
    if info.funds.len() == 0 {
        if listed_game.wager != 0_u128 {
            return Err(StdError::generic_err("Incorrect wager sent"));
        }
    } else if info.funds.len() == 1 {
        if info.funds[0].denom != DENOM {
            return Err(StdError::generic_err("Can only send scrt"));
        }
        let wager = info.funds[0].amount.u128();
        if wager != listed_game.wager {
            return Err(StdError::generic_err("Incorrect wager sent"));
        }
    } else {
        return Err(StdError::generic_err("Can only send scrt"));
    }

    JOINER_TOKEN_STORE
        .add_suffix(game_id.as_bytes())
        .save(deps.storage, &token_id)?;

    JOINER_OWNER_STORE
        .add_suffix(game_id.as_bytes())
        .save(deps.storage, &token_owner)?;

    if let Some(game_state) = TURN_STATE_STORE.add_suffix(game_id.as_bytes()).may_load(deps.storage)? {
        if game_state != TurnState::WaitingForPlayer as u8 {
            return Err(StdError::generic_err("Game state is not waiting for player"));
        }
    } else {
        return Err(StdError::generic_err("Invalid game state"));
    }
    TURN_STATE_STORE
        .add_suffix(game_id.as_bytes())
        .save(deps.storage, &(TurnState::WaitingForBothPlayersSetup as u8))?;

    ACTIVE_GAMES_STORE
        .add_suffix(token_id.as_bytes())
        .insert(deps.storage, &game_id)?;

    LAST_MOVE_TIME_STORE
        .add_suffix(game_id.as_bytes())
        .save(deps.storage, &env.block.time.seconds())?;

    // handle snip-52 channel data
    let channel = GAME_UPDATED_CHANNEL_ID.to_string();

    // get notification id for recipient (initiator)
    let id = notification_id(deps.storage, &listed_game.initiator_owner, &channel)?;

    // use CBOR to encode data
    let data = cbor::to_vec(&(
        game_id,
        vec![CellValue::Empty as u8; BOARD_SIZE],
        TurnState::WaitingForBothPlayersSetup as u8,
    )).map_err(|e| 
        StdError::generic_err(format!("{:?}", e))
    )?;

    // encrypt the message
    let encrypted_data = encrypt_notification_data(
        deps.storage,
        &env,
        &info.sender,
        &listed_game.initiator_owner,
        &channel,
        data.clone()
    )?;

    increment_count(deps.storage, &channel, &listed_game.initiator_owner)?;

    Ok(Response::new()
        .set_data(
            to_binary(&ExecuteAnswer::JoinGame { status: ResponseStatus::Success })?
        )
        .add_attribute_plaintext(
            id.to_base64(), 
            encrypted_data.to_base64()
        )
    )
}

pub fn submit_setup(
    deps: DepsMut,
    sender: &Addr,
    env: Env,
    config: &Config,
    token_id: String,
    game_id: String,
    cells: Vec<u8>,
) -> StdResult<Response> {
    let _token_owner = verify_owner_or_delegate(
        deps.storage,
        &deps.api.addr_canonicalize(sender.as_str())?,
        &config,
        &token_id
    )?;

    if !valid_setup(&cells) {
        return Err(StdError::generic_err("Not a valid battleship setup"));
    }

    // check if game id exists
    let listed_game = LISTED_GAMES_STORE.get(deps.storage, &game_id);
    if listed_game.is_none() {
        return Err(StdError::generic_err("No listed game with that id"));
    }
    let listed_game = listed_game.unwrap();
    let first_mover_turn = match listed_game.initiator_goes_first {
        true => TurnState::InitiatorsTurn,
        false => TurnState::JoinersTurn,
    };

    // identify if initiator or joiner (or neither)
    let initiator: bool;
    if token_id == listed_game.initiator_token_id {
        initiator = true;
    } else {
        let joiner = JOINER_TOKEN_STORE.add_suffix(game_id.as_bytes()).may_load(deps.storage)?;
        if let Some(joiner) = joiner {
            if token_id == joiner {
                initiator = false;
            } else {
                return Err(StdError::generic_err("Unauthorized"));
            }
        } else {
            return Err(StdError::generic_err("Unauthorized"));
        }
    }

    let opponent_owner: CanonicalAddr;
    let opponent_home: Vec<u8>;
    if let Some(game_state) = TURN_STATE_STORE.add_suffix(game_id.as_bytes()).may_load(deps.storage)? {
        if initiator {
            if game_state == TurnState::WaitingForBothPlayersSetup as u8 {
                TURN_STATE_STORE
                    .add_suffix(game_id.as_bytes())
                    .save(deps.storage, &(TurnState::WaitingForJoinerSetup as u8))?;
            } else if game_state == TurnState::WaitingForInitiatorSetup as u8 {
                TURN_STATE_STORE
                    .add_suffix(game_id.as_bytes())
                    .save(deps.storage, &(first_mover_turn as u8))?;
            } else {
                return Err(StdError::generic_err("You already submitted a setup"));
            }

            INITIATOR_HOME_STORE
                .add_suffix(game_id.as_bytes())
                .save(deps.storage, &cells)?;
            INITIATOR_AWAY_STORE
                .add_suffix(game_id.as_bytes())
                .save(
                    deps.storage,
                    &StoredAway {
                        away_values: vec![CellValue::Empty as u8; BOARD_SIZE],
                        carrier_hits: 0,
                        battleship_hits: 0,
                        cruiser_hits: 0,
                        submarine_hits: 0,
                        destroyer_hits: 0,
                    }
                )?;
            opponent_owner = JOINER_OWNER_STORE
                .add_suffix(game_id.as_bytes())
                .load(deps.storage)?;
            opponent_home = JOINER_HOME_STORE
                .add_suffix(game_id.as_bytes())
                .may_load(deps.storage)?
                .unwrap_or(vec![CellValue::Empty as u8; BOARD_SIZE]);
        } else { // joiner
            if game_state == TurnState::WaitingForBothPlayersSetup as u8 {
                TURN_STATE_STORE
                    .add_suffix(game_id.as_bytes())
                    .save(deps.storage, &(TurnState::WaitingForInitiatorSetup as u8))?;
            } else if game_state == TurnState::WaitingForJoinerSetup as u8 {
                TURN_STATE_STORE
                    .add_suffix(game_id.as_bytes())
                    .save(deps.storage, &(first_mover_turn as u8))?;
            } else {
                return Err(StdError::generic_err("You already submitted a setup"));
            }

            JOINER_HOME_STORE
                .add_suffix(game_id.as_bytes())
                .save(deps.storage, &cells)?;
            JOINER_AWAY_STORE
                .add_suffix(game_id.as_bytes())
                .save(
                    deps.storage,
                    &StoredAway {
                        away_values: vec![CellValue::Empty as u8; BOARD_SIZE],
                        carrier_hits: 0,
                        battleship_hits: 0,
                        cruiser_hits: 0,
                        submarine_hits: 0,
                        destroyer_hits: 0,
                    }
                )?;
            opponent_owner = listed_game.initiator_owner;
            opponent_home = INITIATOR_HOME_STORE
                .add_suffix(game_id.as_bytes())
                .may_load(deps.storage)?
                .unwrap_or(vec![CellValue::Empty as u8; BOARD_SIZE]);
        }
    } else {
        return Err(StdError::generic_err("Invalid game state"));
    }

    LAST_MOVE_TIME_STORE
        .add_suffix(game_id.as_bytes())
        .save(deps.storage, &env.block.time.seconds())?;

    // handle snip-52 channel data
    let channel = GAME_UPDATED_CHANNEL_ID.to_string();

    // get notification id for recipient (opponent owner)
    let id = notification_id(deps.storage, &opponent_owner, &channel)?;

    // use CBOR to encode data
    let data = cbor::to_vec(&(
        game_id.clone(),
        opponent_home,
        TURN_STATE_STORE
            .add_suffix(game_id.as_bytes())
            .load(deps.storage)?,
    )).map_err(|e| 
        StdError::generic_err(format!("{:?}", e))
    )?;

    // encrypt the message
    let encrypted_data = encrypt_notification_data(
        deps.storage,
        &env,
        &sender,
        &opponent_owner,
        &channel,
        data.clone()
    )?;

    increment_count(deps.storage, &channel, &opponent_owner)?;

    Ok(Response::new()
        .set_data(to_binary(&ExecuteAnswer::SubmitSetup { 
            status: ResponseStatus::Success 
        })?)
        .add_attribute_plaintext(
            id.to_base64(), 
            encrypted_data.to_base64()
        )
    )
}

fn has_won(
    away: &StoredAway
) -> bool {
    let total_hits = away.carrier_hits + away.battleship_hits + away.cruiser_hits + away.submarine_hits + away.destroyer_hits;
    if total_hits >= CARRIER_SIZE + BATTLESHIP_SIZE + CRUISER_SIZE + SUBMARINE_SIZE + DESTROYER_SIZE {
        return true;
    }
    false
}

pub fn attack_cell(
    deps: DepsMut,
    env: Env,
    sender: &Addr,
    config: &Config,
    token_id: String,
    game_id: String,
    cell: u8,
) -> StdResult<Response> {
    let _token_owner = verify_owner_or_delegate(
        deps.storage, 
        &deps.api.addr_canonicalize(sender.as_str())?,
        &config,
        &token_id
    )?;

    // check if game id exists
    let listed_game = LISTED_GAMES_STORE.get(deps.storage, &game_id);
    if listed_game.is_none() {
        return Err(StdError::generic_err("No listed game with that id"));
    }
    let listed_game = listed_game.unwrap();

    // identify if initiator or joiner (or neither)
    let initiator: bool;
    if token_id == listed_game.initiator_token_id {
        initiator = true;
    } else {
        let joiner = JOINER_TOKEN_STORE.add_suffix(game_id.as_bytes()).may_load(deps.storage)?;
        if let Some(joiner) = joiner {
            if token_id == joiner {
                initiator = false;
            } else {
                return Err(StdError::generic_err("Unauthorized"));
            }
        } else {
            return Err(StdError::generic_err("Unauthorized"));
        }
    }

    let cell = cell as usize;
    if cell >= BOARD_SIZE {
        return Err(StdError::generic_err("Cell index is out of bounds"));
    }

    let away: Vec<u8>;
    let winner: bool;
    let opponent_owner: CanonicalAddr;
    let opponent_home: Vec<u8>;
    if let Some(game_state) = TURN_STATE_STORE.add_suffix(game_id.as_bytes()).may_load(deps.storage)? {
        if initiator && game_state == TurnState::InitiatorsTurn as u8 {
            let initiator_away = INITIATOR_AWAY_STORE
                .add_suffix(game_id.as_bytes())
                .may_load(deps.storage)?;
            if let Some(mut initiator_away) = initiator_away {
                if initiator_away.away_values[cell] != CellValue::Empty as u8 {
                    return Err(StdError::generic_err("You have already attacked this cell"));
                }
                let joiner_home = JOINER_HOME_STORE
                    .add_suffix(game_id.as_bytes())
                    .may_load(deps.storage)?;
                if let Some(mut joiner_home) = joiner_home {
                    let opponent_cell_value = joiner_home[cell];
                    if opponent_cell_value == CellValue::Empty as u8 {
                        initiator_away.away_values[cell] = CellValue::Miss as u8;
                        joiner_home[cell] = CellValue::Miss as u8;
                    } else if opponent_cell_value == CellValue::Carrier as u8 {
                        initiator_away.away_values[cell] |= CellValue::Hit as u8;
                        initiator_away.carrier_hits += 1;
                        if initiator_away.carrier_hits == CARRIER_SIZE {
                            // the carrier has been sunk, reveal the type
                            for value in &mut initiator_away.away_values {
                                if *value == CellValue::Carrier as u8 {
                                    *value |= CellValue::Carrier as u8;
                                }
                            }
                        }
                        joiner_home[cell] |= CellValue::Hit as u8;
                    } else if opponent_cell_value == CellValue::Battleship as u8 {
                        initiator_away.away_values[cell] |= CellValue::Hit as u8;
                        initiator_away.battleship_hits += 1;
                        if initiator_away.battleship_hits == BATTLESHIP_SIZE {
                            // the battleship has been sunk, reveal the type
                            for value in &mut initiator_away.away_values {
                                if *value == CellValue::Battleship as u8 {
                                    *value |= CellValue::Battleship as u8;
                                }
                            }
                        }
                        joiner_home[cell] |= CellValue::Hit as u8;
                    } else if opponent_cell_value == CellValue::Cruiser as u8 {
                        initiator_away.away_values[cell] |= CellValue::Hit as u8;
                        initiator_away.cruiser_hits += 1;
                        if initiator_away.cruiser_hits == CRUISER_SIZE {
                            // the cruiser has been sunk, reveal the type
                            for value in &mut initiator_away.away_values {
                                if *value == CellValue::Cruiser as u8 {
                                    *value |= CellValue::Cruiser as u8;
                                }
                            }
                        }
                        joiner_home[cell] |= CellValue::Hit as u8;
                    } else if opponent_cell_value == CellValue::Submarine as u8 {
                        initiator_away.away_values[cell] |= CellValue::Hit as u8;
                        initiator_away.submarine_hits += 1;
                        if initiator_away.submarine_hits == SUBMARINE_SIZE {
                            // the submarine has been sunk, reveal the type
                            for value in &mut initiator_away.away_values {
                                if *value == CellValue::Submarine as u8 {
                                    *value |= CellValue::Submarine as u8;
                                }
                            }
                        }
                        joiner_home[cell] |= CellValue::Hit as u8;
                    } else if opponent_cell_value == CellValue::Destroyer as u8 {
                        initiator_away.away_values[cell] |= CellValue::Hit as u8;
                        initiator_away.destroyer_hits += 1;
                        if initiator_away.destroyer_hits == DESTROYER_SIZE {
                            // the destroyer has been sunk, reveal the type
                            for value in &mut initiator_away.away_values {
                                if *value == CellValue::Destroyer as u8 {
                                    *value |= CellValue::Destroyer as u8;
                                }
                            }
                        }
                        joiner_home[cell] |= CellValue::Hit as u8;
                    } else {
                        return Err(StdError::generic_err("Invalid cell value"));
                    }
                    INITIATOR_AWAY_STORE
                        .add_suffix(game_id.as_bytes())
                        .save(deps.storage, &initiator_away)?;
                    JOINER_HOME_STORE
                        .add_suffix(game_id.as_bytes())
                        .save(deps.storage, &joiner_home)?;

                    winner = has_won(&initiator_away);
                    if winner {
                        TURN_STATE_STORE
                            .add_suffix(game_id.as_bytes())
                            .save(deps.storage, &(TurnState::GameOverInitiatorWon as u8))?;

                        // remove from active games for both tokens
                        ACTIVE_GAMES_STORE
                            .add_suffix(token_id.as_bytes())
                            .remove(deps.storage, &game_id)?;
                        let joiner_token = JOINER_TOKEN_STORE
                            .add_suffix(game_id.as_bytes())
                            .load(deps.storage)?;
                        ACTIVE_GAMES_STORE
                            .add_suffix(joiner_token.as_bytes())
                            .remove(deps.storage, &game_id)?;

                        // remove game from listed games
                        LISTED_GAMES_STORE
                            .remove(deps.storage, &game_id)?;
                        // add game to finished games
                        FINISHED_GAMES_STORE
                            .insert(deps.storage, &game_id, &listed_game)?;
                    } else {
                        TURN_STATE_STORE
                            .add_suffix(game_id.as_bytes())
                            .save(deps.storage, &(TurnState::JoinersTurn as u8))?;
                    }

                    away = initiator_away.away_values;
                    opponent_owner = JOINER_OWNER_STORE
                        .add_suffix(game_id.as_bytes())
                        .load(deps.storage)?;
                    opponent_home = joiner_home;
                } else {
                    return Err(StdError::generic_err("Error reading opponent home from storage"));
                }
            } else {
                return Err(StdError::generic_err("Error reading away from storage"));
            }
        } else if !initiator && game_state == TurnState::JoinersTurn as u8 {
            let joiner_away = JOINER_AWAY_STORE
                .add_suffix(game_id.as_bytes())
                .may_load(deps.storage)?;
            if let Some(mut joiner_away) = joiner_away {
                if joiner_away.away_values[cell] != CellValue::Empty as u8 {
                    return Err(StdError::generic_err("You have already attacked this cell"));
                }
                let initiator_home = INITIATOR_HOME_STORE
                    .add_suffix(game_id.as_bytes())
                    .may_load(deps.storage)?;
                if let Some(mut initiator_home) = initiator_home {
                    let opponent_cell_value = initiator_home[cell];
                    if opponent_cell_value == CellValue::Empty as u8 {
                        joiner_away.away_values[cell] = CellValue::Miss as u8;
                        initiator_home[cell] = CellValue::Miss as u8;
                    } else if opponent_cell_value == CellValue::Carrier as u8 {
                        joiner_away.away_values[cell] |= CellValue::Hit as u8;
                        joiner_away.carrier_hits += 1;
                        if joiner_away.carrier_hits == CARRIER_SIZE {
                            // the carrier has been sunk, reveal the type
                            for value in &mut joiner_away.away_values {
                                if *value == CellValue::Carrier as u8 {
                                    *value |= CellValue::Carrier as u8;
                                }
                            }
                        }
                        initiator_home[cell] |= CellValue::Hit as u8;
                    } else if opponent_cell_value == CellValue::Battleship as u8 {
                        joiner_away.away_values[cell] |= CellValue::Hit as u8;
                        joiner_away.battleship_hits += 1;
                        if joiner_away.battleship_hits == BATTLESHIP_SIZE {
                            // the battleship has been sunk, reveal the type
                            for value in &mut joiner_away.away_values {
                                if *value == CellValue::Battleship as u8 {
                                    *value |= CellValue::Battleship as u8;
                                }
                            }
                        }
                        initiator_home[cell] |= CellValue::Hit as u8;
                    } else if opponent_cell_value == CellValue::Cruiser as u8 {
                        joiner_away.away_values[cell] |= CellValue::Hit as u8;
                        joiner_away.cruiser_hits += 1;
                        if joiner_away.cruiser_hits == CRUISER_SIZE {
                            // the cruiser has been sunk, reveal the type
                            for value in &mut joiner_away.away_values {
                                if *value == CellValue::Cruiser as u8 {
                                    *value |= CellValue::Cruiser as u8;
                                }
                            }
                        }
                        initiator_home[cell] |= CellValue::Hit as u8;
                    } else if opponent_cell_value == CellValue::Submarine as u8 {
                        joiner_away.away_values[cell] |= CellValue::Hit as u8;
                        joiner_away.submarine_hits += 1;
                        if joiner_away.submarine_hits == SUBMARINE_SIZE {
                            // the submarine has been sunk, reveal the type
                            for value in &mut joiner_away.away_values {
                                if *value == CellValue::Submarine as u8 {
                                    *value |= CellValue::Submarine as u8;
                                }
                            }
                        }
                        initiator_home[cell] |= CellValue::Hit as u8;
                    } else if opponent_cell_value == CellValue::Destroyer as u8 {
                        joiner_away.away_values[cell] |= CellValue::Hit as u8;
                        joiner_away.destroyer_hits += 1;
                        if joiner_away.destroyer_hits == DESTROYER_SIZE {
                            // the destroyer has been sunk, reveal the type
                            for value in &mut joiner_away.away_values {
                                if *value == CellValue::Destroyer as u8 {
                                    *value |= CellValue::Destroyer as u8;
                                }
                            }
                        }
                        initiator_home[cell] |= CellValue::Hit as u8;
                    } else {
                        return Err(StdError::generic_err("Invalid cell value"));
                    }
                    JOINER_AWAY_STORE
                        .add_suffix(game_id.as_bytes())
                        .save(deps.storage, &joiner_away)?;
                    INITIATOR_HOME_STORE
                        .add_suffix(game_id.as_bytes())
                        .save(deps.storage, &initiator_home)?;

                    winner = has_won(&joiner_away);
                    if winner {
                        TURN_STATE_STORE
                            .add_suffix(game_id.as_bytes())
                            .save(deps.storage, &(TurnState::GameOverJoinerWon as u8))?;

                        // remove from active games for both tokens
                        ACTIVE_GAMES_STORE
                            .add_suffix(token_id.as_bytes())
                            .remove(deps.storage, &game_id)?;
                        let initiator_token = listed_game.clone().initiator_token_id;
                        ACTIVE_GAMES_STORE
                            .add_suffix(initiator_token.as_bytes())
                            .remove(deps.storage, &game_id)?;

                        // remove game from listed games
                        LISTED_GAMES_STORE
                            .remove(deps.storage, &game_id)?;
                        // add game to finished games
                        FINISHED_GAMES_STORE
                            .insert(deps.storage, &game_id, &listed_game)?;
                    } else {
                        TURN_STATE_STORE
                            .add_suffix(game_id.as_bytes())
                            .save(deps.storage, &(TurnState::InitiatorsTurn as u8))?;
                    }

                    away = joiner_away.away_values;
                    opponent_owner = listed_game.initiator_owner;
                    opponent_home = initiator_home;
                } else {
                    return Err(StdError::generic_err("Error reading opponent home from storage"));
                }
            } else {
                return Err(StdError::generic_err("Error reading away from storage"));
            }
        } else {
            return Err(StdError::generic_err("Not your turn to attack"));
        }
    } else {
        return Err(StdError::generic_err("Invalid game state"));
    }

    LAST_MOVE_TIME_STORE
        .add_suffix(game_id.as_bytes())
        .save(deps.storage, &env.block.time.seconds())?;

    // handle snip-52 channel data
    let channel = GAME_UPDATED_CHANNEL_ID.to_string();

    // get notification id for recipient (opponent owner)
    let id = notification_id(deps.storage, &opponent_owner, &channel)?;

    let turn = TURN_STATE_STORE
        .add_suffix(game_id.as_bytes())
        .load(deps.storage)?;

    // use CBOR to encode data
    let data = cbor::to_vec(&(
        game_id.clone(),
        opponent_home,
        turn,
    )).map_err(|e| 
        StdError::generic_err(format!("{:?}", e))
    )?;

    // encrypt the message
    let encrypted_data = encrypt_notification_data(
        deps.storage,
        &env,
        &sender,
        &opponent_owner,
        &channel,
        data.clone()
    )?;

    increment_count(deps.storage, &channel, &opponent_owner)?;

    let response: Response;
    if winner {
        response = Response::new()
            .set_data(
                to_binary(&ExecuteAnswer::AttackCell { away, turn }
            )?)        
            .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: sender.clone().into_string(),
                amount: if listed_game.wager == 0 { vec![] } else {
                    vec![
                        Coin {
                            denom: "uscrt".to_string(),
                            amount: Uint128::from((listed_game.wager * 2) - 1000000_u128),
                        }
                    ]
                },
            }))
            .add_attribute_plaintext(
                id.to_base64(), 
                encrypted_data.to_base64()
            );
    } else {
        response = Response::new()
            .set_data(
                to_binary(&ExecuteAnswer::AttackCell { away, turn }
            )?)
            .add_attribute_plaintext(
                id.to_base64(), 
                encrypted_data.to_base64()
            );
    }

    Ok(response)
}

pub fn claim_victory(
    deps: DepsMut,
    env: Env,
    sender: &Addr,
    config: &Config,
    token_id: String,
    game_id: String,
) -> StdResult<Response> {
    let _token_owner = verify_owner_or_delegate(
        deps.storage,
        &deps.api.addr_canonicalize(sender.as_str())?,
        &config,
        &token_id
    )?;

    // check if game id exists
    let listed_game = LISTED_GAMES_STORE.get(deps.storage, &game_id);
    if listed_game.is_none() {
        return Err(StdError::generic_err("No listed game with that id"));
    }
    let listed_game = listed_game.unwrap();

    // identify if initiator or joiner (or neither)
    let initiator: bool;
    if token_id == listed_game.initiator_token_id {
        initiator = true;
    } else {
        let joiner = JOINER_TOKEN_STORE.add_suffix(game_id.as_bytes()).may_load(deps.storage)?;
        if let Some(joiner) = joiner {
            if token_id == joiner {
                initiator = false;
            } else {
                return Err(StdError::generic_err("Unauthorized"));
            }
        } else {
            return Err(StdError::generic_err("Unauthorized"));
        }
    }

    // identify if turn is one where you can claim victory
    let turn = TURN_STATE_STORE
        .add_suffix(game_id.as_bytes())
        .load(deps.storage)?;

    if turn == TurnState::WaitingForBothPlayersSetup as u8 ||
       turn == TurnState::GameOverInitiatorWon as u8 ||
       turn == TurnState::GameOverJoinerWon as u8 {
        return Err(StdError::generic_err("Cannot claim victory this turn"));
    }

    if initiator && (
        turn == TurnState::WaitingForInitiatorSetup as u8 ||
        turn == TurnState::InitiatorsTurn as u8
    ) {
        return Err(StdError::generic_err("Cannot claim victory this turn"));
    } else if !initiator && (
        turn == TurnState::WaitingForJoinerSetup as u8 ||
        turn == TurnState::JoinersTurn as u8
    ) {
        return Err(StdError::generic_err("Cannot claim victory this turn"));
    }

    let bank_msg: CosmosMsg;
    let mut id: Option<Binary> = None;
    let mut encrypted_data: Option<Binary> = None;
    if initiator && turn == TurnState::WaitingForPlayer as u8 {
        // can always pull out of game before someone joins
        ACTIVE_GAMES_STORE
            .add_suffix(listed_game.initiator_token_id.as_bytes())
            .remove(deps.storage, &game_id)?;
        TURN_STATE_STORE
            .add_suffix(game_id.as_bytes())
            .save(deps.storage, &(TurnState::GameOverInitiatorWon as u8))?;
        // remove game from listed games
        LISTED_GAMES_STORE
            .remove(deps.storage, &game_id)?;
        // add game to finished games
        FINISHED_GAMES_STORE
            .insert(deps.storage, &game_id, &listed_game)?;

        bank_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: sender.clone().into_string(),
            amount: vec![
                Coin {
                    denom: "uscrt".to_string(),
                    amount: Uint128::from(listed_game.wager),
                }
            ],
        });
    } else {
        let time = env.block.time.seconds();
        let last_move_time = LAST_MOVE_TIME_STORE
            .add_suffix(game_id.as_bytes())
            .load(deps.storage)?;
        if time < last_move_time + TIMEOUT_SEC {
            return Err(StdError::generic_err("Not enough time elapsed to claim victory"));
        }

        ACTIVE_GAMES_STORE
            .add_suffix(listed_game.initiator_token_id.as_bytes())
            .remove(deps.storage, &game_id)?;
        let joiner_token = JOINER_TOKEN_STORE
            .add_suffix(game_id.as_bytes())
            .load(deps.storage)?;
        ACTIVE_GAMES_STORE
            .add_suffix(joiner_token.as_bytes())
            .remove(deps.storage, &game_id)?;

        let opponent_owner: CanonicalAddr;
        let opponent_home: Vec<u8>;
        if initiator {
            TURN_STATE_STORE
                .add_suffix(game_id.as_bytes())
                .save(deps.storage, &(TurnState::GameOverInitiatorWon as u8))?;
            opponent_owner = JOINER_OWNER_STORE
                .add_suffix(game_id.as_bytes())
                .load(deps.storage)?;
            opponent_home = JOINER_HOME_STORE
                .add_suffix(game_id.as_bytes())
                .load(deps.storage)?;
        } else {
            TURN_STATE_STORE
                .add_suffix(game_id.as_bytes())
                .save(deps.storage, &(TurnState::GameOverJoinerWon as u8))?;
            opponent_owner = listed_game.clone().initiator_owner;
            opponent_home = INITIATOR_HOME_STORE
                .add_suffix(game_id.as_bytes())
                .load(deps.storage)?;
        }

        // remove game from listed games
        LISTED_GAMES_STORE
            .remove(deps.storage, &game_id)?;
        // add game to finished games
        FINISHED_GAMES_STORE
            .insert(deps.storage, &game_id, &listed_game)?;

        bank_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: sender.clone().into_string(),
            amount: vec![
                Coin {
                    denom: "uscrt".to_string(),
                    amount: Uint128::from(listed_game.wager * 2),
                }
            ],
        });

        // handle snip-52 channel data
        let channel = GAME_UPDATED_CHANNEL_ID.to_string();

        // get notification id for recipient (opponent owner)
        id = Some(notification_id(deps.storage, &opponent_owner, &channel)?);

        // use CBOR to encode data
        let data = cbor::to_vec(&(
            game_id.clone(),
            opponent_home,
            TURN_STATE_STORE
                .add_suffix(game_id.as_bytes())
                .load(deps.storage)?,
        )).map_err(|e| 
            StdError::generic_err(format!("{:?}", e))
        )?;

        // encrypt the message
        encrypted_data = Some(encrypt_notification_data(
            deps.storage,
            &env,
            &sender,
            &opponent_owner,
            &channel,
            data.clone()
        )?);

        increment_count(deps.storage, &channel, &opponent_owner)?;
    }

    let response;

    if id.is_some() {
        response = Response::new()
            .set_data(to_binary(&ExecuteAnswer::ClaimVictory { 
                status: ResponseStatus::Success 
            })?)
            .add_message(bank_msg)
            .add_attribute_plaintext(
                id.unwrap().to_base64(), 
                encrypted_data.unwrap().to_base64()
            );
    } else {
        response = Response::new()
            .set_data(to_binary(&ExecuteAnswer::ClaimVictory { 
                status: ResponseStatus::Success 
            })?)
            .add_message(bank_msg)
    }

    Ok(response)
}

pub fn query_list_games(
    deps: Deps,
    token_id: String,
    page: Option<u32>,
    page_size: Option<u32>,
    address_raw: &CanonicalAddr,
) -> StdResult<Binary> {
    let config: Config = load(deps.storage, CONFIG_KEY)?;
    let _token_owner = verify_owner_or_delegate(
        deps.storage,
        address_raw,
        &config,
        &token_id
    )?;

    let page = page.unwrap_or(0_u32);
    let page_size = page_size.unwrap_or(20_u32);
    let games: Vec<ListedGame> = LISTED_GAMES_STORE
        .paging(deps.storage, page, page_size)?
        .into_iter()
        .map(|(game_id, stored_game)| ListedGame {
            game_id,
            wager: Coin {
                denom: DENOM.to_string(),
                amount: Uint128::from(stored_game.wager),
            },
            title: stored_game.title,
            created: stored_game.created,
        })
        .collect();
    to_binary(&QueryAnswer::ListGames { games })
}

pub fn query_active_games(
    deps: Deps,
    token_id: String,
    address_raw: &CanonicalAddr,
) -> StdResult<Binary> {
    let config: Config = load(deps.storage, CONFIG_KEY)?;
    let _token_owner = verify_owner_or_delegate(
        deps.storage,
        address_raw,
        &config,
        &token_id
    )?;
    let game_ids = ACTIVE_GAMES_STORE
        .add_suffix(token_id.as_bytes())
        .iter(deps.storage)?
        .map(|game_id| game_id.unwrap())
        .collect();
        
    to_binary(&QueryAnswer::ActiveGames { game_ids })
}

pub fn query_game_state(
    deps: Deps,
    token_id: String,
    game_id: String,
    address_raw: &CanonicalAddr,
) -> StdResult<Binary> {
    let config: Config = load(deps.storage, CONFIG_KEY)?;
    let _token_owner = verify_owner_or_delegate(
        deps.storage,
        address_raw,
        &config,
        &token_id
    )?;

    let mut listed_game = LISTED_GAMES_STORE
        .get(deps.storage, &game_id);
    if listed_game.is_none() {
        //check if it is a finished game instead
        listed_game = FINISHED_GAMES_STORE
            .get(deps.storage, &game_id);
        if listed_game.is_none() {
            return Err(StdError::generic_err("Game is not listed or finished"));
        }
    }
    let listed_game = listed_game.unwrap();
    let joiner_token = JOINER_TOKEN_STORE
        .add_suffix(game_id.as_bytes())
        .may_load(deps.storage)?;
    let role: PlayerRole;
    let home: Vec<u8>;
    let away: Vec<u8>;
    if token_id == listed_game.initiator_token_id {
        role = PlayerRole::Initiator;
        home = INITIATOR_HOME_STORE
            .add_suffix(game_id.as_bytes())
            .may_load(deps.storage)?
            .unwrap_or(vec![CellValue::Empty as u8; BOARD_SIZE]);
        let stored_away = INITIATOR_AWAY_STORE
            .add_suffix(game_id.as_bytes())
            .may_load(deps.storage)?;
        if stored_away.is_some() {
            away = stored_away.unwrap().away_values;
        } else {
            away = vec![CellValue::Empty as u8; BOARD_SIZE];
        }
    } else if Some(token_id) == joiner_token {
        role = PlayerRole::Joiner;
        home = JOINER_HOME_STORE
            .add_suffix(game_id.as_bytes())
            .may_load(deps.storage)?
            .unwrap_or(vec![CellValue::Empty as u8; BOARD_SIZE]);
        let stored_away = JOINER_AWAY_STORE
            .add_suffix(game_id.as_bytes())
            .may_load(deps.storage)?;
        if stored_away.is_some() {
            away = stored_away.unwrap().away_values;
        } else {
            away = vec![CellValue::Empty as u8; BOARD_SIZE];
        }
    } else {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let turn = TURN_STATE_STORE
        .add_suffix(game_id.as_bytes())
        .load(deps.storage)?;

    let game = listed_game;
    let wager = Coin {
        denom: "uscrt".to_string(),
        amount: Uint128::from(game.wager)
    };

    to_binary(&QueryAnswer::GameState { 
        role: role as u8, 
        turn: turn, 
        home, 
        away,
        game_id,
        wager,
        title: game.title,
        created: game.created,
    })
}

// STATE

/// a listed game
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredListedGame {
    pub title: String,
    pub wager: u128,
    pub created: Timestamp,
    pub initiator_token_id: String, // token id
    pub initiator_owner: CanonicalAddr, // owner of token
    pub initiator_goes_first: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredAway {
    pub away_values: Vec<u8>,
    pub carrier_hits: u8,
    pub battleship_hits: u8,
    pub cruiser_hits: u8,
    pub submarine_hits: u8,
    pub destroyer_hits: u8,
}

// prefix game_id. value is TurnState
pub static TURN_STATE_STORE: Item<u8> = Item::new(b"turn-state");
// prefix game_id. value is joiner token_id
pub static JOINER_TOKEN_STORE: Item<String> = Item::new(b"game-joiner-tok");
// prefix game_id. value is joiner token owner
pub static JOINER_OWNER_STORE: Item<CanonicalAddr> = Item::new(b"game-joiner-owner");
// prefix game_id. value is initiator home board
pub static INITIATOR_HOME_STORE: Item<Vec<u8>> = Item::new(b"initiator-home");
// prefix game_id. value is joiner home board
pub static JOINER_HOME_STORE: Item<Vec<u8>> = Item::new(b"joiner-home");
// prefix game_id. value is initiator away board
pub static INITIATOR_AWAY_STORE: Item<StoredAway> = Item::new(b"initiator-away");
// prefix game_id. value is joiner away board
pub static JOINER_AWAY_STORE: Item<StoredAway> = Item::new(b"joiner-away");
// game_id -> listed game
pub static LISTED_GAMES_STORE: Keymap<String, StoredListedGame> = Keymap::new(b"listed-games");
// game_id -> finished game
pub static FINISHED_GAMES_STORE: Keymap<String, StoredListedGame> = Keymap::new(b"finished-games");
// set of active game_ids for prefix token_id
pub static ACTIVE_GAMES_STORE: Keyset<String> = Keyset::new(b"active-games");
// prefix game_id. value is last move timestamp
pub static LAST_MOVE_TIME_STORE: Item<u64> = Item::new(b"last-move");


// testing

#[cfg(test)]
mod tests {
    use std::any::Any;

    use cosmwasm_std::{testing::*, Coin, Uint128};
    use cosmwasm_std::{
        from_binary, Binary, OwnedDeps,
        Response, StdError, StdResult,
    };
    use crate::battleship::valid_setup;
    use crate::contract::{execute, instantiate,};
    use crate::msg::{
        ExecuteAnswer, ExecuteMsg, InstantiateConfig,
        InstantiateMsg, 
    };

    // Helper functions

    fn init_helper_default() -> (
        StdResult<Response>,
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("instantiator", &[]);
        let init_msg = InstantiateMsg {
            name: "sec721".to_string(),
            symbol: "S721".to_string(),
            admin: Some("admin".to_string()),
            entropy: "We're going to need a bigger boat".to_string(),
            royalty_info: None,
            config: None,
            post_init_callback: None,
        };

        (instantiate(deps.as_mut(), env, info, init_msg), deps)
    }

    fn init_helper_with_config(
        public_token_supply: bool,
        public_owner: bool,
        enable_sealed_metadata: bool,
        unwrapped_metadata_is_private: bool,
        minter_may_update_metadata: bool,
        owner_may_update_metadata: bool,
        enable_burn: bool,
        minter_may_put_token_storage: bool,
        minter_may_put_global_storage: bool,
    ) -> (
        StdResult<Response>,
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
    ) {
        let mut deps = mock_dependencies();

        let env = mock_env();
        let init_config: InstantiateConfig = from_binary(&Binary::from(
            format!(
                "{{\"public_token_supply\":{},
                \"public_owner\":{},
                \"enable_sealed_metadata\":{},
                \"unwrapped_metadata_is_private\":{},
                \"minter_may_update_metadata\":{},
                \"owner_may_update_metadata\":{},
                \"enable_burn\":{},
                \"minter_may_put_token_storage\":{},
                \"minter_may_put_global_storage\":{}}}",
                public_token_supply,
                public_owner,
                enable_sealed_metadata,
                unwrapped_metadata_is_private,
                minter_may_update_metadata,
                owner_may_update_metadata,
                enable_burn,
                minter_may_put_token_storage,
                minter_may_put_global_storage,
            )
            .as_bytes(),
        ))
        .unwrap();
        let info = mock_info("instantiator", &[]);
        let init_msg = InstantiateMsg {
            name: "sec821".to_string(),
            symbol: "S821".to_string(),
            admin: Some("admin".to_string()),
            entropy: "We're going to need a bigger boat".to_string(),
            royalty_info: None,
            config: Some(init_config),
            post_init_callback: None,
        };

        (instantiate(deps.as_mut(), env, info, init_msg), deps)
    }

    fn extract_error_msg<T: Any>(error: StdResult<T>) -> String {
        match error {
            Ok(_response) => panic!("Expected error, but had Ok response"),
            Err(err) => match err {
                StdError::GenericErr { msg, .. } => msg,
                _ => panic!("Unexpected error result {:?}", err),
            },
        }
    }

    fn extract_log(resp: StdResult<Response>) -> String {
        match resp {
            Ok(response) => response.attributes[0].value.clone(),
            Err(_err) => "These are not the logs you are looking for".to_string(),
        }
    }

    #[test]
    fn test_new_game() {
        let (init_result, mut deps) =
            init_helper_with_config(false, false, false, false, true, false, false, false, false);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let execute_msg = ExecuteMsg::NewGame { 
            token_id: "token 1".to_string(),
            title: "game 1".to_string(),
            padding: None
        };
        /*
        let exec_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[
                Coin{
                    denom: "uscrt".to_string(),
                    amount: Uint128::from(1000000_u128),
                }
            ]),
            execute_msg,
        );
        let exec_answer: ExecuteAnswer = from_binary(&exec_result.unwrap().data.unwrap()).unwrap();
        println!("{:?}", exec_answer);
        */
    }

    #[test]
    fn test_valid_setup() {
        let setup: Vec<u8> = vec![
            0,0,0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
        ];
        assert!(!valid_setup(&setup));
        let setup: Vec<u8> = vec![
            0,0,0,0,0,2,2,2,2,2,
            0,0,0,0,0,0,0,0,0,3,
            0,0,0,0,0,0,0,0,0,3,
            0,0,0,0,0,0,0,0,0,3,
            0,0,0,0,0,0,0,0,0,3,
            4,4,4,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
            0,0,5,0,0,0,0,0,0,0,
            0,0,5,0,0,0,0,0,0,0,
            0,0,5,0,0,0,0,0,6,6,
        ];
        assert!(valid_setup(&setup));
        let setup: Vec<u8> = vec![
            0,0,0,0,0,2,2,2,2,2,
            0,0,0,0,0,0,2,0,0,3,
            0,0,0,0,0,0,2,0,0,3,
            0,0,0,0,0,0,2,0,0,3,
            0,0,0,0,0,0,2,0,0,3,
            4,4,4,0,0,0,2,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
            0,0,5,0,0,0,0,0,0,0,
            0,0,5,0,0,0,0,0,0,0,
            0,0,5,0,0,0,0,0,6,6,
        ];
        assert!(!valid_setup(&setup));
        let setup: Vec<u8> = vec![
            0,0,0,0,0,2,2,2,2,2,
            0,0,0,0,0,0,0,0,0,3,
            0,0,0,0,0,0,0,0,0,3,
            0,0,0,0,0,0,0,0,0,3,
            0,0,0,0,0,0,0,0,0,3,
            4,4,4,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
            0,0,5,0,0,0,0,0,0,0,
            0,0,5,0,0,0,0,0,0,0,
            0,0,5,0,0,0,0,0,0,0,
        ];
        assert!(!valid_setup(&setup));
        let setup: Vec<u8> = vec![
            0,0,0,0,0,2,0,0,0,0,
            0,0,0,0,0,2,0,0,0,3,
            0,0,0,0,0,2,0,0,0,3,
            0,0,0,0,0,2,0,0,0,3,
            0,0,0,0,0,2,0,0,0,3,
            4,0,0,0,0,0,0,0,0,0,
            4,0,0,0,0,0,0,0,0,0,
            4,0,5,6,0,0,0,0,0,0,
            0,0,5,6,0,0,0,0,0,0,
            0,0,5,0,0,0,0,0,0,0,
        ];
        assert!(valid_setup(&setup));
        let setup: Vec<u8> = vec![
            0,0,0,0,0,0,2,2,2,2,
            2,0,0,0,0,0,0,0,0,3,
            0,0,0,0,0,0,0,0,0,3,
            0,0,0,0,0,0,0,0,0,3,
            0,0,0,0,0,0,0,0,0,3,
            4,0,0,0,0,0,0,0,0,0,
            4,0,0,0,0,0,0,0,0,0,
            4,0,5,6,0,0,0,0,0,0,
            0,0,5,6,0,0,0,0,0,0,
            0,0,5,0,0,0,0,0,0,0,
        ];
        assert!(!valid_setup(&setup));
        let setup: Vec<u8> = vec![
            0,0,0,0,0,2,2,2,2,2,
            3,3,3,3,4,4,4,5,5,5,
            6,6,0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,0,0,
        ];
        assert!(valid_setup(&setup));
    }
}