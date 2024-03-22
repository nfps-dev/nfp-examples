<script lang="ts">
	import type {ActiveGame, ListedGame} from './interface/app';
	import type {SecretAccAddr, Uint128} from '@solar-republic/contractor';
	
	import {oda} from '@blake.regalia/belt';
	import HotWallet, {type UiController} from '@nfps.dev/components/HotWallet';
	import {subscribe_snip52_channels, type WeakSecretAccAddr} from '@solar-republic/neutrino';
	
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
		}, XG_LIMIT_BASE, '0' as Uint128 !== g_game.wager.amount? [
			[g_game.wager.amount, 'uscrt'],
		]: []);

		// toggle off busy flag
		y_neutrino.status(b_loading=false);

		// did not work
		if(xc_code) {
			y_neutrino.tx_err(s_res);
		}
		// success
		else {
			g_game_active = {
				...g_game,
				role: PlayerRole.JOINER,
				turn: TurnState.WAITING_FOR_BOTH_PLAYERS_SETUP,
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
		else if(g_res) {
			// game has started
			if(g_res.turn) {
				g_game_active = g_res as unknown as ActiveGame;
			}
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

		// subscribe to game state updates
		await subscribe_snip52_channels(K_WALLET.rpc, K_CONTRACT, Z_AUTH, {
			game_updated([si_game, a_home, xc_turn]) {
				// not for the active game
				if(si_game !== g_game_active?.game_id) {
					// for an own listed game
					for(const g_listed of a_games_own) {
						if(si_game === g_listed.game_id) {
							// convert listing into active game
							g_game_active = {
								...g_listed,
								role: PlayerRole.INITIATOR,
								away: Array(100).fill(0),
								home: a_home,
								turn: xc_turn,
							};

							c_updates++;
						}
					}

					// done
					return;
				}

				// player is playing themself, wrong token; ignore
				if(xc_turn < TurnState.GAME_OVER_INITIATOR_WON && g_game_active.role as number !== xc_turn % 2) return;

				// reactive update assignment
				oda(g_game_active, {
					home: a_home,
					turn: xc_turn,
				});

				c_updates++;
			},
		});

		// done loading
		b_loading = false;
	};

	function spendable_gas_refreshed({detail:[, a_granters]}: CustomEvent<[bigint, [SecretAccAddr, string][]]>) {
		const [sa_granter] = a_granters.reduce((a_best, a_each) => +a_each[1] > +a_best[1]? a_each: a_best, ['', '0']);
		K_SERVICE.granter(sa_granter as WeakSecretAccAddr);
	}

	// dom bindings
	let dm_error: HTMLDialogElement;

	// whether a game is currently being joined
	let b_loading = false;

	// error message
	let s_error = '';

	// active game
	let g_game_active: ActiveGame;

	let y_neutrino: UiController;

	let a_games_own: ListedGame[];

	let c_updates = 1;

	$: if(s_error) {
		dm_error.showModal();
	}

	// start load
	void load();
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

	{#if g_game_active}
		<Game g_game={g_game_active} {c_updates} {y_neutrino} />
	{:else}
		<Lobby
			bind:a_games_own={a_games_own}
			b_busy={b_loading}
			{y_neutrino}
			on:join={join_game}
		/>
	{/if}
</main>

<NeutrinoWallet
	on:spendable_gas_refreshed={spendable_gas_refreshed}
	bind:controller={y_neutrino}
	args={[K_WALLET, SA_OWNER, Z_AUTH, A_COMCS, K_CONTRACT, dm_foreign]}
	/>