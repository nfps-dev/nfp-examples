<script lang="ts">
	import {oda, timeout} from '@blake.regalia/belt';
	import {create_html} from '@nfps.dev/runtime';
	import {onMount} from 'svelte';

	import SX_SPRITES from '../../media/explosion-sprites-lo.png';

	const NL_SPRITES = 4;

	const XL_SPRITE_SRC_DIM = 240 / 4;
	const XL_SPRITE_DEST_DIM = 50;

	const random = (x_range: number, x_min: number=0) => (Math.random() * x_range) + x_min;


	const A_COLS = 'ABCDEFGHIJ'.split('');

	const A_COLORS = [
		'f2cc9c',
		'f0b989',
		'e5a575',
	];

	const click_grid = (d_event: MouseEvent) => {
		const dm_td = (d_event.target as HTMLTableCellElement)?.closest('td');
		if(dm_td) {
			explode(+dm_td.dataset['index']!);
		}
	};

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

		const dm_grid = create_html('table', {}, A_COLS.map((_, i_y) => create_html('tr', {}, A_COLS.map((__, i_x) => {
			const x_rnd = Math.random();

			const dm_td = create_html('td', {
				'data-index': (i_y * 10)+i_x+'',
				'style': `background:#${x_rnd < 0.5? A_COLORS[0]: x_rnd < 0.9? A_COLORS[1]: A_COLORS[2]}`,
			}, []);

			return dm_td;
		}))));

		const dm_middle = create_html('div', {
			class: 'middle',
		}, [
			dm_rows, dm_grid, dm_rows.cloneNode(true),
		]);

		dm_app.append(dm_cols, dm_middle, dm_cols.cloneNode(true));

		d_2d0 = dm_overlay0.getContext('2d')!;
		d_2d1 = dm_overlay1.getContext('2d')!;

		graphics();
	});

	const graphics_rock = (xl_x: number, xl_y: number, xl_w: number, xl_h: number, xl_r: number) => {
		d_2d0.beginPath();
		d_2d0.moveTo(xl_x + xl_r, xl_y);
		d_2d0.arcTo(xl_x + xl_w, xl_y, xl_x + xl_w, xl_y + xl_h, xl_r);
		d_2d0.arcTo(xl_x + xl_w, xl_y + xl_h, xl_x, xl_y + xl_h, xl_r);
		d_2d0.arcTo(xl_x, xl_y + xl_h, xl_x, xl_y, xl_r);
		d_2d0.arcTo(xl_x, xl_y, xl_x + xl_w, xl_y, xl_r);
		d_2d0.closePath();
		d_2d0.fill();
	};

	const draw_sprite = (d_2d: CanvasRenderingContext2D, i_index: number, i_sprite: number, xl_enlarge=0) => {
		const i_y = Math.floor(i_index / 10);
		const i_x = i_index % 10;

		d_2d.setTransform(1, 0, 0, 1, 30+(i_x*40) + 20, 30+(i_y*40) + 20);
		d_2d.rotate(random(2*Math.PI));

		d_2d.drawImage(
			dm_sprites,
			i_sprite * XL_SPRITE_SRC_DIM, 0,
			XL_SPRITE_SRC_DIM, XL_SPRITE_SRC_DIM,
			-XL_SPRITE_DEST_DIM/2, -XL_SPRITE_DEST_DIM/2,
			XL_SPRITE_DEST_DIM+xl_enlarge, XL_SPRITE_DEST_DIM+xl_enlarge
		);

		d_2d.resetTransform();
	};

	const graphics_blast = (i_index: number) => {
		const i_sprite = Math.floor(random(NL_SPRITES, 1));

		d_2d0.globalCompositeOperation = 'multiply';
		d_2d0.globalAlpha = 0.95;

		draw_sprite(d_2d0, i_index, i_sprite);
	};

	const graphics_bang = (i_index: number, xl_enlarge=0) => {
		d_2d1.globalCompositeOperation = 'lighter';
		draw_sprite(d_2d1, i_index, 0, xl_enlarge);
	};

	const explode = async(i_index: number) => {
		graphics_blast(i_index);
		graphics_bang(i_index);

		await timeout(100);
		graphics_bang(i_index, 10);

		await timeout(250);
		dm_overlay1.style.opacity = '0.4';

		await timeout(150);
		d_2d1.clearRect(0, 0, 460, 460);
		dm_overlay1.style.opacity = '1';
	};

	// const load_blast = () => {
	// 	for(let i=0; i<50; i+=3) {
	// 		graphics_blast(i);
	// 	}

	// 	graphics_blast(5);
	// 	graphics_bang(5);

	// 	graphics_blast(7);
	// 	graphics_bang(7);

	// 	setTimeout(() => {
	// 		dm_overlay1.style.opacity = '0.5';
	// 		setTimeout(() => {
	// 			d_2d1.clearRect(0, 0, 460, 460);
	// 		}, 200);
	// 	}, 3e3);
	// };

	const graphics = () => {
		// dm_sprites.onload = () => {
		// 	load_blast();
		// };
	
		dm_sprites.src = SX_SPRITES;

		d_2d0.save();

		oda(d_2d0, {
			shadowOffsetX: -2,
			shadowOffsetY: 2,
			shadowBlur: 2,
			shadowColor: '#a66a4f',
		});

		d_2d0.fillStyle = '#f3cfa1';

		for(let i_rock=0; i_rock<16; i_rock++) {
			graphics_rock(
				random(400 - 30, 30),
				random(400 - 30, 30),
				random(5, 3),
				random(6, 2),
				1
			);
		}

		d_2d0.restore();
	};

	export let b_home = false;
	export let b_game_on = false;

	let dm_overlay0: HTMLCanvasElement;
	let dm_overlay1: HTMLCanvasElement;

	const dm_sprites = new Image(NL_SPRITES*XL_SPRITE_DEST_DIM, XL_SPRITE_SRC_DIM);

	let dm_app: HTMLDivElement;

	let d_2d0: CanvasRenderingContext2D;
	let d_2d1: CanvasRenderingContext2D;

</script>

<style lang="less">
	@xl_sink: 80px;
	@xl_shift: 100px;
	@xl_dim: 460px;
	@sx_pers: perspective(30cm);

	@keyframes away-entry {
		100% {
			transform: rotateX(-30deg) translate3d(0, @xl_sink, 0);
		}
	}

	@keyframes home-entry {
		40% {
			transform: rotateX(0deg) translate3d(0, 0px, -@xl_shift);
		}
		100% {
			transform: rotateX(60deg) translate3d(0, -@xl_sink, 0);
		}
	}

	.grid {
		border: 1px solid #ccc;
		display: flex;
		flex-direction: column;
		width: @xl_dim;
		height: @xl_dim;

		// transition: transform 2s ease-in-out;
		transform: rotateX(0deg) translate3d(0, 0px, -@xl_shift);

		background:
			linear-gradient(90deg, #78362855 25%, #0000 25%, #0000 50%, #a4533d55 50%, #a4533d55 75%, #0000 75%),
			linear-gradient(#8a413366 25%, #0000 25%, #0000 50%),
			linear-gradient(#ce8764 25%, #c67a58 25%, #c67a58 50%, #cc8460 50%, #cc8460 75%, #c78058 75%);
		background-size: 160px 160px;
		background-position: -10px -10px;

		&:not(.home) {
			transform: rotateX(-90deg) translate3d(0, 0px, -@xl_shift);
		}
	}

	.game-on {
		animation: 2.6s ease-in-out 1 both away-entry;
	}

	.home {
		animation-name: home-entry;
	}

	.overlay {
		position: absolute;
		pointer-events: none;
		width: 100%;
		height: 100%;
	}

</style>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="grid" bind:this={dm_app} on:click={click_grid}
	class:home={b_home}
	class:game-on={b_game_on}
>
	<div class="overlay">
		<canvas bind:this={dm_overlay0} width=460 height=460 />
	</div>
	<div class="overlay">
		<canvas bind:this={dm_overlay1} width=460 height=460 />
	</div>
</div>
