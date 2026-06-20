use clap::Parser;
use image::ImageReader;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{path::PathBuf, str::FromStr, time::Duration};

use crate::{
    color::{ColorCube, save_palette},
    palgen::PalGen,
    report::{CalcStatus, LoadStatus},
};

mod color;
mod palgen;
mod report;

#[derive(Parser, Debug)]
struct Args {
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    println!("= IMAGEPAL v0.1 =\n");

    let args = Args::parse_from(wild::args());

    let m = MultiProgress::new();

    let title_spinner = m.add(ProgressBar::new_spinner());
    title_spinner.set_style(ProgressStyle::with_template("{spinner:.green} {msg}...\n\n").unwrap());

    // LOAD IMAGES

    let load_status = LoadStatus::new(&m, args.files.len());

    title_spinner.set_message("Loading images");
    title_spinner.enable_steady_tick(Duration::from_millis(100));

    let mut cube = ColorCube::new();

    for filename in args.files {
        load_status.step_before(&filename);

        let img = ImageReader::open(filename)?.decode()?.to_rgb8();
        cube.update(&img);

        load_status.step_after();
    }
    load_status.finish();

    /*let mut colors = 0;
    for r in 0..256 {
        for g in 0..256 {
            for b in 0..256 {
                if cube.0[r][g][b] > 0 {
                    colors += 1;
                }
            }
        }
    }*/

    // CALCULATE PALETTE

    title_spinner.set_message("Calculating colors");

    let mut palgen = PalGen::new(256, cube)?;

    let mut calc_status = CalcStatus::new(&m, 5, 2000);

    let result_palette = palgen.run(&mut calc_status, 5, 2000)?;

    calc_status.finish();
    title_spinner.finish_and_clear();

    save_palette(&result_palette, PathBuf::from_str("result.ipal")?)?;
    println!("Done!");

    Ok(())
}
