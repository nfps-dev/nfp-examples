import {writable} from 'svelte/store';

export const yxt_now = writable(Date.now());

setInterval(() => {
	yxt_now.set(Date.now());
}, 1e3);


export const XG_LIMIT_BASE = 58_000n;

export const X_GAS_PRICE_DEFAULT = 0.125;
