import {index_to_crd, random} from './graphics';

import SX_SPRITES from '../../media/explosion-sprites-lo.png';

export const NL_SPRITES = 4;

const XL_SPRITE_SRC_DIM = 240 / 4;
const XL_SPRITE_DEST_DIM = 50;

const dm_sprites = new Image(NL_SPRITES*XL_SPRITE_DEST_DIM, XL_SPRITE_SRC_DIM);
dm_sprites.src = SX_SPRITES;


export const draw_sprite = (d_2d: CanvasRenderingContext2D, i_index: number, i_sprite: number, xl_enlarge=0) => {
	const [xl_x, xl_y] = index_to_crd(i_index, 0.5);

	d_2d.setTransform(1, 0, 0, 1, xl_x, xl_y);
	d_2d.rotate(random(2*Math.PI));

	d_2d.drawImage(
		dm_sprites,
		i_sprite * XL_SPRITE_SRC_DIM, 0,
		XL_SPRITE_SRC_DIM, XL_SPRITE_SRC_DIM,
		-XL_SPRITE_DEST_DIM/2, -XL_SPRITE_DEST_DIM/2,
		XL_SPRITE_DEST_DIM+xl_enlarge, XL_SPRITE_DEST_DIM+xl_enlarge
	);

	d_2d.resetTransform();
};
