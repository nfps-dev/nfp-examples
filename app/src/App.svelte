<script lang="ts">
	import type {ActiveGame, ListedGame} from './interface/app';
	import type {Coin, Timestamp, Uint128} from '@solar-republic/contractor';
	
	import NeutrinoWallet, {type UiController} from '@nfps.dev/components/NeutrinoWallet';
	
	import {A_TOKEN_LOCATION, K_CONTRACT} from 'nfpx:bootloader';
	
	import {PlayerRole, TurnState, CellValue} from './interface/app';
	
	import {XG_LIMIT_BASE} from './stores';
	
	import Game from './Game.svelte';
	import Lobby from './Lobby.svelte';

	const {
		K_WALLET,
		K_SERVICE,
		Z_AUTH,
		SA_OWNER,
		A_COMCS,
		dm_foreign,
		dm_root,
	} = destructureImportedNfpModule('app');

	// prevent screen shifting
	dm_root.addEventListener('mousedown', (d_event: MouseEvent) => {
		if(d_event.shiftKey) {
			d_event.preventDefault();
		}
	});

	// join a listed game
	const join_game = async({detail:g_game}: CustomEvent<ListedGame>) => {
		// indicate busy status
		y_neutrino.status(b_loading=true);

		// submit join request
		const [, xc_code, s_res] = await K_SERVICE.exec('join_game', {
			token_id: A_TOKEN_LOCATION[2],
			game_id: g_game.game_id,
		}, XG_LIMIT_BASE, [
			[g_game.wager.amount, 'uscrt'],
		]);

		// toggle off busy flag
		y_neutrino.status(b_loading=false);

		// did not work
		if(xc_code) {
			y_neutrino.tx_err(s_res);
		}
		// success
		else {
			g_active_game_listing = g_game;
			g_active_game_state = {
				role: PlayerRole.JOINER,
				state: TurnState.WAITING_FOR_BOTH_PLAYERS_SETUP,
				home: Array(100).fill(CellValue.EMPTY),
				away: Array(100).fill(CellValue.EMPTY),
			};
		}
	};

	// 
	const reconnect_game = async(si_game: string) => {
		// fetch game state
		const [g_res, xc_code, s_err] = await K_SERVICE.query('game_state', {
			token_id: A_TOKEN_LOCATION[2],
			game_id: si_game,
		}, Z_AUTH);

		// failed to fetch
		if(xc_code) {
			s_error = `Failed to load running game: ${s_err}`;

			throw new Error(`Failed to load game state: ${s_err}`);
		}
		// success
		else {
			// set active game
			g_active_game_listing = g_res!;
			g_active_game_state = g_res!;
		}
	};

	// init
	const load = async() => {
		// wait
		b_loading = true;

		// query for active games
		const [g_res, xc_code, s_err] = await K_SERVICE.query('active_games', {
			token_id: A_TOKEN_LOCATION[2],
		}, Z_AUTH);

		// query failed
		if(xc_code) {
			s_error = `Failed to check for running games. Try reloading.\n\n${s_err}`;
		}
		// success
		else {
			// games running
			const a_active = g_res!.game_ids;
			if(a_active.length) {
				// query for game
				for(const si_game of a_active) {
					await reconnect_game(si_game);
				}
			}
		}

		// done loading
		b_loading = false;
	};

	const show_board = import.meta.env? () => {
		g_active_game_state = {
			role: PlayerRole.JOINER,
			state: TurnState.WAITING_FOR_BOTH_PLAYERS_SETUP,
			home: Array(100).fill(CellValue.EMPTY),
			away: Array(100).fill(CellValue.EMPTY),
		};

		g_active_game_listing = {
			created: (Date.now()*1e3)+'' as Timestamp,
			title: 'Test',
			game_id: 'dev',
			wager: {
				amount: '0' as Uint128,
				denom: 'uscrt',
			} as Coin,
		};
	}: void 0;


	// dom bindings
	let dm_error: HTMLDialogElement;

	// whether a game is currently being joined
	let b_loading = false;

	// error message
	let s_error = '';

	// active game
	let g_active_game_listing: ListedGame;
	let g_active_game_state: ActiveGame;

	let y_neutrino: UiController;

	$: if(s_error) {
		dm_error.showModal();
	}

	// start load
	if(!import.meta.env.DEV) {
		void load();
	}
	else {
		show_board!();
	}
</script>

<style lang="less">
	@import './def.less';

	:global(#app) {
		--ease-out-quick: @ease-out-quick;
	}

	main {
		user-select: none;
	}

	section {
		margin: 1em
	}

	h3 {
		&.on i {
			background: chartreuse;
		}
	}

	i {
		border-radius: 8px;
		width: 8px;
		height: 8px;
		display: inline-block;
		margin-bottom: 1px;

		background: darkorange;
	}

	fieldset {
		:global(&) {
			border: 0;
		}

		input, button {
			:global(&) {
				background: #333;
				color: #ce3;
				padding: 8px 12px;
				border: 0;
		
				&:focus {
					outline: 1px solid orange;
					border-radius: 2px;
				}
		
				&:disabled {
					opacity: 0.8;
				}
			}
		}
	}


	:global(.flex) {
		display: flex;
	}

	:global(.spaced) {
		justify-content: space-between;
	}
</style>

<main>
	<dialog class="error" bind:this={dm_error} on:close={() => s_error = ''}>
		<form method="dialog">
			<h3>Error</h3>

			<p>
				{s_error}
			</p>

			<button class="cta" on:click={() => dm_error.close()}>
				Dismiss
			</button>
		</form>
	</dialog>

	{#if import.meta.env.DEV}
		<button on:click={show_board}>
			Dev: Show Board
		</button>
	{/if}

	{#if g_active_game_listing}
		<Game g_listing={g_active_game_listing} g_state={g_active_game_state} {y_neutrino} />
	{:else}
		<Lobby b_busy={b_loading} {y_neutrino} on:join={join_game} />
	{/if}
</main>

<NeutrinoWallet args={[K_WALLET, SA_OWNER, Z_AUTH, A_COMCS, K_CONTRACT, dm_foreign]} bind:controller={y_neutrino} />