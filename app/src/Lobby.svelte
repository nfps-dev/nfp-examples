<script lang="ts">	
	import type {ListedGame} from './interface/app';

	import type {UiController} from '@nfps.dev/components/NeutrinoWallet';
	import type {Uint128, Coin, Timestamp} from '@solar-republic/contractor';
	
	import {subscribe_snip52_channels} from '@solar-republic/neutrino';
	import {A_TOKEN_LOCATION, K_CONTRACT, SH_VIEWING_KEY} from 'nfpx:bootloader';
	import {onMount} from 'svelte';
	
	import {XG_LIMIT_BASE} from './stores';
	
	import Listing from './Listing.svelte';

	const {
		K_WALLET,
		K_SERVICE,
		SA_OWNER,
		Z_AUTH,
	} = destructureImportedNfpModule('app');

	const refresh = async() => {
		// query for listed games
		const [g_res,, s_err] = await K_SERVICE.query('list_games', {
			token_id: A_TOKEN_LOCATION[2],
		}, Z_AUTH);

		// success; update games list
		if(g_res) {
			a_games = g_res.games;
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

		// subscribe to notification channel for when new games are listed
		await subscribe_snip52_channels(K_WALLET.rpc, K_CONTRACT, [SH_VIEWING_KEY, SA_OWNER], {
			game_listed(a_details) {
				const [si_game, s_title, xg_wager] = a_details;

				// append to games list
				a_games = [...a_games, {
					game_id: si_game,
					title: s_title,
					wager: {amount:xg_wager} as Coin,
					created: Date.now()+'000' as Timestamp,
				}];
			},
		});
	});


	const A_ADJECTIVES = [
		'Nautical',
		'Anchored',
		'Formidable',
		'Armored',
		'Maritime',
		'Ironclad',
		'Steely',
		'Cruising',
		'Submersible',
		'Aquatic',
		'Hypersonic',
	];

	const A_NOUNS = [
		'Dreadnought',
		'Torpedo',
		'Cruiser',
		'Destroyer',
		'Patrol',
		'Submarine',
		'Battleship',
		'Carrier',
		'Cargo',
		'Mast',
		'Frigate',
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
			title: s_title,
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

	const dev_new_game = import.meta.env.DEV
		? () => {
			a_games_own = [
				...a_games_own,
				{
					game_id: 'xzy27f14ccbedf89102',
					title: 'Dev: My New Game',
					created: (Date.now()*1e3)+'' as Timestamp,
					wager: {
						amount: '0' as Uint128,
						denom: 'uscrt',
					} as Coin,
				},
			];
		}: void 0;


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
	let a_games: ListedGame[] = Array(15).fill(
		{
			created: ((Date.now() - 5000)*1e3)+'' as Timestamp,
			title: 'Test',
			game_id: 'c002f0cd-4053-4eba-9bbb-7295c41db594',
			wager: {
				amount: '2000000' as Uint128,
				denom: 'uscrt',
			} as Coin,
		}
	);

	// own games
	let a_games_own: ListedGame[] = [];

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

		{#if import.meta.env.DEV}
			<button on:click={dev_new_game}>
				Dev: Add Game
			</button>

			<button on:click={() => {

			}}>

			</button>
		{/if}
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
			{#each a_games_own as g_game_own}
				<Listing g_game={g_game_own} xc_own={1} />
			{/each}

			{#each a_games as g_game (g_game.game_id)}
				<Listing {g_game} on:join />
			{/each}
		</tbody>
	</table>

	{#if b_busy}
		<div class="curtain" />
	{/if}
</section>
