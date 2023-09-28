/// <reference types="svelte" />
/// <reference types="@nfps.dev/sdk/env-vars" />
/// <reference types="@nfps.dev/sdk/nfpx" />

import type {AppInterface} from './interface/app';
import type {QueryPermit} from '@solar-republic/contractor';
import type {AuthSecret_ViewerInfo, SecretApp, Wallet} from '@solar-republic/neutrino';

// declare what the app can export dynamically
export interface NfpxExports {
	K_WALLET: Wallet;
	K_SERVICE: SecretApp<AppInterface>;
	Z_AUTH: AuthSecret_ViewerInfo | QueryPermit;
}

/* eslint-disable @typescript-eslint/naming-convention,no-var */
declare global {
	function exportNfpx(h_exports: NfpxExports): void;
}
