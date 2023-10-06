use bytes::Buf;
use std::io::prelude::Read;

pub fn parse_to_linear(mut src: &[u8]) -> (usize, usize, Vec<u8>) {
    let _unknown = src.get_u32_le();
    let width = src.get_u32_le() as usize;
    let height = src.get_u32_le() as usize;
    let tile_width = src.get_u32_le() as usize;
    let tile_height = src.get_u32_le() as usize;
    let palette_size = src.get_u32_le() as usize;
    let palette = extract_palette(&src.copy_to_bytes(palette_size * 2), palette_size);
    let raw_size = src.get_u32_le() as usize;

    let mut tiled_image = src.copy_to_bytes(raw_size).reader();
    let mut indexed_linear_image = vec![0; raw_size];

    assert!(palette_size <= 16);

    // image is 4bpp so we need to divide by two
    let column = width / tile_width;
    let row = height / tile_height;
    let tile_size = tile_width * tile_height / 2;

    let mut tile = vec![0; tile_size];

    for y in 0..row {
        let base_y = y * tile_height;

        for x in 0..column {
            let base_x = x * tile_width;
            tiled_image.read_exact(&mut tile).expect("failed to read");

            let tile_row_size = tile_width / 2;

            for (i, tile_row) in tile.chunks_exact(tile_row_size).enumerate() {
                let pos = ((base_y + i) * width / 2) + (base_x / 2);
                indexed_linear_image[pos..pos + tile_row_size].copy_from_slice(&tile_row);
            }
        }
    }

    // pixel format for output is rgb24
    let mut linear_image = Vec::with_capacity(width * height * 3);

    for byte in indexed_linear_image.into_iter() {
        let nibble1 = (byte & 0xF) as usize;
        let nibble2 = ((byte & 0xF0) >> 4) as usize;

        linear_image.extend_from_slice(&palette[nibble1]);
        linear_image.extend_from_slice(&palette[nibble2]);
    }

    (width, height, linear_image)
}

fn extract_palette(mut slice: &[u8], palette_size: usize) -> Vec<[u8; 3]> {
    let mut pal = Vec::with_capacity(palette_size);

    for _ in 0..palette_size {
        let col = slice.get_u16_le();
        let mut r = ((col & 0x1F) * 8) as u8;
        let mut g = (((col & 0x3E0) >> 5) * 8) as u8;
        let mut b = (((col & 0x7C00) >> 10) * 8) as u8;

        r = r + r / 32;
        g = g + g / 32;
        b = b + b / 32;

        pal.push([r, g, b]);
    }

    pal
}
