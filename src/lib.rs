mod bmp;
pub use bmp::*;

#[cfg(test)]
mod tests {
    use crate::{Rgb, RgbImage};

    #[test]
    fn save_bmp() {
        let width = 30;
        let height = 30;

        let pixels = (0..height)
            .flat_map(|y| {
                (0..width)
                    .map(|x| {
                        let color =
                            (((((x + y) as f32) * 0.1).sin() + 1.0).clamp(0.0, 2.0) * 128.0) as u8;
                        Rgb::new(color, color, color)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let res = RgbImage::new(pixels, width).save_bmp("hello.bmp");
        assert!(res.is_ok(), "Error: {}", res.unwrap_err())
    }

    #[test]
    fn test_load() {
        let res = RgbImage::load_bmp("hello.bmp");
        assert!(res.is_ok(), "Error: {}", res.unwrap_err());

        let mut pic = res.unwrap();
        for p in pic.pixels.as_mut_slice() {
            p.r = 255 - p.r;
            p.g = 255 - p.g;
            p.b = 255 - p.b;
        }

        let res = pic.save_bmp("goodbye.bmp");
        assert!(res.is_ok(), "Error: {}", res.unwrap_err())
    }
}
