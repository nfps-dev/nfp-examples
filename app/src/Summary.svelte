<script lang="ts">
	import {SX_VEHICLES, clip_dims} from './vehicles';

	export let h_destroyed: Record<number, 1>;
</script>

{#if Object.keys(h_destroyed).length}
	{#each Object.keys(h_destroyed).sort() as xc_vehicle}
		{@const [sx_preview_w, sx_preview_h, sx_preview_pos] = clip_dims(+xc_vehicle)}
		<div class="preview-obj"
			style:background-image={`url('${SX_VEHICLES}')`}
			style:background-position={sx_preview_pos}
			style:width={sx_preview_w}
			style:height={sx_preview_h}
			style:transform='scale(0.5)'
			style:transform-origin='top left'
			style:margin-bottom='-50px'
		/>
	{/each}
{:else}
	<em>None yet</em>
{/if}
