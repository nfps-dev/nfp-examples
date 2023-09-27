use std::convert::TryFrom;
use rand_core::RngCore;
use secret_toolkit::{storage::{Keyset, Item, Keymap}, crypto::ContractPrng};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use cosmwasm_std::{Coin, Timestamp, DepsMut, Addr, StdResult, Response, to_binary, Uint128, Deps, Binary, StdError, CanonicalAddr, MessageInfo, Env};

use crate::msg::{ExecuteAnswer, ResponseStatus, QueryAnswer};

pub const DENOM: &str = "uscrt";
pub const BOARD_SIZE: usize = 100; // 100
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

fn valid_setup(
    cells: &Vec<u8>,
) -> bool {
    if cells.len() != BOARD_SIZE as usize {
        return false; // The board should have exactly 100 cells.
    }

    let mut counts = [0; 6]; // Count of each ship type and empty cells.

    // Check for each ship type and count the occurrences.
    for cell in cells {
        match *cell {
            2 => counts[0] += 1, // Carrier
            3 => counts[1] += 1, // Battleship
            4 => counts[2] += 1, // Cruiser
            5 => counts[3] += 1, // Submarine
            6 => counts[4] += 1, // Destroyer
            0 => counts[5] += 1, // Empty cell
            _ => return false,   // Invalid cell value
        }
    }

    // Check if the counts of each ship type are valid.
    if counts[0] != CARRIER_SIZE || 
       counts[1] != BATTLESHIP_SIZE || 
       counts[2] != CRUISER_SIZE || 
       counts[3] != SUBMARINE_SIZE || 
       counts[4] != DESTROYER_SIZE || 
       counts[5] != BOARD_SIZE as u8 - CARRIER_SIZE - BATTLESHIP_SIZE - CRUISER_SIZE - SUBMARINE_SIZE - DESTROYER_SIZE {
        return false;
    }

    // Check if the ships are adjacent in a straight line.
    for ship_type in 0..5 {
        let mut found_start = false;

        for (i, cell) in cells.iter().enumerate() {
            if *cell == ship_type {
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

    true
}

pub fn listed_game(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
) -> StdResult<Response> {
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
    let game_id = base32::encode(base32::Alphabet::Crockford, &prng.rand_bytes());
    //

    LISTED_GAMES_STORE.insert(
        deps.storage, 
        &game_id, 
        &StoredListedGame {
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

    let game = ListedGame { 
        game_id, 
        wager: Coin {
            denom: DENOM.to_string(),
            amount: Uint128::from(wager),
        }, 
        title, 
        created 
    };

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::ListedGame {
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
    let listed_game = LISTED_GAMES_STORE.get(deps.storage, &game_id);
    if listed_game.is_none() {
        return Err(StdError::generic_err("No listed game with that id"));
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

    let listed_game: StoredListedGame = listed_game.unwrap();


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
    cells: Vec<u8>,
) -> StdResult<Response> {
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
        true => GameState::InitiatorsTurn,
        false => GameState::JoinersTurn,
    };

    let sender_raw = deps.api.addr_canonicalize(sender.as_str())?;
    let initiator: bool;
    if sender_raw == listed_game.initiator {
        initiator = true;
    } else {
        let joiner = JOINER_STORE.add_suffix(game_id.as_bytes()).may_load(deps.storage)?;
        if let Some(joiner) = joiner {
            if joiner == listed_game.initiator {
                initiator = false;
            } else {
                return Err(StdError::generic_err("Unauthorized"));
            }
        } else {
            return Err(StdError::generic_err("Unauthorized"));
        }
    }

    if let Some(game_state) = GAME_STATE_STORE.add_suffix(game_id.as_bytes()).may_load(deps.storage)? {
        if initiator {
            if game_state == GameState::WaitingForBothPlayersSetup as u8 {
                GAME_STATE_STORE
                    .add_suffix(game_id.as_bytes())
                    .save(deps.storage, &(GameState::WaitingForJoinerSetup as u8))?;
            } else if game_state == GameState::WaitingForInitiatorSetup as u8 {
                GAME_STATE_STORE
                    .add_suffix(game_id.as_bytes())
                    .save(deps.storage, &(first_mover_turn as u8))?;
            } else {
                return Err(StdError::generic_err("Game state is not at submit setup"));
            }

            INITIATOR_HOME_STORE
                .add_suffix(game_id.as_bytes())
                .save(deps.storage, &cells)?;
            INITIATOR_AWAY_STORE
                .add_suffix(game_id.as_bytes())
                .save(
                    deps.storage,
                    &StoredTracking {
                        tracking_values: vec![0_u8; BOARD_SIZE],
                        carrier_hits: 0,
                        battleship_hits: 0,
                        cruiser_hits: 0,
                        submarine_hits: 0,
                        destroyer_hits: 0,
                    }
                )?;
        } else { // joiner
            if game_state == GameState::WaitingForBothPlayersSetup as u8 {
                GAME_STATE_STORE
                    .add_suffix(game_id.as_bytes())
                    .save(deps.storage, &(GameState::WaitingForInitiatorSetup as u8))?;
            } else if game_state == GameState::WaitingForJoinerSetup as u8 {
                GAME_STATE_STORE
                    .add_suffix(game_id.as_bytes())
                    .save(deps.storage, &(first_mover_turn as u8))?;
            } else {
                return Err(StdError::generic_err("Game state is not at submit setup"));
            }

            JOINER_HOME_STORE
                .add_suffix(game_id.as_bytes())
                .save(deps.storage, &cells)?;
            JOINER_AWAY_STORE
                .add_suffix(game_id.as_bytes())
                .save(
                    deps.storage,
                    &StoredTracking {
                        tracking_values: vec![0_u8; BOARD_SIZE],
                        carrier_hits: 0,
                        battleship_hits: 0,
                        cruiser_hits: 0,
                        submarine_hits: 0,
                        destroyer_hits: 0,
                    }
                )?;
        }
    } else {
        return Err(StdError::generic_err("Invalid game state"));
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

pub fn query_game_state(
    deps: Deps,
    game_id: String,
    address_raw: CanonicalAddr,
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
pub struct StoredListedGame {
    pub title: String,
    pub wager: u128,
    pub created: Timestamp,
    pub initiator: CanonicalAddr,
    pub initiator_goes_first: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredTracking {
    pub tracking_values: Vec<u8>,
    pub carrier_hits: u8,
    pub battleship_hits: u8,
    pub cruiser_hits: u8,
    pub submarine_hits: u8,
    pub destroyer_hits: u8,
}

pub static GAME_STATE_STORE: Item<u8> = Item::new(b"game-state");
pub static JOINER_STORE: Item<CanonicalAddr> = Item::new(b"game-joiner");
pub static INITIATOR_HOME_STORE: Item<Vec<u8>> = Item::new(b"initiator-home");
pub static JOINER_HOME_STORE: Item<Vec<u8>> = Item::new(b"joiner-home");
pub static INITIATOR_AWAY_STORE: Item<StoredTracking> = Item::new(b"initiator-away");
pub static JOINER_AWAY_STORE: Item<StoredTracking> = Item::new(b"joiner-away");
// game_id -> listed game
pub static LISTED_GAMES_STORE: Keymap<String, StoredListedGame> = Keymap::new(b"listed-games");


// testing

#[cfg(test)]
mod tests {
    use std::any::Any;

    use cosmwasm_std::{testing::*, Coin, Uint128};
    use cosmwasm_std::{
        from_binary, to_binary, Addr, Api, Binary, OwnedDeps,
        Response, StdError, StdResult,
    };
    use crate::contract::{execute, instantiate, query,};
    use crate::msg::{
        ContractStatus, ExecuteAnswer, ExecuteMsg, InstantiateConfig,
        InstantiateMsg, Mint, QueryAnswer, QueryMsg,
        KeyValuePair, ViewerInfo, ViewerInfoAddrOpt,
    };
    use crate::nfp::{RawData, KEY_CLEARED_PACKAGES};
    use crate::token::{Extension, Metadata,};

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

        let execute_msg = ExecuteMsg::ListedGame { 
            title: "game 1".to_string(),
            padding: None
        };
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
    }

    #[test]
    fn test_valid_setup() {
        let setup: Vec<u8> = vec![

        ];
    }
}