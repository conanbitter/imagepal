use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Read, Write},
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

pub struct Palette(pub Vec<Color>);

fn channel_to_byte(channel: f64) -> u8 {
    (channel.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

impl Palette {
    pub fn flom_file(filename: PathBuf) -> anyhow::Result<Palette> {
        let file = File::open(filename)?;
        let mut reader = BufReader::new(file);

        let mut buffer = [0u8; 24];
        let mut result = Palette(vec![]);

        loop {
            match reader.read_exact(&mut buffer) {
                Ok(_) => {
                    let color = Color {
                        r: f64::from_le_bytes(buffer[0..8].try_into().unwrap()),
                        g: f64::from_le_bytes(buffer[8..16].try_into().unwrap()),
                        b: f64::from_le_bytes(buffer[16..24].try_into().unwrap()),
                    };

                    result.0.push(color);
                }
                Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(anyhow::Error::from(e)),
            }
        }

        Ok(result)
    }

    pub fn save(&self, filename: PathBuf) -> anyhow::Result<()> {
        let file = File::create(filename)?;
        let mut writer = BufWriter::new(file);

        for color in &self.0 {
            writer.write_all(&color.r.to_le_bytes())?;
            writer.write_all(&color.g.to_le_bytes())?;
            writer.write_all(&color.b.to_le_bytes())?;
        }

        writer.flush()?;

        Ok(())
    }

    pub fn export(&self, filename: PathBuf) -> anyhow::Result<()> {
        let file = File::create(filename)?;
        let mut writer = BufWriter::new(file);

        let mut buffer = [0u8; 4];

        for color in &self.0 {
            buffer[1] = channel_to_byte(color.r);
            buffer[2] = channel_to_byte(color.g);
            buffer[3] = channel_to_byte(color.b);
            writer.write_all(&buffer)?;
        }

        writer.flush()?;

        Ok(())
    }

    pub fn get_png_palette(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.0.len() * 3);

        for color in &self.0 {
            result.push(channel_to_byte(color.r));
            result.push(channel_to_byte(color.g));
            result.push(channel_to_byte(color.b));
        }

        result
    }

    pub fn find_index(&self, color: Color) -> i32 {
        let mut best_index = 0;
        let mut best_difference = f64::MAX;
        for (i, palcol) in self.0.iter().enumerate() {
            let difference = palcol.distance(color);
            if difference < best_difference {
                best_difference = difference;
                best_index = i;
            }
        }
        best_index as i32
    }
}
