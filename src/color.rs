use std::{
    fs::File,
    io::{BufWriter, Write},
    ops,
    path::PathBuf,
};

use image::RgbImage;

#[derive(Debug, Clone, Copy)]
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

    pub fn distance(&self, other: Color) -> f64 {
        let dr = self.r - other.r;
        let dg = self.g - other.g;
        let db = self.b - other.b;

        (dr * dr + dg * dg + db * db).sqrt()
    }

    pub fn distance_squared(&self, other: Color) -> f64 {
        let dr = self.r - other.r;
        let dg = self.g - other.g;
        let db = self.b - other.b;

        dr * dr + dg * dg + db * db
    }

    pub fn luma(&self) -> f64 {
        0.299 * self.r + 0.587 * self.g + 0.114 * self.b
    }
}

impl Default for Color {
    fn default() -> Self {
        Self { r: 0.0, g: 0.0, b: 0.0 }
    }
}

impl ops::AddAssign<Color> for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}

impl ops::Mul<f64> for Color {
    type Output = Color;

    fn mul(self, rhs: f64) -> Self::Output {
        Color {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

impl ops::Div<f64> for Color {
    type Output = Color;

    fn div(self, rhs: f64) -> Self::Output {
        Color {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
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

pub type Palette = Vec<Color>;

pub fn save_palette(palette: &Palette, filename: PathBuf) -> anyhow::Result<()> {
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    for color in palette {
        writer.write_all(&color.r.to_le_bytes())?;
        writer.write_all(&color.g.to_le_bytes())?;
        writer.write_all(&color.b.to_le_bytes())?;
    }

    writer.flush()?;

    Ok(())
}
