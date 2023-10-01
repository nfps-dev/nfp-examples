<script lang="ts">
	import type {ActiveGame, CellValue, ListedGame} from './interface/app';
	import type {Nilable} from '@blake.regalia/belt';
	import type {UiController} from '@nfps.dev/components/NeutrinoWallet';
	
	import {__UNDEFINED, timeout} from '@blake.regalia/belt';
	
	import {A_TOKEN_LOCATION} from 'nfpx:bootloader';
	import {slide} from 'svelte/transition';
	
	import {XG_LIMIT_BASE} from './stores';
	
	import Grid from './Grid.svelte';
	const {K_SERVICE} = destructureImportedNfpModule('app');


	export let g_listing: ListedGame;

	export let g_state: ActiveGame;

	export let y_neutrino: UiController;

	let b_game_on = false;

	let f_retry: Nilable<() => void>;

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
			game_id: g_listing.game_id,
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
	}

	.game-on {
		transform: perspective(30cm) translateY(-50px);
	}
</style>

<section>
	<div class="above">
		<h2 style="text-align:center">
			{g_listing.title}
		</h2>

		<div class="controls">
			{#if f_retry}
				<button class="retry" transition:slide on:click={f_retry}>
					Retry
				</button>
			{/if}
		</div>
	</div>

	<div class="board" class:game-on={b_game_on}>
		<Grid b_game_on={b_game_on} />
		<Grid b_game_on={b_game_on} b_home
			on:submit={submit_setup}
		/>
	</div>
</section>
