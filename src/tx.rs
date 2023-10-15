use bytes::Buf;

struct NttxChunk<'a> {
    magic: &'a [u8],
    _unknown1: u32,
    file_size: u32,
    chunk_size: u16,
    _unknown2: u16,
}

struct PaltChunk<'a> {
    magic: &'a [u8],
    chunk_size: u32,
    palette_count: u32,
    palette: &'a [u8],
}

struct ImgeChunk<'a> {
    magic: &'a [u8],
    _chunk_size: u32,
    _unknown: u32,
    width: u16,
    height: u16,
    img_size: u32,
    img_data: &'a [u8],
}

impl<'a> NttxChunk<'a> {
    pub fn new(mut slice: &'a [u8]) -> Self {
        let magic = &slice[0..4];
        slice.advance(4);

        Self {
            magic,
            _unknown1: slice.get_u32_le(),
            file_size: slice.get_u32_le(),
            chunk_size: slice.get_u16_le(),
            _unknown2: slice.get_u16_le(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == b"NTTX"
    }
}

impl<'a> PaltChunk<'a> {
    pub fn new(mut slice: &'a [u8]) -> Self {
        let magic = &slice[0..4];
        slice.advance(4);
        let chunk_size = slice.get_u32_le();
        let palette_count = slice.get_u32_le();
        let palette = &slice[12..12 + palette_count as usize * 2];

        Self {
            magic,
            chunk_size,
            palette_count,
            palette,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == b"PALT"
    }
}

impl<'a> ImgeChunk<'a> {
    pub fn new(mut slice: &'a [u8]) -> Self {
        let magic = &slice[0..4];
        slice.advance(4);

        Self {
            magic,
            _chunk_size: slice.get_u32_le(),
            _unknown: slice.get_u32_le(),
            width: slice.get_u16_le(),
            height: slice.get_u16_le(),
            img_size: slice.get_u32_le(),
            img_data: &slice[..],
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == b"IMGE" && self.img_data.len() == self.img_size as usize
    }
}

// BUG: generated image using wrong color
pub fn convert(src: &[u8]) -> (usize, usize, Vec<u8>) {
    let nttx = NttxChunk::new(&src[..]);
    let palt = PaltChunk::new(&src[nttx.chunk_size as usize..]);
    let imge = ImgeChunk::new(&src[nttx.chunk_size as usize + palt.chunk_size as usize..]);

    assert_eq!(nttx.file_size as usize, src.len());
    assert!(nttx.is_valid() && palt.is_valid() && imge.is_valid());

    // HACK
    let palette = super::txb::extract_palette(palt.palette, palt.palette_count as usize);

    // FIXME: causing panic because division by zero
    let bpp = 8 / ((imge.width * imge.height) as u32 / imge.img_size);
    let mut image_data = Vec::with_capacity((imge.width * imge.height) as usize * 3);

    match bpp {
        4 => {
            for byte in imge.img_data {
                let nibble1 = (byte & 0xF) as usize;
                let nibble2 = ((byte & 0xF0) >> 4) as usize;

                image_data.extend_from_slice(&palette[nibble1]);
                image_data.extend_from_slice(&palette[nibble2]);
            }
        }
        2 => {
            for byte in imge.img_data {
                let px1 = (byte & 0b11) as usize;
                let px2 = ((byte & 0b11 << 2) >> 2) as usize;
                let px3 = ((byte & 0b11 << 4) >> 4) as usize;
                let px4 = ((byte & 0b11 << 6) >> 6) as usize;

                image_data.extend_from_slice(&palette[px1]);
                image_data.extend_from_slice(&palette[px2]);
                image_data.extend_from_slice(&palette[px3]);
                image_data.extend_from_slice(&palette[px4]);
            }
        }
        // TODO: add another bpp support
        other => panic!("Unsupported bit per pixel: {other}"),
    };

    (imge.width as usize, imge.height as usize, image_data)
}

// TODO: write function to extract palette
