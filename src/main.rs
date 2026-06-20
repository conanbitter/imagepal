use clap::{Args, Parser, Subcommand};
use console::style;
use image::ImageReader;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{path::PathBuf, time::Duration};

use crate::{
    color::{ColorCube, save_palette},
    palgen::PalGen,
    report::{CalcStatus, LoadStatus},
};

mod color;
mod palgen;
mod report;

#[derive(Parser, Debug)]
#[command(
    name = "imagepal",
    version = "0.1",
    about = "A palette generator for groups of images"
)]
struct AppArgs {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Generate(GenArgs),
}

#[derive(Args, Debug)]
struct GenArgs {
    #[arg(required = true)]
    files: Vec<PathBuf>,

    #[arg(short, long, required = true, help = "name for the palette file")]
    output: PathBuf,

    #[arg(short, long, default_value_t = 256, help = "target number of colors")]
    colors: u32,

    #[arg(short, long, default_value_t = 5, help = "number of attempts")]
    attempts: u64,

    #[arg(short, long, default_value_t = 1000, help = "maximum number of steps")]
    steps: u64,
}

fn command_generate(args: GenArgs) -> anyhow::Result<()> {
    println!("{} Generate palette\n", style("│").green());

    let multi = MultiProgress::new();

    let title_spinner = multi.add(ProgressBar::new_spinner());
    title_spinner.set_style(ProgressStyle::with_template("{spinner:.green} {msg}...\n\n").unwrap());
    title_spinner.enable_steady_tick(Duration::from_millis(100));

    // LOAD IMAGES

    let load_status = LoadStatus::new(&multi, args.files.len());

    title_spinner.set_message("Loading images");

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

    let mut palgen = PalGen::new(args.colors, cube)?;

    let mut calc_status = CalcStatus::new(&multi, args.attempts, args.steps);

    let result_palette = palgen.run(&mut calc_status, args.attempts, args.steps)?;

    calc_status.finish();

    save_palette(&result_palette, args.output.clone())?;

    title_spinner.finish_and_clear();
    println!("Done!");
    println!("Palette saved to \"{}\"", args.output.to_string_lossy());

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = AppArgs::parse_from(wild::args());

    println!(
        "{}\n{} {}",
        style("╭──────────").green(),
        style("│").green(),
        style("IMAGEPAL").bold()
    );

    match args.command {
        Commands::Generate(gen_args) => command_generate(gen_args)?,
    }

    Ok(())
}
