<script lang="ts">
	import {oda, timeout} from '@blake.regalia/belt';
	import {create_html, qs, qsa} from '@nfps.dev/runtime';
	
	import {createEventDispatcher, onMount} from 'svelte';
	
	import {random} from './graphics';
	
	import {NL_SPRITES, draw_sprite} from './sprites';
	import {A_VEHICLES, H_VEHICLE_NAMES, H_VEHICLE_WIDTHS, SX_VEHICLES, clip_dims, draw_vehicle, draw_x} from './vehicles';

	const XL_CANVAS_DIM = 460;

	const A_COLS = 'ABCDEFGHIJ'.split('');

	const A_COLORS = [
		'f2cc9c',
		'f0b989',
		'e5a575',
	];

	const A_PLACEMENTS: [number, number, boolean][] = [];

	const dispatch = createEventDispatcher();

	export let b_home = false;
	export let b_game_on = false;

	let a_grid = Array(100).fill(0);

	let dm_overlay0: HTMLCanvasElement;
	let dm_overlay1: HTMLCanvasElement;
	let dm_overlay2: HTMLCanvasElement;


	let dm_grid: HTMLDivElement;

	let d_2d0: CanvasRenderingContext2D;
	let d_2d1: CanvasRenderingContext2D;
	let d_2d2: CanvasRenderingContext2D;


	// the cell the user is targetting with cursor
	let i_index_target = 0;

	// the closest in-bounds cell the vehicle placement can start at
	let i_index_placement = 0;

	// index of which vehicle is being placed (-1 means not in placement mode)
	let i_vehicle = b_home && !b_game_on? 0: -1;

	// code of vehicle currently being placed
	$: xc_vehicle = i_vehicle < 0? 0: A_VEHICLES[i_vehicle];

	// clipping path for extracting vehicle from sprite-sheet
	// $: [sx_clip, sx_obj_pos] = i_vehicle < A_VEHICLES.length? clip_path(A_VEHICLES[i_vehicle]): 'none';
	$: [sx_preview_w, sx_preview_h, sx_preview_pos] = i_vehicle < 0? []: clip_dims(A_VEHICLES[i_vehicle]);

	let a_grid_prospective = a_grid.slice();
	let b_intersects = false;



	const shift_key_listener = (d_event: KeyboardEvent) => {
		// reset to target cell
		i_index_placement = i_index_target;

		position_vehicle(d_event.shiftKey);
	};

	const place_vehicle = (d_event: MouseEvent) => {
		const dm_td = d_event.currentTarget as HTMLTableCellElement;
		i_index_placement = i_index_target = +dm_td.dataset['index']!;

		position_vehicle(d_event.shiftKey);
	};

	const bind_placement_listeners = () => {
		// eslint-disable-next-line array-callback-return
		qsa(dm_grid, 'td').map((dm_td) => {
			dm_td.addEventListener('mouseenter', place_vehicle);
		});

		document.addEventListener('keydown', shift_key_listener);
		document.addEventListener('keyup', shift_key_listener);
	};

	const unbind_placement_listeners = () => {
		// eslint-disable-next-line array-callback-return
		qsa(dm_grid, 'td').map((dm_td) => {
			dm_td.removeEventListener('mouseenter', place_vehicle);
		});

		document.removeEventListener('keydown', shift_key_listener);
		document.removeEventListener('keyup', shift_key_listener);
	};

	const position_vehicle = (b_rotated: boolean) => {
		// vehicle canvas
		d_2d2.clearRect(0, 0, XL_CANVAS_DIM, XL_CANVAS_DIM);

		// un-hover all cells
		qsa(dm_grid, 'td.hover').map(dm => dm.classList.remove('hover'));

		// lookup current vehicle id and width
		const x_vehicle_width = H_VEHICLE_WIDTHS[xc_vehicle];

		// // re-cast vehicle id
		// const xc_vehicle = +s_vehicle;

		// snap vehicle placement start cell within bounds
		if(b_rotated) {
			for(; 10 - Math.floor(i_index_placement / 10) < x_vehicle_width; i_index_placement-=10);
		}
		else {
			for(; 10 - (i_index_placement % 10) < x_vehicle_width; i_index_placement--);
		}

		// draw the vehicle
		draw_vehicle(d_2d2, xc_vehicle as number, i_index_placement, b_rotated);

		// draw previous placements
		for(const [xc_vehicle_placed, i_index, b_rot] of A_PLACEMENTS) {
			draw_vehicle(d_2d2, xc_vehicle_placed, i_index, b_rot);
		}

		// reset intersection flag
		b_intersects = false;

		// reset prospective grid vector
		a_grid_prospective = [...a_grid];

		// check each footprint cell for intersection
		for(let c_span=0, i_index_footprint=i_index_placement; c_span<x_vehicle_width; c_span++, i_index_footprint+=b_rotated? 10: 1) {
			// cell is occupied
			if(a_grid_prospective[i_index_footprint]) {
				// draw "X"
				draw_x(d_2d2, i_index_footprint);

				// set intersection flag
				b_intersects = true;
			}
			// cell is available
			else {
				a_grid_prospective[i_index_footprint] = xc_vehicle;
			}

			// hover cell
			qs(dm_grid, `td[data-index="${i_index_footprint}"]`)?.classList.add('hover');
		}
	};

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
		d_2d1.clearRect(0, 0, XL_CANVAS_DIM, XL_CANVAS_DIM);
		dm_overlay1.style.opacity = '1';
	};


	const graphics = () => {
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

	const click_grid = (d_event: MouseEvent) => {
		// find table cell element
		const dm_td = (d_event.target as HTMLTableCellElement)?.closest('td');
		if(dm_td) {
			// home grid
			if(b_home) {
				// no intersection
				if(!b_intersects) {
					// save vehicle placement
					A_PLACEMENTS.push([A_VEHICLES[i_vehicle], i_index_placement, d_event.shiftKey]);

					// update grid
					a_grid = a_grid_prospective;

					// style table cells
					qsa(dm_grid, 'td.hover').map(dm => dm.classList.add('set'));

					// not done yet
					if(i_vehicle + 1 < A_VEHICLES.length) {
						// move onto next vehicle
						i_vehicle += 1;
					}
					// more vehicles remain
					else {
						// end placement mode
						i_vehicle = -1;
	
						// remove listeners
						unbind_placement_listeners();

						// remove all td classes
						qsa(dm_grid, 'td').map(dm => dm.className = '');
	
						// submit setup
						dispatch('submit', a_grid);
					}
				}
			}
			else {
				void explode(+dm_td.dataset['index']!);
			}
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

		const dm_table = create_html('table', {}, A_COLS.map((_, i_y) => create_html('tr', {}, A_COLS.map((__, i_x) => {
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
			dm_rows, dm_table, dm_rows.cloneNode(true),
		]);

		dm_grid.append(dm_cols, dm_middle, dm_cols.cloneNode(true));

		d_2d0 = dm_overlay0.getContext('2d')!;
		d_2d1 = dm_overlay1.getContext('2d')!;
		d_2d2 = dm_overlay2.getContext('2d')!;

		d_2d2.globalCompositeOperation = 'source-over';

		graphics();

		if(b_home && !b_game_on) {
			bind_placement_listeners();
		}
	});

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

	section {
		display: flex;
		transform-style: preserve-3d;
		// transition: transform 2s ease-in-out;
		transform: rotateX(0deg) translate3d(0, 0px, 0px);

		&:not(.home) {
			transform: rotateX(-90deg) translate3d(0, 0px, -@xl_shift);
		}
	}

	.home {
		animation-name: home-entry;
		animation-delay: 250ms;
	}

	.game-on {
		animation: 2.6s ease-in-out 1 both away-entry;
	}

	.grid {
		border: 1px solid #ccc;
		display: flex;
		flex-direction: column;
		width: @xl_dim;
		height: @xl_dim;

		background:
			linear-gradient(90deg, #78362855 25%, #0000 25%, #0000 50%, #a4533d55 50%, #a4533d55 75%, #0000 75%),
			linear-gradient(#8a413366 25%, #0000 25%, #0000 50%),
			linear-gradient(#ce8764 25%, #c67a58 25%, #c67a58 50%, #cc8460 50%, #cc8460 75%, #c78058 75%);
		background-size: 160px 160px;
		background-position: -10px -10px;
	}

	.overlay {
		position: absolute;
		pointer-events: none;
		width: 100%;
		height: 100%;
	}

	.info {
		margin-left: 2em;

		>div {
			margin: 6px 0;
		}
	}

	dt,dd {
		display: inline-block;
	}

	dt {
		font-weight: 600;
	}

	dd {
		text-transform: uppercase;;
	}

	.preview-obj {
		transform-origin: 0 0;
		transform: scale(0.75);
		background-repeat: no-repeat;
	}

	.preview-img {
		transform-origin: 0 0;
		transform: scale(0.75);
		width: 540px;
		height: 140px;
	}
</style>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<section
	class:home={b_home}
	class:game-on={b_game_on}
>
	<div class="grid" bind:this={dm_grid} on:click={click_grid}>
		<div class="overlay">
			<canvas bind:this={dm_overlay0} width={XL_CANVAS_DIM} height={XL_CANVAS_DIM} />
		</div>
		<div class="overlay">
			<canvas bind:this={dm_overlay1} width={XL_CANVAS_DIM} height={XL_CANVAS_DIM} />
		</div>
		<div class="overlay">
			<canvas bind:this={dm_overlay2} width={XL_CANVAS_DIM} height={XL_CANVAS_DIM} />
		</div>
	</div>

	<!-- placement mode -->
	{#if i_vehicle >= 0}
		<div class="info">
			<h3>
				Place vehicle:
			</h3>

			<div>
				<em>
					Hold SHIFT to rotate 90 degrees.
				</em>
			</div>

			<div>
				<dt>
					Class:
				</dt>
				<dd>
					{H_VEHICLE_NAMES[A_VEHICLES[i_vehicle]]}
				</dd>
			</div>

			<div>
				<dt>
					Size:
				</dt>
				<dd>
					{H_VEHICLE_WIDTHS[xc_vehicle]} units
				</dd>
			</div>

			<div class="preview-obj"
				style:background-image={`url('${SX_VEHICLES}')`}
				style:background-position={sx_preview_pos}
				style:width={sx_preview_w}
				style:height={sx_preview_h}
			/>

			<!-- <div class="preview-img">
				<img src={SX_VEHICLES} alt="" style:clip-path={sx_clip} style:object-position={sx_obj_pos} />
			</div> -->
		</div>
	{/if}
</section>