import {oda} from '@blake.regalia/belt';

import {index_to_crd} from './graphics';

import SX_DEBRIS from '../../media/debris.png';
import SX_VEHICLES from '../../media/vehicles.png';

export {SX_VEHICLES};

export const XC_VEHICLE_TITAN = 2;
export const XC_VEHICLE_ENFORCER = 3;
export const XC_VEHICLE_DRIFTER = 4;
export const XC_VEHICLE_CRAWLER = 5;
export const XC_VEHICLE_SCOUT = 6;

export const A_VEHICLES = [
	XC_VEHICLE_TITAN,
	XC_VEHICLE_ENFORCER,
	XC_VEHICLE_DRIFTER,
	XC_VEHICLE_CRAWLER,
	XC_VEHICLE_SCOUT,
];

export const H_VEHICLE_WIDTHS: Record<number, number> = {
	[XC_VEHICLE_TITAN]: 5,
	[XC_VEHICLE_ENFORCER]: 4,
	[XC_VEHICLE_DRIFTER]: 3,
	[XC_VEHICLE_CRAWLER]: 3,
	[XC_VEHICLE_SCOUT]: 2,
};

export const H_VEHICLE_NAMES: Record<number, string> = {
	[XC_VEHICLE_TITAN]: 'Titan',
	[XC_VEHICLE_ENFORCER]: 'Enforcer',
	[XC_VEHICLE_DRIFTER]: 'Drifter',
	[XC_VEHICLE_CRAWLER]: 'Crawler',
	[XC_VEHICLE_SCOUT]: 'Scout',
};

const XL_SRC_ALIVE_W = 540;
const XL_SRC_ALIVE_H = 480;

const XL_SRC_DEAD_W = 405;
const XL_SRC_DEAD_H = 360;

const XL_DST_W = 5.4 * 40;  // 5 cols + 2/5ths
const XL_DST_H = 4.8 * 40;  // 4 rows + 4/5ths

const XL_INNER_Y = 100 / XL_SRC_ALIVE_H;
const XL_OUTER_Y = 140 / XL_SRC_ALIVE_H;
const XL_OFF_X = 20 / XL_SRC_ALIVE_W;
const XL_OFF_Y = 20 / XL_SRC_ALIVE_H;

const H_COORDS: Record<number, [number, number, number, number, number, number]> = {
	[XC_VEHICLE_TITAN]: [0, 0, 1, XL_OUTER_Y, XL_OFF_X, XL_OFF_Y],
	[XC_VEHICLE_ENFORCER]: [0, 0.5, 440/540, XL_OUTER_Y, XL_OFF_X, XL_OFF_Y],
	[XC_VEHICLE_DRIFTER]: [200/540, 140/480, 340/540, XL_INNER_Y, XL_OFF_X, 0],
	[XC_VEHICLE_CRAWLER]: [220/540, 380/480, 320/540, XL_INNER_Y, 0, 0],
	[XC_VEHICLE_SCOUT]: [0, 380/480, 220/540, XL_INNER_Y, XL_OFF_X, 0],
};

const dm_alive = new Image(XL_SRC_ALIVE_W, XL_SRC_ALIVE_H);
dm_alive.src = SX_VEHICLES;

const dm_debris = new Image(XL_SRC_DEAD_W, XL_SRC_DEAD_H);
dm_debris.src = SX_DEBRIS;

// export const clip_path = (
// 	xc_vehicle: number,
// 	[xl_src_x, xl_src_y, xl_src_w, xl_src_h, xl_off_x, xl_off_y]=H_COORDS[xc_vehicle]!
// ) => [
// 	`inset(${xl_src_y * XL_SRC_H}px ${(1 - (xl_src_x + xl_src_w)) * XL_SRC_W}px ${(1 - (xl_src_y + xl_src_h + xl_off_y)) * XL_SRC_H}px ${xl_src_x * XL_SRC_W}px)`,
// 	`-${xl_src_x * XL_SRC_W}px -${xl_src_y * XL_SRC_H}px`,
// ];

export const clip_dims = (
	xc_vehicle: number,
	[xl_src_x, xl_src_y, xl_src_w, xl_src_h, xl_off_x, xl_off_y]=H_COORDS[xc_vehicle]!
) => [
	`${xl_src_w * XL_SRC_ALIVE_W}px`,
	`${xl_src_h * XL_SRC_ALIVE_H}px`,
	`-${xl_src_x * XL_SRC_ALIVE_W}px -${xl_src_y * XL_SRC_ALIVE_H}px`,
];

// export const draw_body = (
// 	d_2d: CanvasRenderingContext2D,
// 	dm_sheet: HTMLImageElement,
// 	xl_src_w: number,
// 	xl_src_h: number,
// 	xc_vehicle: number,
// 	i_index: number,
// 	b_rot: boolean
// ) => {
// 	const [xs_src_x, xs_src_y, xs_src_w, xs_src_h, xs_off_x, xs_off_y] = H_COORDS[xc_vehicle]!;

// 	oda(d_2d, {
// 		shadowOffsetX: -4,
// 		shadowOffsetY: 8,
// 		shadowBlur: 2,
// 		shadowColor: '#6f4735',
// 	});

// 	const [xl_x, xl_y] = index_to_crd(i_index, 0.5);

// 	d_2d.setTransform(1, 0, 0, 1, xl_x, xl_y);
// 	if(b_rot) d_2d.rotate(Math.PI/2);

// 	d_2d.drawImage(dm_sheet,
// 		xs_src_x * xl_src_w, xs_src_y * xl_src_h,
// 		xs_src_w * xl_src_w, xs_src_h * xl_src_h,
// 		-20 - (xs_off_x * XL_DST_W), -20 - (xs_off_y * XL_DST_H),
// 		xs_src_w * XL_DST_W, xs_src_h * XL_DST_H
// 	);

// 	d_2d.resetTransform();
// };


export const draw_vehicle = (
	d_2d: CanvasRenderingContext2D,
	xc_vehicle: number,
	i_index: number,
	b_rot=false
) => {
	const [xl_src_x, xl_src_y, xl_src_w, xl_src_h, xl_off_x, xl_off_y] = H_COORDS[xc_vehicle]!;

	oda(d_2d, {
		shadowOffsetX: -4,
		shadowOffsetY: 8,
		shadowBlur: 2,
		shadowColor: '#6f4735',
	});

	const [xl_x, xl_y] = index_to_crd(i_index, 0.5);

	d_2d.setTransform(1, 0, 0, 1, xl_x, xl_y);
	if(b_rot) d_2d.rotate(Math.PI/2);

	d_2d.drawImage(dm_alive,
		xl_src_x * XL_SRC_ALIVE_W, xl_src_y * XL_SRC_ALIVE_H,
		xl_src_w * XL_SRC_ALIVE_W, xl_src_h * XL_SRC_ALIVE_H,
		-20 - (xl_off_x * XL_DST_W), -20 - (xl_off_y * XL_DST_H),
		xl_src_w * XL_DST_W, xl_src_h * XL_DST_H
	);

	d_2d.resetTransform();
};

export const draw_destroyed = (
	d_2d: CanvasRenderingContext2D,
	xc_vehicle: number,
	i_index: number,
	b_rot=false
) => {
	const [xl_src_x, xl_src_y, xl_src_w, xl_src_h, xl_off_x, xl_off_y] = H_COORDS[xc_vehicle]!;

	oda(d_2d, {
		shadowOffsetX: -4,
		shadowOffsetY: 8,
		shadowBlur: 2,
		shadowColor: '#6f4735',
	});

	const [xl_x, xl_y] = index_to_crd(i_index, 0.5);

	d_2d.setTransform(1, 0, 0, 1, xl_x, xl_y);
	if(b_rot) d_2d.rotate(Math.PI/2);

	d_2d.drawImage(dm_debris,
		xl_src_x * XL_SRC_DEAD_W, xl_src_y * XL_SRC_DEAD_H,
		xl_src_w * XL_SRC_DEAD_W, xl_src_h * XL_SRC_DEAD_H,
		-20 - (xl_off_x * XL_DST_W), -20 - (xl_off_y * XL_DST_H),
		xl_src_w * XL_DST_W, xl_src_h * XL_DST_H
	);

	d_2d.resetTransform();
};

export const draw_x = (
	d_2d: CanvasRenderingContext2D,
	i_index: number
): void => {
	oda(d_2d, {
		shadowOffsetX: 0,
		shadowOffsetY: 0,
		shadowBlur: 0,
		shadowColor: '#0000',
	});

	const [xl_x, xl_y] = index_to_crd(i_index);

	d_2d.strokeStyle = '#d11';
	d_2d.lineWidth = 5;
	d_2d.beginPath();
	d_2d.moveTo(xl_x, xl_y);
	d_2d.lineTo(xl_x + 40, xl_y + 40);
	d_2d.moveTo(xl_x, xl_y+40);
	d_2d.lineTo(xl_x + 40, xl_y);
	d_2d.stroke();
};
