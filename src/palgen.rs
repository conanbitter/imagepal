use std::sync::Mutex;

use anyhow::Result;
use itertools::izip;
use rand::RngExt;
use rayon::prelude::*;

use crate::{
    CalcStatus,
    color::{Color, ColorCube, Palette},
};

struct ColorPoint {
    color: Color,
    segment: usize,
    count: u64,
    distance: f64,
}

impl ColorPoint {
    fn distance_squared(&mut self, c: Color) -> f64 {
        let dist = self.color.distance_squared(c);
        if dist < self.distance {
            self.distance = dist;
            return dist;
        }
        self.distance
    }
}

pub struct PalGen {
    points: Vec<ColorPoint>,
    centroids: Vec<Color>,
    new_centroids: Vec<Color>,
    point_counts: Vec<u64>,

    colors: u32,
    point_count: u64,

    total_distance: f64,
    points_changed: u64,

    best_error: f64,
    best_palette: Palette,
}

impl PalGen {
    pub fn new(color_count: u32, colors: ColorCube) -> Result<PalGen> {
        let mut total_colors = {
            if color_count > 256 {
                256u64
            } else if color_count < 1 {
                1u64
            } else {
                color_count as u64
            }
        };

        let mut points = vec![];

        for r in 0..256 {
            for g in 0..256 {
                for b in 0..256 {
                    if colors.0[r][g][b] > 0 {
                        let col = Color::new(r as i32, g as i32, b as i32);

                        points.push(ColorPoint {
                            color: col,
                            segment: 0,
                            count: colors.0[r][g][b],
                            distance: f64::MAX,
                        })
                    }
                }
            }
        }

        let unique_colors = points.len() as u64;
        if total_colors > unique_colors {
            total_colors = unique_colors;
        }

        Ok(PalGen {
            colors: total_colors as u32,
            points,
            centroids: vec![Color::default(); total_colors as usize],
            new_centroids: vec![Color::default(); total_colors as usize],
            point_counts: vec![0u64; total_colors as usize],
            point_count: unique_colors,
            total_distance: 0.0,
            points_changed: 0,
            best_error: 0.0,
            best_palette: vec![Color::default(); total_colors as usize],
        })
    }

    fn init_centroids(&mut self) {
        let mut rng = rand::rng();
        self.points.swap(0, rng.random_range(0..self.point_count) as usize);
        for cent_ind in 1..(self.colors - 1) as usize {
            let mut sum = 0.0;
            let cent_color = self.points[cent_ind - 1].color;
            for i in cent_ind - 1..self.point_count as usize {
                sum += self.points[i].distance_squared(cent_color);
            }

            let rnd = sum * rng.random::<f64>();
            sum = 0.0;
            let mut next = self.point_count as usize - 1;
            for i in cent_ind + 1..self.point_count as usize {
                sum += self.points[i].distance;
                if sum > rnd {
                    next = i;
                    break;
                }
            }
            self.points.swap(cent_ind, next);
        }
        for i in 0..self.colors {
            self.centroids[i as usize] = self.points[i as usize].color;
        }
    }

    fn calc_centroids(&mut self) {
        self.new_centroids.fill(Color::default());
        self.point_counts.fill(0);

        for point in &self.points {
            self.point_counts[point.segment] += point.count;
            self.new_centroids[point.segment] += point.color * (point.count as f64);
        }

        self.total_distance = 0.0;

        for (color, count, sum) in izip!(&mut self.centroids, &self.point_counts, &self.new_centroids) {
            if *count == 0 {
                *color = Color::default();
            } else {
                let new_color = *sum / *count as f64;
                self.total_distance += color.distance(new_color);
                *color = new_color;
            }
        }
    }

    fn calc_segments(&mut self) {
        let points_changed = Mutex::new(0u64);

        self.points.par_iter_mut().for_each(|point| {
            let old_seg = point.segment;
            let mut new_seg = old_seg;
            let mut min_dist = point.color.distance_squared(self.centroids[old_seg]);

            for (i, c) in self.centroids.iter().enumerate() {
                let dist = point.color.distance_squared(*c);
                if min_dist > dist {
                    min_dist = dist;
                    new_seg = i;
                }
            }

            if new_seg != old_seg {
                point.segment = new_seg;
                let mut changed = points_changed.lock().unwrap();
                *changed += 1;
            }
        });

        self.points_changed = *points_changed.lock().unwrap();
    }

    fn calc_error(&self) -> f64 {
        self.points.iter().fold(0.0, |acc, p| {
            acc + p.color.distance_squared(self.centroids[p.segment]) * (p.count as f64)
        })
    }

    pub fn run(&mut self, status: &mut CalcStatus, max_attempts: u64, max_steps: u64) -> Result<Palette> {
        for attempt in 0..max_attempts {
            self.init_centroids();
            status.new_attempt(attempt);

            for step in 0..max_steps {
                self.calc_segments();

                if self.points_changed == 0 {
                    status.reduce_total(max_steps - step);
                    status.step(self.points_changed, self.total_distance, true);
                    break;
                }

                self.calc_centroids();

                status.step(self.points_changed, self.total_distance, false);
            }

            self.calc_segments();

            let error = self.calc_error();
            if attempt == 0 || error < self.best_error {
                self.best_error = error;
                self.best_palette.copy_from_slice(&self.centroids);
            }
        }

        self.best_palette.sort_by(|a, b| a.luma().total_cmp(&b.luma()));
        Ok(self.best_palette.clone())
    }
}
