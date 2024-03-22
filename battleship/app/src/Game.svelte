<script lang="ts">
	import type {Nilable} from '@blake.regalia/belt';
	import type {UiController} from '@nfps.dev/components/HotWallet';
	
	import type {Uint8} from '@solar-republic/contractor';
	
	import {__UNDEFINED, F_IDENTITY, timeout} from '@blake.regalia/belt';
	
	import {A_TOKEN_LOCATION} from 'nfpx:bootloader';
	import {slide} from 'svelte/transition';
	
	import {type PlayerRole, type ActiveGame, CellValue, type ListedGame} from './interface/app';
	
	import {TurnState} from './interface/app';
	
	
	import {XG_LIMIT_BASE} from './stores';
	
	import Grid from './Grid.svelte';
	const {K_SERVICE} = destructureImportedNfpModule('app');

	const attack_cell = async(i_cell: number) => {
		b_lock_away = true;
		y_neutrino.status(1);

		const b_finishing_move = g_game.away.reduce((c, x) => c + (x & CellValue.HIT? 1: 0), 0) >= (2 + 3 + 3 + 4 + 5 - 1);

		// submit move
		const [g_res, xc_code, s_res] = await K_SERVICE.exec('attack_cell', {
			token_id: A_TOKEN_LOCATION[2],
			game_id: g_game.game_id,
			cell: i_cell as Uint8,
		}, XG_LIMIT_BASE + (b_finishing_move? 30_000n: 0n));

		b_lock_away = false;
		y_neutrino.status(0);

		// success
		if(g_res) {
			// update away grid and turn
			g_game.away = a_away = g_res.away;
			xc_turn = g_res.turn;
		}
		// failure
		else {
			y_neutrino.tx_err(s_res);
		}
	};

	const handle_attack = ({detail:i_cell}: CustomEvent<number>) => attack_cell(i_cell);

	const submit_setup = async({detail:a_cells}: {detail: CellValue[]}) => {
		// reset retry handler
		f_retry = __UNDEFINED;

		// clear error
		y_neutrino.collapse();

		// set wallet ui status to busy
		y_neutrino.status(1);

		// execute contract
		const [, xc_code, s_res] = await K_SERVICE.exec('submit_setup', {
			token_id: A_TOKEN_LOCATION[2],
			game_id: g_game.game_id,
			cells: a_cells,
		}, XG_LIMIT_BASE);

		// reset busy status
		y_neutrino.status(0);

		// failure
		if(xc_code) {
			// show error in wallet ui
			y_neutrino.tx_err(s_res);

			// pause, then show retry button
			await timeout(750);
			f_retry = () => void submit_setup({detail:a_cells});
		}
		// success
		else {
			b_game_on = true;
		}
	};

	export let g_game: ActiveGame;

	export let y_neutrino: UiController;

	export let c_updates: number;

	let a_home: CellValue[] = Array(100).fill(0);
	let a_away: CellValue[] = a_home.slice();

	let b_game_on = false;

	let b_lock_away = false;

	let xc_turn = TurnState.WAITING_FOR_PLAYER;

	let xc_role: PlayerRole;

	let f_retry: Nilable<() => void>;

	// when game state updates
	$: if(c_updates && g_game.turn) {
		// home is not all empty
		b_game_on = g_game.home.some(F_IDENTITY);

		// update cells
		a_away = g_game.away;
		a_home = g_game.home;

		// update turn state
		xc_turn = g_game.turn;
		xc_role = g_game.role;
	}
</script>

<style lang="less">
	.above {
		width: 500px;
		margin-left: 80px;
		position: relative;
		z-index: 2;
	}

	.controls {
		height: 3em;
		margin: 1em;
		text-align: center;
	}

	button {
		padding: 0 2em;
		height: 100%;
	}

	.board {
		transform: perspective(30cm) translateY(-460px);
		transform-style: preserve-3d;
		margin: 0 50px;
		padding: 0 50px;

		transition: transform 2s ease-in-out;

		position: relative;
		z-index: 1;
		width: 500px;
	}

	.game-on {
		transform: perspective(30cm) translateY(-50px);
	}
</style>

<section>
	<div class="above">
		<h2 style="text-align:center">
			{g_game.title}
		</h2>

		<div class="controls">
			{#if f_retry}
				<button class="retry" transition:slide on:click={f_retry}>
					Retry
				</button>
			{/if}
		</div>
	</div>

<!-- 
	{#if [TurnState.GAME_OVER_INITIATOR_WON, TurnState.GAME_OVER_JOINER_WON].includes(xc_turn)}
		<h1>
			You {xc_role === xc_turn % 2? 'won!': 'lost.'}
		</h1>
	{:else} -->
		<div class="board" class:game-on={b_game_on}>
			<Grid
				{b_game_on}
				{xc_turn}
				{xc_role}
				a_cells={a_away}
				b_locked={b_lock_away}
				on:attack={handle_attack}
			/>
			<Grid b_home
				{b_game_on}
				{xc_turn}
				{xc_role}
				a_cells={a_home}
				on:submit={submit_setup}
			/>
		</div>
	<!-- {/if} -->
</section>
