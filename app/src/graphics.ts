
const index_to_xy = (i_index: number): [number, number] => [i_index % 10, Math.floor(i_index / 10)];

export const index_to_crd = (i_index: number, xs_off=0): [number, number] => index_to_xy(i_index).map(i => 30 + (i * 40) + xs_off * 40) as [number, number];

export const random = (x_range: number, x_min: number=0): number => (Math.random() * x_range) + x_min;
