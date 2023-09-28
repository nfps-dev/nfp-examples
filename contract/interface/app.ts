
import type {
	Coin, Timestamp, Uint128, Uint32, Uint8,
	SecretContractInterface, Snip52, Snip821,
	MethodDescriptorGroup, MethodGroup, WithSnipAuthViewer, MakeQueryPermitVariants,
} from '@solar-republic/contractor';
import { U } from 'ts-toolbelt';

// import {AccessLevel} from '@solar-republic/contractor/snip-721';

// export type * from '@solar-republic/contractor/snip-721';

// export * from '@solar-republic/contractor/snip-721';

// export {AccessLevel};

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
	// nothing occupies the cell. used for both `home` and `away` grids
	// for the `away` grid: this indicates player has not yet attacked cell
	EMPTY,

	// for `away` grid only: player missed the cell
	MISS,

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

	// for `away` grid only: player hit a vessel but has not yet sunk it
	HIT_UNKNOWN=0x80,
}


/**
 * Used to represent a game to prospective players browsing the lobby
 */
export type ListedGame = {
	game_id: string;
	wager: Coin;
	title: string;
	created: Timestamp;
};

/**
 * Used to represent the complete state of an active game
 */
export type ActiveGame = {
	role: PlayerRole;
	state: GameState;
	home: CellValue[];
	away: CellValue[];
};

// utility type that facilitates adding the same `game_id` key to each msg
type MsgsRequiresGameId<h_group extends MethodDescriptorGroup> = MethodGroup.Augment<h_group, {
	msg: {
		game_id: string;
	};
}>;

type AuthenticatedQueries = MethodGroup.Canonicalize<{
	/**
	 * Fetches a list of active games in the lobby
	 */
	list_games: [{
		page_size?: Uint32;
		page?: Uint32;
	}, {
		games: ListedGame[];
	}];

	/**
	 * Gets the list of active games this player is party to
	 */
	active_games: [{}, {
		game_ids: string[];
	}];
}
& MsgsRequiresGameId<{
	/**
	 * Fetches the current game state
	 */
	game_state: [{}, ListedGame & ActiveGame];
}>>;

export type AppInterface<w_defer=never> = SecretContractInterface<{
	extends: [
		Snip52,
		// Snip821,
	];

	config: {
		snip52_channels: {
			/**
			 * A new game has been added to the lobby
			 */
			game_listed: [
				game_id: string,
				title: string,
				wager_uscrt: Uint128,
			];

			/**
			 * A player joined the user's new game
			 */
			player_joined: [
				game_id: string,
			];

			/**
			 * The opponent attacked a cell
			 */
			opponent_attacked: [
				cell: Uint8,
			];
		};
	};

	executions: {
		/**
		 * Creates a new game in the lobby
		 */
		new_game: {
			msg: {
				title?: string;
			};
			response: {
				game: ListedGame;
			};
			funds: {
				amount: Uint128<'0' | `${WagerAmountsScrt}000000`>;
				denom: 'uscrt';
			};
		};
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
		with_permit: {
			variants: U.ListOf<MakeQueryPermitVariants<AuthenticatedQueries>>;
		};
	} & WithSnipAuthViewer<AuthenticatedQueries>;
}>;

// type insp = AppInterface['queries']['with_permit']['variants'][1]
