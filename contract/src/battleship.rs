use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use cosmwasm_std::{Coin, Timestamp};

/// Distinguishes to a player which role they fulfil
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, JsonSchema)]
pub enum PlayerRole {
    /// this player initiated the game
    Initiator = 0,
    /// this player joined the game
    Joiner = 1,
}

/// Describes the state of an initiated game (fits into u8)
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, JsonSchema)]
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
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, JsonSchema)]
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
