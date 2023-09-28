<script lang="ts">
	import {create_html} from '@nfps.dev/runtime';
	import {onMount} from 'svelte';

	const A_COLS = 'ABCDEFGHIJ'.split('');

	const A_COLORS = [
		'f2cc9c',
		'f0b989',
		'e5a575',
	];

	let dm_app: HTMLDivElement;

	onMount(() => {
		const a_cols = [];
		const a_rows = [];

		for(let i_col=0; i_col<10; i_col++) {
			a_cols.push(create_html('div', {}, [
				A_COLS[i_col],
			]));

			a_rows.push(create_html('div', {}, [
				(i_col + 1)+'',
			]));
		}

		const dm_cols = create_html('div', {
			class: 'cols',
		}, a_cols);

		const dm_rows = create_html('div', {
			class: 'rows',
		}, a_rows);

		const dm_grid = create_html('table', {}, A_COLS.map(_ => create_html('tr', {}, A_COLS.map((__) => {
			const x_rnd = Math.random();

			const dm_td = create_html('td', {
				style: `background:#${x_rnd < 0.5? A_COLORS[0]: x_rnd < 0.9? A_COLORS[1]: A_COLORS[2]}`,
			}, []);

			return dm_td;
		}))));

		const dm_middle = create_html('div', {
			class: 'middle',
		}, [
			dm_rows, dm_grid, dm_rows.cloneNode(true),
		]);

		dm_app.append(dm_cols, dm_middle, dm_cols.cloneNode(true));
	});

	export let b_home = false;

</script>

<style lang="less">
	@xl_sink: 80px;
	@xl_shift: 100px;
	@xl_dim: 460px;
	@sx_pers: perspective(30cm);

	@keyframes away-entry {
		100% {
			transform: rotateX(-45deg) translate3d(0, @xl_sink, 0);
		}
	}

	@keyframes home-entry {
		100% {
			transform: rotateX(45deg) translate3d(0, -@xl_sink, 0);
		}
	}

	.grid {
		border: 1px solid #ccc;
		display: flex;
		flex-direction: column;
		width: @xl_dim;
		height: @xl_dim;
		animation: 3s ease-out 1 both away-entry;

		transform: rotateX(0deg) translate3d(0, 0px, -@xl_shift);

		background:
			linear-gradient(90deg, #78362855 25%, #0000 25%, #0000 50%, #a4533d55 50%, #a4533d55 75%, #0000 75%),
			linear-gradient(#8a413366 25%, #0000 25%, #0000 50%),
			linear-gradient(#ce8764 25%, #c67a58 25%, #c67a58 50%, #cc8460 50%, #cc8460 75%, #c78058 75%);
		background-size: 160px 160px;
		background-position: -10px -10px;
	}

	.home {
		animation-name: home-entry;
	}

</style>

<div class="grid" class:home={b_home} bind:this={dm_app} />
