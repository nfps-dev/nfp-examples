/**
 * This script is only used in the development environment. It's purpose is to emulate the boot process
 * offline instead of forcing you, the developer, to perform an actual boot every time you reload.
 */
import type {SecretAccAddr} from '@solar-republic/contractor';
import type {HttpsUrl, WeakSecretAccAddr} from '@solar-republic/neutrino';

import {
	create_svg,
	qsa,
} from '@nfps.dev/runtime';

import {SecretContract} from '@solar-republic/neutrino';

import {dm_root} from './common';

// exporting as a function ensures no side effects when imported
export default function autoboot(): void {
	// show onlyscripts
	qsa(dm_root, '.onlyscript').map(dm_elmt => dm_elmt.classList.remove('onlyscript'));

	// env vars
	const h_env = import.meta.env;

	// wait for document to load
	addEventListener('load', async() => {
		const sa_contract = h_env.SELF_CONTRACT as SecretAccAddr;
		const p_lcd = h_env.WEB_LCDS?.split(',')[0] as HttpsUrl;

		// override sdk's default script loader with a custom one to link to local package instead
		const load_script = nfpx.l = si_package => Promise.resolve(create_svg('script', {
			href: `./${si_package.replace(/\.js$/, '.dev.js')}`,
		}));

		// save reusable globals
		exportNfpx({
			// boot info
			A_TOKEN_LOCATION: [h_env.SELF_CHAIN!, sa_contract, h_env.SELF_TOKEN!],
			P_LCD: p_lcd,
			K_CONTRACT: await SecretContract(p_lcd, sa_contract),
			A_COMCS: h_env.WEB_COMCS!.split(',') as HttpsUrl[],
			Z_AUTH: [h_env.VIEWING_KEY!, h_env.OWNER as WeakSecretAccAddr],

			// export the custom script loader
			load_script,
		});

		// inject the app
		dm_root.append(create_svg('script', {
			href: './app.dev.js',
		}));

		// hide banner
		qsa(dm_root, '#default div')[0].style.display = 'none';
	});
}
