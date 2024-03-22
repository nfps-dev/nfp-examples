import type {NfpxExports} from './env';
import type {SecretAccAddr} from '@solar-republic/contractor';
import type {HttpsUrl, WeakSecretAccAddr} from '@solar-republic/neutrino';

import {create_svg, create_html, ls_write, ls_write_b64, ls_read_b64} from '@nfps.dev/runtime';
import {Wallet, gen_sk, exec_contract, SecretApp} from '@solar-republic/neutrino';

// autoimport types so that svelte components can destructure
import type {} from 'nfpx:app';

// reuse as much as possible from the bootloader to cut down on app's bundle size
import {
	A_TOKEN_LOCATION,
	K_CONTRACT,
	P_LCD,
	Z_AUTH,
	ls_read,
	nfp_tags,
	nfp_attr,
	A_COMCS,
} from 'nfpx:bootloader';

// import compiled app-wide css
import SX_CSS_APP from './app.less?inline';

// import root svelte component
import App from './App.svelte';

// document element root
const dm_root = document.documentElement;

// add app css to document
dm_root.append(create_svg('style', {}, [SX_CSS_APP]));

// read rpc data from nfp
const dm_web = nfp_tags('web')?.[0];
const A_RPCS = dm_web? nfp_attr(dm_web, 'rpcs')?.split(',') as HttpsUrl[]: [];

// create ui to allow user to play/pause animations
const dm_pause = create_html('button', {
	style: 'position:absolute;top:40em;left:22em;background:#333;color:#ce3',
}, [
	'Pause',
]);

const dm_app = create_html('div', {
	id: 'app',
});

const dm_foreign = create_svg('foreignObject', {
	width: '100%',
	height: '100%',
	x: '0',
	y: '0',
}, [
	dm_app,
]);

dm_root.append(dm_foreign);

setTimeout(() => {
	dm_app.classList.add('loaded');
}, 250);

dm_pause.onclick = () => {
	dm_pause.textContent = dm_root.classList.toggle('paused')? 'Resume': 'Pause';
};

// read optional query params to override token id
const d_params = new URLSearchParams(location.search);
A_TOKEN_LOCATION[2] = d_params.get('token-id') || A_TOKEN_LOCATION[2];

// create local storage key based on token location
const si_storage_token_owner_addr = 'toa:'+A_TOKEN_LOCATION.join(':');

// fetch token owner address
const SA_OWNER = d_params.get('owner')
	|| ls_read(si_storage_token_owner_addr) as SecretAccAddr
	|| ls_write<SecretAccAddr>(si_storage_token_owner_addr,
		(Z_AUTH
			? Array.isArray(Z_AUTH)
				? Z_AUTH[1]
				: 'object' === typeof Z_AUTH
					// TODO: make pubkey_to_bech32 synchronous (needs sha256_sync)
					// so that bech32 can be obtained from query permit
					? null
					// ? pubkey_to_bech32(base64_to_buffer((Z_AUTH as QueryPermit).signature.pub_key.value))
					: null
			: null)
		|| prompt('Please enter the account address that owns this token') || ''
	);

if(Array.isArray(Z_AUTH)) {
	Z_AUTH[0] = d_params.get('vk') || Z_AUTH[0];
	Z_AUTH[1] = (d_params.get('owner') || Z_AUTH[1]) as WeakSecretAccAddr;
}


// go async
(async() => {
	// get or create private key
	const atu8_sk = ls_read_b64('sk') || gen_sk();

	// create wallet
	const k_wallet = await Wallet(atu8_sk, A_TOKEN_LOCATION[0], P_LCD, A_RPCS[0]);

	// save to local storage
	ls_write_b64('sk', atu8_sk);
	console.log(`Hot wallet address: ${k_wallet.addr}`);

	// dynamic export before importing libs (which depend on these exports)
	exportNfpx({
		K_WALLET: k_wallet,
		K_SERVICE: SecretApp(k_wallet, K_CONTRACT, 0.125),
		Z_AUTH: Z_AUTH!,
	});

	// launch the app
	new App({
		target: dm_app,
	});
})();

// augment the collection of data and functions that can be reused by other modules
export {
	A_RPCS,
	A_COMCS,
	SA_OWNER,
	exec_contract,
	dm_root,
	dm_foreign,
};

export default interface Default extends NfpxExports {
	merge: string;
}
