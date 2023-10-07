import {qsa} from '@nfps.dev/runtime';

// reference the root document element (svg)
export const dm_root = document.documentElement;

// dismiss the banner
window.dismiss = () => document.getElementById('dismiss')!.click();

// show onlyscripts
qsa(dm_root, '.onlyscript').map(dm_elmt => dm_elmt.classList.remove('onlyscript'));

// // fix viewport scale for embedded HTML
// {
// 	const xl_w = visualViewport?.width || window.innerWidth;
// 	const xl_h = visualViewport?.height || window.innerHeight;
// 	const xl_dim = Math.floor(Math.min(xl_w, xl_h));
// 	dm_root.setAttribute('viewBox', `0 0 ${xl_dim} ${xl_dim}`);
// 	dm_root.querySelector('.autoscale')?.setAttribute('style', `transform:scale(${xl_dim / 6400})`);
// }

// const XL_WIDTH = 853;

// workaround for firefox not animating foreignObject opacity
setTimeout(() => {
	qsa(dm_root, 'foreignObject').map(dm_elmt => dm_elmt.setAttribute('opacity', '1'));
}, 1e3);

// // handle resize
// const resize = () => {
// 	console.warn('#resize');
// 	qsa(dm_root, 'foreignObject').map((dm_elmt) => {
// 		const xl_w = visualViewport?.width || window.innerWidth;
// 		const xl_h = visualViewport?.height || window.innerHeight;

// 		const xl_off_x = xl_w < xl_h? xl_w - XL_WIDTH: XL_WIDTH - xl_w;

// 		const xl_x = xl_w < 800? 0: Math.min(0, Math.round(xl_off_x / 2));
// 		console.warn('#resize: '+xl_x, {
// 			xl_w: xl_w,
// 			vvw: visualViewport?.width,
// 			wiw: window.innerWidth,
// 			XLW: XL_WIDTH,
// 		});
// 		dm_elmt.setAttribute('x', xl_x+'');
// 	});
// };

// let i_resize = 0;
// addEventListener('resize', () => {
// 	clearTimeout(i_resize);
// 	i_resize = window.setTimeout(resize, 250);
// });

// addEventListener('load', resize);

// new MutationObserver((a_records, d_observer) => {
// 	let b_resize = false;
// 	a_records.map((d_record) => {
// 		if([...d_record.addedNodes].some(dm => 'foreignObject' === dm.nodeName)) b_resize = true;
// 	});
// 	if(b_resize) resize();
// }).observe(dm_root, {childList:true});
