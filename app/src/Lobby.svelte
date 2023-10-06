<script lang="ts">	
	import type {ListedGame} from './interface/app';

	import type {UiController} from '@nfps.dev/components/NeutrinoWallet';
	import type {Uint128} from '@solar-republic/contractor';
	
	import {A_TOKEN_LOCATION} from 'nfpx:bootloader';
	import {onMount} from 'svelte';
	
	import {XG_LIMIT_BASE} from './stores';
	
	import Listing from './Listing.svelte';

	const {
		K_SERVICE,
		Z_AUTH,
	} = destructureImportedNfpModule('app');

	const refresh = async() => {
		// query for active games
		let a_active: string[] = [];
		{
			const [g_res,, s_err] = await K_SERVICE.query('active_games', {
				token_id: A_TOKEN_LOCATION[2],
			}, Z_AUTH);

			if(g_res) {
				a_active = g_res.game_ids;
			}
		}

		// query for listed games
		const [g_res,, s_err] = await K_SERVICE.query('list_games', {
			token_id: A_TOKEN_LOCATION[2],
		}, Z_AUTH);

		// success; update games list
		if(g_res) {
			a_games = g_res.games.filter(g => !a_active.includes(g.game_id));
			a_games_own = g_res.games.filter(g => a_active.includes(g.game_id));
		}
		// error
		else {
			s_error = s_err;
		}
	};

	// once the dom mounts
	onMount(async() => {
		// load the set of listed games
		await refresh();

		// // subscribe to notification channel for when new games are listed
		// await subscribe_snip52_channels(K_WALLET.rpc, K_CONTRACT, [SH_VIEWING_KEY, SA_OWNER], {
		// 	// game_listed(a_details) {
		// 	// 	const [si_game, s_title, xg_wager] = a_details;

		// 	// 	// append to games list
		// 	// 	a_games = [...a_games, {
		// 	// 		game_id: si_game,
		// 	// 		title: s_title,
		// 	// 		wager: {amount:xg_wager} as Coin,
		// 	// 		created: Date.now()+'000000' as Timestamp,
		// 	// 	}];
		// 	// },
		// });

		// refresh every so often
		setInterval(() => {
			void refresh();
		}, 15e3);
	});


	const A_ADJECTIVES = [
		'Scorched',
		'Anchored',
		'Formidable',
		'Armored',
		'Nomadic',
		'Ironclad',
		'Steely',
		'Arid',
		'Rugged',
		'Hardened',
		'Hypersonic',
		'Blazing',
		'Forged',
	];

	const A_NOUNS = [
		'Dreadnought',
		'Oasis',
		'Cruiser',
		'Enforcer',
		'Patrol',
		'Sandstorm',
		'Dune',
		'Titan',
		'Cargo',
		'Fortress',
		'Frigate',
		'Mirage',
	];

	const select = (a_words: string[]) => a_words[Math.floor(Math.random() * a_words.length)];


	const create_game = async() => {
		// close modal
		dm_dialog.close();

		// indicate busy status
		y_neutrino.status(b_loading=true);

		// create wager amount string
		const sg_wager_uscrt = (+s_wager * 1e6)+'' as Uint128;

		// submit launch request
		const [g_ans,, s_res] = await K_SERVICE.exec('new_game', {
			token_id: A_TOKEN_LOCATION[2],
			title: s_title || s_placeholder,
		}, XG_LIMIT_BASE, '0' === sg_wager_uscrt as string? []: [
			[sg_wager_uscrt, 'uscrt'],
		]);

		// no longer busy
		y_neutrino.status(b_loading=false);

		// success; add to own games
		if(g_ans) {
			a_games_own = [...a_games_own, g_ans.game];
		}
		// did not work
		else {
			y_neutrino.tx_err(s_res);
		}
	};


	/* eslint-disable prefer-const */

	// busy state
	export let b_busy = false;

	// query error
	let s_error = '';

	// neutrino controller
	export let y_neutrino: UiController;

	// dom bindings
	let dm_dialog: HTMLDialogElement;
	let dm_error: HTMLDialogElement;

	// list of joinable games
	let a_games: ListedGame[] = [];

	// own games
	export let a_games_own: ListedGame[] = [];

	let b_loading = false;

	let s_title = '';
	let s_wager = '0';

	let s_placeholder = select(A_ADJECTIVES)+' '+select(A_NOUNS);

	$: if(s_error) {
		dm_error.showModal();
	}

	/* eslint-enable */
</script>

<style lang="less">
	section {

	}

	dialog {

	}

	fieldset {
		>div {
			display: flex;
			align-items: center;
			margin: 6px;
			min-width: 50ch;

			>:first-child {
				min-width: 12ch;
			}

			>:nth-child(2) {
				width: 100%;
			}
		}
	}

	.actions {
		margin-top: 2em;
		display: flex;
		gap: 1em;
		justify-content: center;

		>* {
			min-width: 9em;
		}
	}

	table {
		width: 100%;
		min-width: 600px;
		max-width: 840px;

		border-collapse: collapse;
	}

	thead {
		height: 3em;
	}
</style>

<section>
	<div>
		<button class="cta" on:click={() => dm_dialog.showModal()}>
			New Game
		</button>

		<!-- {#if import.meta.env.DEV}
			<button on:click={dev_new_game}>
				Dev: Add Game
			</button>

			<button on:click={() => {

			}}>

			</button>
		{/if} -->
	</div>

	<div>
		<dialog bind:this={dm_dialog}>
			<form method="dialog" on:submit={create_game}>
				<h3>
					Create a new game
				</h3>

				<fieldset>
					<div>
						<span>
							Title:
						</span>
						<input maxlength="40" type="text" autocomplete="off" bind:value={s_title} placeholder={s_placeholder}>
					</div>
		
					<div>
						<span>
							Wager:
						</span>
						<select bind:value={s_wager}>
							<option value="0">No wager</option>
							{#each [1, 2, 5, 10] as n_wager}
								<option value="{n_wager}">{n_wager} SCRT</option>
							{/each}
						</select>
					</div>
				</fieldset>

				<div class="actions">
					<button type="button" on:click={() => dm_dialog.close()}>
						Cancel
					</button>
		
					<button class="cta">
						Create
					</button>
				</div>
			</form>
		</dialog>

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
	</div>

	<table>
		<thead>
			<tr>
				<th></th>
				<th>ID</th>
				<th>Title</th>
				<th>Wager</th>
				<th>Created</th>
			</tr>
		</thead>
		<tbody>
			{#if (a_games_own.length + a_games.length) > 0}
				{#each a_games_own as g_game_own}
					<Listing g_game={g_game_own} xc_own={1} />
				{/each}

				{#each a_games as g_game (g_game.game_id)}
					<Listing {g_game} on:join />
				{/each}
			{:else}
				<tr>
					<td style="text-align:center">
						<!-- svelte-ignore a11y-no-static-element-interactions -->
						Currently no open games. <span class="link" on:click={() => dm_dialog.showModal()}>Start a new game</span>
					</td>
				</tr>
			{/if}
		</tbody>
	</table>

	{#if b_busy}
		<div class="curtain" />
	{/if}
</section>
