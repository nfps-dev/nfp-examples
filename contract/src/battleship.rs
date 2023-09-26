use rand_core::RngCore;
use secret_toolkit::{storage::{Keyset, Item, Keymap}, crypto::ContractPrng};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use cosmwasm_std::{Coin, Timestamp, DepsMut, Addr, StdResult, Response, to_binary, Uint128, Deps, Binary, StdError, CanonicalAddr, MessageInfo, Env};

use crate::msg::{ExecuteAnswer, ResponseStatus, QueryAnswer};

pub const BOARD_SIZE: u8 = 100_u8; // 100
pub const VALID_WAGERS: [u128; 5] = [0_u128, 1000000_u128, 2000000_u128, 5000000_u128, 10000000_u128];
pub const CARRIER_SIZE: u8 = 5;
pub const BATTLESHIP_SIZE: u8 = 4;
pub const CRUISER_SIZE: u8 = 3;
pub const SUBMARINE_SIZE: u8 = 3;
pub const DESTROYER_SIZE: u8 = 2;

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
pub enum GameState {
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
    /// part of the "Carrier" vessel occupies the cell
    Carrier = 0,
    /// part of the "Battleship" vessel occupies the cell
    Battleship = 1,
    /// part of the "Cruiser" vessel occupies the cell
    Cruiser = 2,
    /// part of the "Submarine" vessel occupies the cell
    Submarine = 3,
    /// part of the "Destroyer" vessel occupies the cell
    Destroyer = 4,
    /// nothing occupies the cell. used for both `board` and `tracking` grids
    Empty = 5,
    /// for `tracking` grid only: player missed the cell
    Miss = 6,
    /// for `tracking grid only: player hit a vessel but has not yet sunk it
    HitUnknown = 7,
}

/// Used to represent a game to prospective players browsing the lobby
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, JsonSchema)]
pub struct NewGame {
    pub game_id: String,
    pub wager: Coin,
    pub title: String,
    pub created: Timestamp,
}

fn valid_setup(
    cells: &Vec<CellValue>,
) -> bool {
    if cells.len() != BOARD_SIZE as usize {
        return false; // The board should have exactly 100 cells.
    }

    let mut counts = [0; 6]; // Count of each ship type and empty cells.

    // Check for each ship type and count the occurrences.
    for cell in cells {
        match *cell as u8 {
            0 => counts[0] += 1, // Carrier
            1 => counts[1] += 1, // Battleship
            2 => counts[2] += 1, // Cruiser
            3 => counts[3] += 1, // Submarine
            4 => counts[4] += 1, // Destroyer
            5 => counts[5] += 1, // Empty cell
            _ => return false,   // Invalid cell value
        }
    }

    // Check if the counts of each ship type are valid.
    if counts[0] != CARRIER_SIZE || 
       counts[1] != BATTLESHIP_SIZE || 
       counts[2] != CRUISER_SIZE || 
       counts[3] != SUBMARINE_SIZE || 
       counts[4] != DESTROYER_SIZE || 
       counts[5] != BOARD_SIZE - CARRIER_SIZE - BATTLESHIP_SIZE - CRUISER_SIZE - SUBMARINE_SIZE - DESTROYER_SIZE {
        return false;
    }

    // Check if the ships are adjacent in a straight line.
    for ship_type in 0..5 {
        let mut found_start = false;

        for (i, cell) in cells.iter().enumerate() {
            if *cell as u8 == ship_type {
                if found_start {
                    return false; // Ship cells are not adjacent.
                }
                found_start = true;

                // Check horizontally
                if i % 10 < 10 - counts[ship_type as usize] as usize {
                    for j in 0..counts[ship_type as usize] as usize {
                        if cells[i + j] as u8 != ship_type {
                            return false; // Ship cells are not in a straight line.
                        }
                    }
                }

                // Check vertically
                if i / 10 < 10 - counts[ship_type as usize] as usize {
                    for j in 0..counts[ship_type as usize] as usize {
                        if cells[i + j * 10] as u8 != ship_type {
                            return false; // Ship cells are not in a straight line.
                        }
                    }
                }
            }
        }
    }

    return true;
}

pub fn new_game(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
) -> StdResult<Response> {
    let created = env.block.time.clone();
    let mut wager = 0_u128;
    if info.funds.len() == 1 {
        if info.funds[0].denom != "uscrt" {
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
    let game_id = base32::encode(base32::Alphabet::Crockford, &prng.rand_bytes());

    ACTIVE_GAMES_STORE.insert(
        deps.storage, 
        &game_id, 
        &StoredActiveGame {
            title: title.clone(),
            wager, 
            created,
            initiator: deps.api.addr_canonicalize(info.sender.as_str())?, 
            initiator_goes_first
        },
    )?;

    GAME_STATE_STORE
        .add_suffix(game_id.as_bytes())
        .save(deps.storage, &(GameState::WaitingForPlayer as u8))?;

    let game = NewGame { 
        game_id, 
        wager: info.funds[0].clone(), 
        title, 
        created 
    };

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::NewGame {
            game
        })?),
    )
}

pub fn join_game(
    deps: DepsMut,
    sender: &Addr,
    game_id: String,
) -> StdResult<Response> {
    // check if game id exists
    let active_game = ACTIVE_GAMES_STORE.get(deps.storage, &game_id);
    if active_game.is_none() {
        return Err(StdError::generic_err("No active game with that id"));
    }
    // check if there is already a joiner
    let joiner_addr = deps.api.addr_canonicalize(sender.as_str())?;
    let joiner = JOINER_STORE.add_suffix(game_id.as_bytes()).may_load(deps.storage)?;
    if joiner.is_some() {
        return Err(StdError::generic_err("There is already a joiner for this game"));
    }
    JOINER_STORE
        .add_suffix(game_id.as_bytes())
        .save(deps.storage, &joiner_addr)?;

    if let Some(game_state) = GAME_STATE_STORE.add_suffix(game_id.as_bytes()).may_load(deps.storage)? {
        if game_state != GameState::WaitingForPlayer as u8 {
            return Err(StdError::generic_err("Game state is not waiting for player"));
        }
    } else {
        return Err(StdError::generic_err("Invalid game state"));
    }
    GAME_STATE_STORE
        .add_suffix(game_id.as_bytes())
        .save(deps.storage, &(GameState::WaitingForBothPlayersSetup as u8))?;

    let active_game: StoredActiveGame = active_game.unwrap();


    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::JoinGame { 
            status: ResponseStatus::Success 
        })?)
    )
}

pub fn submit_setup(
    deps: DepsMut,
    sender: &Addr,
    game_id: String,
    ready: Option<bool>,
    cells: Vec<CellValue>,
) -> StdResult<Response> {
    if !valid_setup(&cells) {
        return Err(StdError::generic_err("Not a valid battleship setup"));
    }
    // ... TODO
    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::SubmitSetup { 
            status: ResponseStatus::Success 
        })?)
    )
}

pub fn attack_cell(
    deps: DepsMut,
    sender: &Addr,
    game_id: String,
    cell: u8,
) -> StdResult<Response> {
    // ... TODO
    let result = CellValue::Miss;
    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::AttackCell { 
            result 
        })?)
    )
}

pub fn claim_victory(
    deps: DepsMut,
    env: Env,
    sender: &Addr,
    game_id: String,
) -> StdResult<Response> {
    // ... TODO
    let time = env.block.time;
    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::ClaimVictory { 
            status: ResponseStatus::Success 
        })?)
    )
}

pub fn query_list_games(
    deps: Deps,
    page: Option<u32>,
    page_size: Option<u32>,
) -> StdResult<Binary> {
    // ... TODO
    let games: Vec<NewGame> = vec![];
    to_binary(&QueryAnswer::ListGames { games })
}

pub fn query_game_state(
    deps: Deps,
    game_id: String,
) -> StdResult<Binary> {
    // ... TODO
    to_binary(&QueryAnswer::GameState { 
        role: PlayerRole::Initiator, 
        state: GameState::InitiatorsTurn, 
        tracking: vec![], 
        board: vec![] 
    })
}

// STATE

/// an active game
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredActiveGame {
    pub title: String,
    pub wager: u128,
    pub created: Timestamp,
    pub initiator: CanonicalAddr,
    pub initiator_goes_first: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredAttacks {
    pub attack_values: Vec<u8>,
    pub carrier_hits: u8,
    pub battleship_hits: u8,
    pub cruiser_hits: u8,
    pub submarine_hits: u8,
    pub destroyer_hits: u8,
}

pub static GAME_STATE_STORE: Item<u8> = Item::new(b"game-state");
pub static JOINER_STORE: Item<CanonicalAddr> = Item::new(b"game-joiner");
pub static INITIATOR_SETUP_STORE: Item<Vec<u8>> = Item::new(b"initiator-setup");
pub static JOINER_SETUP_STORE: Item<Vec<u8>> = Item::new(b"joiner-setup");
pub static INITIATOR_ATTACKS_STORE: Item<StoredAttacks> = Item::new(b"initiator-attacks");
pub static JOINER_ATTACKS_STORE: Item<StoredAttacks> = Item::new(b"joiner-attacks");
// game_id -> active game
pub static ACTIVE_GAMES_STORE: Keymap<String, StoredActiveGame> = Keymap::new(b"active-games");
