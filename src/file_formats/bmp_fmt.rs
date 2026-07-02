use bmp;
use std::io::{Cursor, Error, ErrorKind};

pub fn to_rwmj_rgba<const N: usize>(bmp: &[u8]) -> Result<[u8; N], Error> {
    let image = bmp::from_reader(&mut Cursor::new(bmp))
        .map_err(|e| Error::new(ErrorKind::InvalidData, e.details))?;

    let mut buffer = [0u8; N];
    let (height, width) = (image.get_height() as usize, image.get_width() as usize);
    if N != height * width * 4 {
        return Err(Error::new(ErrorKind::InvalidData, format!(
            "Will compute a(n) {}-sized array when expecting a(n) {N}-sized array", height * width * 4
        )));
    }

    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x as u32, y as u32);
            let offset = (y * width + x) * 4;
            buffer[offset] = pixel.r;
            buffer[offset + 1] = pixel.g;
            buffer[offset + 2] = pixel.b;
            buffer[offset + 3] = 255;
        }
    }
    Ok(buffer)
}