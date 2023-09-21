<script lang="ts">
	import Wallet from '@nfps.dev/components/src/Wallet.svelte';
	import {qs, qsa} from '@nfps.dev/runtime';
	
	// use import statements for any modules that are already loaded by the time 'app' starts loading
	import {
		K_CONTRACT,
		SH_VIEWING_KEY,
		ls_read,
		ls_write,
	} from 'nfpx:bootloader';

	const {
		K_WALLET,
		SA_OWNER,
		A_COMCS,
		dm_foreign,
	} = destructureImportedNfpModule('app');

	// before 'App.svelte' is instantiated, 'main.ts' dynamically loads the 'storage' module.
	// use an reserved function to import from the loaded module rather than a static import.
	// this way, the destructuring expression will not get moved outside the svelte component.
	const {
		readOwner,
		writeOwner,
	} = destructureImportedNfpModule('storage');

</script>

<style lang="less">
	@import './def.less';

	:global(#app) {
		--ease-out-quick: @ease-out-quick;
	}

	:global(*) {
		color: #f7f7f7;
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

<section bind:this={dm_section}>
	<div class="flex spaced">
		<h3 class:on={!b_loading && b_writable}>
			<i /> {b_loading? 'Loading...': b_writable? 'Synced': 'Awaiting Wallet'}
		</h3>
	</div>

	<div>
		<fieldset disabled={b_loading || !b_writable}>
			<input id="name" type="text" autocomplete="off"
				disabled={b_locked} bind:value={s_name} placeholder={s_placeholder}>
			<button on:click={edit_name}>{s_action}</button>
		</fieldset>
	</div>

	<!-- <Notifications /> -->
</section>

<Wallet
	args={[K_WALLET, SA_OWNER, [SH_VIEWING_KEY, SA_OWNER], A_COMCS, K_CONTRACT, dm_foreign]}
	/>
