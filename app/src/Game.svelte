<script lang="ts">
	import type {ActiveGame, ListedGame} from './interface/app';
	
	import Grid from './Grid.svelte';


	// const submit_setup = async() => {
	// 	const si_game = '';
	// 	const a_board = Array(100);

	// 	// await exec_contract(K_CONTRACT, K_WALLET, {
	// 	// 	submit_setup: {
	// 	// 		game_id: si_game,
	// 	// 		cells: a_board,
	// 	// 	},
	// 	// }, );
	// };


	export let g_listing: ListedGame;

	export let g_state: ActiveGame;

	let b_game_on = false;
</script>

<style lang="less">
	.board {
		transform: perspective(30cm) translateY(-500px);
		transform-style: preserve-3d;
		width: 450px;
		margin: 0 50px;
		padding: 0 50px;

		transition: transform 2s ease-in-out;
	}

	.game-on {
		transform: perspective(30cm) translateY(-50px);
	}
</style>

<section>
	<div>
		{g_listing.title}
	</div>

	<div>
		<div>
			<h3>
				Position your fleet
			</h3>

			<button on:click={() => b_game_on = true}>
				Begin
			</button>
		</div>

		<div class="board" class:game-on={b_game_on}>
			<Grid b_game_on={b_game_on} />
			<Grid b_game_on={b_game_on} b_home />
		</div>
	</div>
</section>
