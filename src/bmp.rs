use std::{
    fmt::Display,
    fs::File,
    io::{Read, Write},
};

#[derive(Debug)]
pub enum Error {
    FileError(std::io::Error),
    InvalidSignature,
    InvalidHeaderSize(u32),
    UnsupportedPlaneCount(u16),
    UnsupportedColorDepth(u16),
    UnsupportedCompression(u32),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::FileError(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FileError(e) => write!(f, "File Error: {e}"),
            Error::InvalidSignature => write!(f, "Invalid Signature"),
            Error::InvalidHeaderSize(e) => write!(f, "Invalid header size, expected 40, got {e}"),
            Error::UnsupportedPlaneCount(e) => {
                write!(f, "Unsupported plane count, expected 1, got {e}")
            }
            Error::UnsupportedColorDepth(e) => {
                write!(f, "Unsupported color depth, expected 24, got {e}")
            }
            Error::UnsupportedCompression(e) => {
                write!(f, "Unsupported compression, expected 0, got {e}")
            }
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Debug)]
pub struct RgbImage {
    pub pixels: Vec<Rgb>,
    pub width: u32,
}

impl RgbImage {
    pub fn new(pixels: Vec<Rgb>, width: u32) -> Self {
        Self { pixels, width }
    }

    pub fn save_bmp(&self, file_path: &str) -> Result<(), Error> {
        let width = self.width;
        let len = self.pixels.len() as u32;

        let header_size = 14;
        let info_header_size = 40;
        let padding = (4 - ((width * 3) % 4)) % 4;
        let height = len / width;
        let file_size = header_size + info_header_size + height * padding + len * 3;
        let data_offset = header_size + info_header_size;
        let mut buff = Vec::with_capacity(file_size as usize);

        // Header
        write_u8(&mut buff, 'B' as u8);
        write_u8(&mut buff, 'M' as u8);
        write_u32(&mut buff, file_size);
        write_u32(&mut buff, 0); // unused
        write_u32(&mut buff, data_offset);

        //InfoHeader
        write_u32(&mut buff, info_header_size);
        write_u32(&mut buff, width);
        write_u32(&mut buff, height);
        write_u16(&mut buff, 1); // planes
        write_u16(&mut buff, 24); // bits per pixel
        write_u32(&mut buff, 0); // compression  0=no compression
        write_u32(&mut buff, 0); // compressed size, 0=no compression
        write_u32(&mut buff, width); // horizontal pixel/meter
        write_u32(&mut buff, height); // vertical pixel/meter
        write_u32(&mut buff, 16777216); // used colors, 2^24
        write_u32(&mut buff, 0); // important colors, 0=all

        // Pixels
        for i in 0..height {
            let i = height - i - 1;
            for j in 0..width {
                let index = (i * width + j) as usize;
                write_u8(&mut buff, self.pixels[index].b);
                write_u8(&mut buff, self.pixels[index].g);
                write_u8(&mut buff, self.pixels[index].r);
            }

            for _ in 0..padding {
                write_u8(&mut buff, 0);
            }
        }

        File::create(file_path)?.write_all(buff.as_mut_slice())?;

        Ok(())
    }

    pub fn load_bmp(file_path: &str) -> Result<Self, Error> {
        let mut buff = vec![];
        File::open(file_path)?.read_to_end(&mut buff)?;

        let src = read_header(&buff)?;
        let (src, width, height) = read_info_header(src)?;
        let (_, pixels) = read_pixels(src, width, height)?;

        Ok(Self { pixels, width })
    }
}

fn read_header(src: &[u8]) -> Result<&[u8], Error> {
    dbg!(&src[..14]);
    let (src, letter_b) = read_u8(src)?;
    let (src, letter_m) = read_u8(src)?;
    let (src, _file_size) = read_u32(src)?;
    let (src, _reserved) = read_u32(src)?;
    let (src, _data_offset) = read_u32(src)?;

    if letter_b as char != 'B' {
        return Err(Error::InvalidSignature);
    }
    if letter_m as char != 'M' {
        return Err(Error::InvalidSignature);
    }

    Ok(src)
}

fn read_info_header(src: &[u8]) -> Result<(&[u8], u32, u32), Error> {
    dbg!(&src[..40]);
    let (src, header_size) = read_u32(src)?;
    let (src, width) = read_u32(src)?;
    let (src, height) = read_u32(src)?;
    let (src, planes) = read_u16(src)?;
    let (src, bits_per_pixel) = read_u16(src)?;
    let (src, compression) = read_u32(src)?;
    let (src, _file_size) = read_u32(src)?;
    let (src, _horiz_pixel_per_meter) = read_u32(src)?;
    let (src, _vert_pixel_per_meter) = read_u32(src)?;
    let (src, _colors_used) = read_u32(src)?;
    let (src, _important_colors) = read_u32(src)?;

    if header_size != 40 {
        return Err(Error::InvalidHeaderSize(header_size));
    }
    if planes != 1 {
        return Err(Error::UnsupportedPlaneCount(planes));
    }
    if bits_per_pixel != 24 {
        return Err(Error::UnsupportedColorDepth(bits_per_pixel));
    }
    if compression != 0 {
        return Err(Error::UnsupportedCompression(compression));
    }

    Ok((src, width, height))
}

fn read_pixels(mut src: &[u8], width: u32, height: u32) -> Result<(&[u8], Vec<Rgb>), Error> {
    let padding = (4 - ((width * 3) % 4)) % 4;

    let mut pixels = Vec::with_capacity((width * height) as usize);
    pixels.resize((width * height) as usize, Rgb::default());

    for i in 0..height {
        let i = height - i - 1;
        for j in 0..width {
            let index = (i * width + j) as usize;
            let (next, b) = read_u8(src)?;
            let (next, g) = read_u8(next)?;
            let (next, r) = read_u8(next)?;
            pixels[index] = Rgb::new(r, g, b);
            src = next;
        }

        for _ in 0..padding {
            let (next, _) = read_u8(src)?;
            src = next;
        }
    }

    Ok((src, pixels))
}

fn write_u32(buff: &mut Vec<u8>, val: u32) {
    for b in val.to_le_bytes() {
        buff.push(b);
    }
}

fn write_u16(buff: &mut Vec<u8>, val: u16) {
    for b in val.to_le_bytes() {
        buff.push(b);
    }
}

fn write_u8(buff: &mut Vec<u8>, val: u8) {
    buff.push(val);
}

fn read_u32(mut src: &[u8]) -> Result<(&[u8], u32), Error> {
    let mut bytes = [0; 4];
    src.read_exact(&mut bytes)?;
    let val = bytes[0] as u32
        | ((bytes[1] as u32) << 8)
        | ((bytes[2] as u32) << 16)
        | ((bytes[3] as u32) << 24);

    Ok((src, val))
}

fn read_u16(mut src: &[u8]) -> Result<(&[u8], u16), Error> {
    let mut bytes = [0; 2];
    src.read_exact(&mut bytes)?;

    Ok((src, bytes[0] as u16 | ((bytes[1] as u16) << 8)))
}

fn read_u8(mut src: &[u8]) -> Result<(&[u8], u8), Error> {
    let mut bytes = [0; 1];
    src.read_exact(&mut bytes)?;

    Ok((src, bytes[0]))
}
