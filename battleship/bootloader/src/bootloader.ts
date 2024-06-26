import type {NfpxExports} from './env';

import type {AppInterface} from './interface/app';
import type {BootInfo} from '@nfps.dev/runtime';
import type {HttpsUrl} from '@solar-republic/neutrino';

import {
	boot,
	load_script,
	ls_read,
	ls_write,
	ls_read_json,
	ls_write_json,
	nfp_attr,
	nfp_tags,
} from '@nfps.dev/runtime';

import {
	query_contract_infer,
} from '@solar-republic/neutrino';

// import autoboot as a module so that it can be optimized out in production
import autoboot from './_autoboot';

// for side-effects
import './common';

// development mode; perform autoboot
if(import.meta.env.DEV) {
	autoboot();
}
// production
else {
	// bind boot function to global so that user has to click to boot
	window.boot = async(d_event) => {
		const dm_button = d_event.target as HTMLButtonElement;
		dm_button.textContent = 'Connecting...';
		dm_button.disabled = true;

		// boot and save bootinfo
		let a_info: BootInfo | void;
		if((a_info=await boot())) {  // eslint-disable-line @typescript-eslint/no-extra-parens
			const dm_div = dm_button.closest('div')!;
			dm_div.firstChild!.textContent = 'Loaded latest app from chain';
			dm_div.style.background = '#2ec';
			dm_div.style.opacity = '0';

			dm_button.textContent = 'Connected';

			// save reusable globals
			exportNfpx({
				// boot info
				A_TOKEN_LOCATION: a_info[1],
				P_LCD: a_info[2],
				K_CONTRACT: a_info[3],
				Z_AUTH: a_info[4],

				// comc hosts
				A_COMCS: import.meta.env.WEB_COMCS!.split(',') as HttpsUrl[],
			});

			// inject scripts
			a_info[0]();
		}
		// boot failed
		else {
			dm_button.textContent = 'Reset';
			dm_button.disabled = false;
			dm_button.onclick = () => {
				dm_button.disabled = true;
				localStorage.clear();
				dm_button.textContent = 'Done. Reload to retry';
			};

			// await import('nfpx:public', {
			// 	contract: k_contract,
			// 	location: [si_chain, k_contract.addr],
			// });

			// dm_button.parentElement?.append(create_html('button', {
			// 	onclick: 'mint',
			// }, [
			// 	`Don't have one yet? Mint your own`,
			// ]));
		}
	};
}

// export anything that may come in handy to reuse in the app or applibs
export {
	load_script,
	ls_read,
	ls_write,
	ls_read_json,
	ls_write_json,
	query_contract_infer,
	nfp_tags,
	nfp_attr,
};

export default interface Default extends NfpxExports {}
