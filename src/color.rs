use image::RgbImage;

pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub fn new(r: i32, g: i32, b: i32) -> Color {
        Color {
            r: (r as f64) / 255.0,
            g: (g as f64) / 255.0,
            b: (b as f64) / 255.0,
        }
    }
}

pub struct ColorCube(pub Vec<Vec<Vec<u64>>>);

impl ColorCube {
    pub fn new() -> ColorCube {
        ColorCube(vec![vec![vec![0u64; 256]; 256]; 256])
    }

    pub fn update(&mut self, image: &RgbImage) {
        for color in image.pixels() {
            self.0[color[0] as usize][color[1] as usize][color[2] as usize] += 1;
        }
    }
}
