<script lang="ts">
	import type {ListedGame} from './interface/app';
	import type {Uint128} from '@solar-republic/contractor/datatypes';
	
	import {createEventDispatcher} from 'svelte';
	
	import {yxt_now} from './stores';

	const dispatch = createEventDispatcher();

	export let g_game: ListedGame;

	// whether this is another player's listing or own
	export let xc_own: 0 | 1 = 0;

	let s_created = '...';
	$: {
		const n_secs = ($yxt_now - Number(g_game.created.slice(0, -3))) / 1e3;

		if(n_secs < 10) {
			s_created = 'a few seconds ago';
		}
		else if(n_secs < 90) {
			s_created = `${Math.floor(n_secs / 10) * 10} seconds ago`;
		}
		else if(n_secs < 5400) {
			s_created = `${Math.floor(n_secs / 60)} min ago`;
		}
		else {
			s_created = `${Math.floor(n_secs / 3600)} hours ago`;
		}
	}

	const print_uscrt = (s_amount: Uint128) => ('0' as Uint128) === s_amount
		? 'FREE'
		: s_amount.slice(0, -6)+'.'+s_amount.slice(-6, -5)+' SCRT';

	const join_game = () => dispatch('join', g_game);
</script>

<style lang="less">
	tr {
		background: transparent;
		animation: row 10s ease infinite;

		&:hover,&.own {
			background: linear-gradient(-45deg, #13c5c566, #2636D966, #d03f4866, #880eda66);	
			background-size: 400% 400%;

			button {
				background: orange;
				color: #000;
			}
		}
	}

	.delay(@i) when (@i > 0) {
		@sel: ~"5n + @{i}";
		tr:nth-child(@{sel}) {
			animation-delay: (@i - 1) * -2s;
		}
		.delay(@i - 1);
	}

	.delay(5);

	@keyframes row {
		0% {
			background-position: 0% 50%;
		}
		50% {
			background-position: 100% 50%;
		}
		100% {
			background-position: 0% 50%;
		}
	}

	td {
		border: 1px solid #666;
		border-color: #666 #333;
		padding: 0 8px;
		min-width: 120px;
	}
	
	.full {
		padding: 0;

		>button {
			width: 100%;
			color: #fff;
		}
	}

	.id {
		width: 10ch;
		max-width: 10ch;
		min-width: 0;
	}

	i {
		color: #888;
		vertical-align: inherit;
		width: 8ch;
		display: inline-block;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.title {
		width: 90%;
		max-width: 350px;
	}

	.time {
		width: 120px;
	}
</style>

<tr class:own={xc_own}>
	<td class="full">
		{#if xc_own}
			<button>
				Waiting
			</button>
		{:else}
			<button on:click={join_game}>
				Join
			</button>
		{/if}
	</td>
	<td class="id"><i>{g_game.game_id}</i></td>
	<td class="title">{g_game.title}</td>
	<td>{print_uscrt(g_game.wager.amount)}</td>
	<td class="time">{s_created}</td>
</tr>
