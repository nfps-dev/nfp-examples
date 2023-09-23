use serde::{Deserialize, Serialize};
use cosmwasm_std::{Coin, Timestamp};

/// Distinguishes to a player which role they fulfil
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum PlayerRole {
    /// this player initiated the game
    Initiator = 0,
    /// this player joined the game
    Joiner = 1,
}

/// Describes the state of an initiated game (fits into u8)
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
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
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
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
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct NewGame {
    pub game_id: String,
    pub wager: Coin,
    pub title: String,
    pub created: Timestamp,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    
    /// Creates a new game in the lobby
    NewGame {
        pub padding: Option<String>,
    },
    
    /// Joins a new game that is currently waiting for another player
    JoinGame {
        pub padding: Option<String>,
        pub game_id: String,
    },
    
    /// Player submits their board setup
    SubmitSetup {
        pub padding: Option<String>,
        pub game_id: String,
        pub ready: Option<bool>,
        pub cells: Vec<CellValue>,
    },
    
    /// Player submits their move attacking an opponent's cell `w = x + (y * 10)` where w is in [0,99]
    AttackCell {
        pub padding: Option<String>,
        pub game_id: String,
        pub cell: u8,
    },
    
    /// Allows a player to claim victory once their opponent has exceeded their turn timer
    ClaimVictory {
        pub padding: Option<String>,
        pub game_id: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteAnswer {
    
    /// Creates a new game in the lobby
    NewGame {
        pub game: NewGame,
    },
    
    /// Joins a new game that is currently waiting for another player
    JoinGame {
        pub status: ResponseStatus,
    },
    
    /// Player submits their board setup
    SubmitSetup {
        pub status: ResponseStatus,
    },
    
    /// Player submits their move attacking an opponent's cell `w = x + (y * 10)` where w is in [0,99]
    AttackCell {
        pub result: CellValue,
    },
    
    /// Allows a player to claim victory once their opponent has exceeded their turn timer
    ClaimVictory {
        pub status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    
    /// Fetches a list of active games in the loby
    ListGames {
        pub page_size: Option<u32>,
        pub page: Option<u32>,
    },
    
    /// Fetches the current game state
    GameState {
        pub game_id: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    
    /// Fetches a list of active games in the loby
    ListGames {
        pub games: Vec<NewGame>,
    },
    
    /// Fetches the current game state
    GameState {
        pub role: PlayerRole,
        pub state: GameState,
        pub tracking: Vec<CellValue>,
        pub board: Vec<CellValue>,
    },
}
