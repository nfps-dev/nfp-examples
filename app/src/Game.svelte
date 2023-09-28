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
</script>

<style lang="less">
	.board {
		transform: perspective(30cm) translateY(-50px);
		transform-style: preserve-3d;
		width: 450px;
		margin: 0 50px;
		padding: 0 50px;
	}
</style>

<section>
	<div>
		{g_listing.title}
	</div>

	<div>
		<h3>
			Position your fleet
		</h3>

		<div class="board">
			<Grid />
			<Grid b_home />
		</div>
	</div>
</section>
