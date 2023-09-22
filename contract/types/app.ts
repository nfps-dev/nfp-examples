import type {Coin, Timestamp, Uint32, Uint8} from '@solar-republic/contractor/datatypes';
import type {SecretContractInterface, Snip721} from '@solar-republic/contractor/snips';
import type {MethodDescriptorGroup, MethodGroup, Execution} from '@solar-republic/contractor/typings';

import {AccessLevel} from '@solar-republic/contractor/snip-721';

// export type * from '@solar-republic/contractor/snip-721';

// export * from '@solar-republic/contractor/snip-721';

export {AccessLevel};

type WagerAmountsScrt = '1' | '2' | '5' | '10';

/**
 * Distinguishes to a player which role they fulfil
 */
export enum PlayerRole {
	// this player initiated the game
	INITIATOR,

	// this player joined the game
	JOINER,
}

/**
 * Describes the state of an initiated game (fits into u8)
 */
export enum GameState {
	// waiting for another player to join
	WAITING_FOR_PLAYER,

	// waiting for both players to submit their setups
	WAITING_FOR_BOTH_PLAYERS_SETUP,

	// waiting for only the initiator to submit their setup
	WAITING_FOR_INITIATOR_SETUP,

	// waiting for only the joiner to submit their setup
	WAITING_FOR_JOINER_SETUP,

	// waiting for the player who initiated to submit their move
	INITIATORS_TURN,

	// waiting for the player who joined to submit their move
	JOINERS_TURN,

	// player who initiated won
	GAME_OVER_INITIATOR_WON,

	// player who joined won
	GAME_OVER_JOINER_WON,
}

/**
 * Describes the occupancy of a cell (fits into u8)
 */
export enum CellValue {
	// part of the "Carrier" vessel occupies the cell
	CARRIER,

	// part of the "Battleship" vessel occupies the cell
	BATTLESHIP,

	// part of the "Cruiser" vessel occupies the cell
	CRUISER,

	// part of the "Submarine" vessel occupies the cell
	SUBMARINE,

	// part of the "Destroyer" vessel occupies the cell
	DESTROYER,

	// nothing occupies the cell. used for both `board` and `tracking` grids
	// for the `tracking` grid: this indicates player has not yet attacked cell
	EMPTY,

	// for `tracking` grid only: player missed the cell
	MISS,

	// for `tracking grid only: player hit a vessel but has not yet sunk it
	HIT_UNKNOWN,
}


/**
 * Used to represent a game to prospective players browsing the lobby
 * @derive Serialize, Deserialize, JsonSchema
 */
export type NewGame = {
	game_id: string;
	wager: Coin;
	title: string;
	created: Timestamp;
};

// utility type that facilitates adding the same `game_id` key to each msg
type MsgsRequiresGameId<h_group extends MethodDescriptorGroup> = MethodGroup.Augment<h_group, {
	msg: {
		game_id: string;
	};
}>;

export type App = SecretContractInterface<{
	// extends: [Snip721];

	executions: {
		/**
		 * Creates a new game in the lobby
		 */
		new_game: Execution.RequireSentFunds<[{
			title?: string;
		}, {
			game: NewGame;
		}], {
			amount: '0' | `${WagerAmountsScrt}000000`;
			denom: 'uscrt';
		}>;
	}
	& MsgsRequiresGameId<{
		/**
		 * Joins a new game that is currently waiting for another player
		 */
		join_game: [{}];

		/**
		 * Player submits their board setup
		 */
		submit_setup: [{
			ready?: boolean;
			cells: CellValue[];
		}];

		/**
		 * Player submits their move attacking an opponent's cell `w = x + (y * 10)` where w is in [0,99]
		 */
		attack_cell: [{
			cell: Uint8;
		}, {
			result: CellValue;
		}];

		/**
		 * Allows a player to claim victory once their opponent has exceeded their turn timer
		 */
		claim_victory: [{}];
	}>;

	queries: {
		/**
		 * Fetches a list of active games in the loby
		 */
		list_games: [{
			page_size?: Uint32;
			page?: Uint32;
		}, {
			games: NewGame[];
		}];
	}
	& MsgsRequiresGameId<{
		/**
		 * Fetches the current game state
		 */
		game_state: [{}, {
			role: PlayerRole;
			state: GameState;
			tracking: CellValue[];
			board: CellValue[];
		}];
	}>;
}>;
