use clap::{Args, Parser, Subcommand, ValueEnum};
use console::style;
use image::ImageReader;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    time::Duration,
};

use crate::{
    color::{Color, ColorCube, Palette},
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
    Export(ExpArgs),
    Convert(ConvArgs),
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

#[derive(Args, Debug)]
struct ExpArgs {
    #[arg(required = true)]
    file: PathBuf,

    #[arg(short, long, required = true, help = "name for the exported palette file")]
    output: PathBuf,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum ConvFormat {
    Png,
    Raw,
}

#[derive(Args, Debug)]
struct ConvArgs {
    #[arg(required = true)]
    files: Vec<PathBuf>,

    #[arg(short, long, required = true, help = "folder for converted files")]
    output: PathBuf,

    #[arg(short, long, required = true, help = "palette file")]
    palette: PathBuf,

    #[arg(short, long, value_enum, default_value_t = ConvFormat::Raw,  help = "output format")]
    format: ConvFormat,
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

    result_palette.save(args.output.clone())?;

    title_spinner.finish_and_clear();
    println!("Done!");
    println!("Palette saved to \"{}\"", args.output.to_string_lossy());

    Ok(())
}

fn command_export(args: ExpArgs) -> anyhow::Result<()> {
    println!("{} Export palette\n", style("│").green());

    let palette = Palette::flom_file(args.file)?;
    palette.export(args.output.clone())?;

    println!("Palette exported to \"{}\"", args.output.to_string_lossy());

    Ok(())
}

fn command_convert(args: ConvArgs) -> anyhow::Result<()> {
    println!("{} Convert images\n", style("│").green());

    let fpal = Palette::flom_file(args.palette)?;

    match args.format {
        ConvFormat::Png => {
            let ppal = fpal.get_png_palette();

            for filename in args.files {
                let stem = filename.file_stem().expect("Invalid file name");
                let mut new_name = args.output.join(stem);
                new_name.set_extension("png");

                println!("Saving {}", new_name.display());

                let file = File::create(new_name)?;
                let writer = BufWriter::new(file);

                let img = ImageReader::open(filename)?.decode()?.to_rgb8();
                let mut encoder = png::Encoder::new(writer, img.width(), img.height());
                encoder.set_color(png::ColorType::Indexed);
                encoder.set_depth(png::BitDepth::Eight);
                encoder.set_palette(&ppal);

                let mut indices = Vec::with_capacity((img.width() * img.height()) as usize);

                for color in img.pixels() {
                    let fcolor = Color::new(color[0] as i32, color[1] as i32, color[2] as i32);
                    let index = fpal.find_index(fcolor) as u8;
                    indices.push(index);
                }

                let mut writer = encoder.write_header()?;
                writer.write_image_data(&indices)?;
                writer.finish()?;
            }
        }
        ConvFormat::Raw => {
            for filename in args.files {
                let stem = filename.file_stem().expect("Invalid file name");
                let mut new_name = args.output.join(stem);
                new_name.set_extension("raw");

                println!("Saving {}", new_name.display());

                let file = File::create(new_name)?;
                let mut writer = BufWriter::new(file);

                let img = ImageReader::open(filename)?.decode()?.to_rgb8();
                let width = img.width();
                let height = img.height();
                writer.write_all(&width.to_le_bytes())?;
                writer.write_all(&height.to_le_bytes())?;

                let mut buffer = [0u8; 1];

                for color in img.pixels() {
                    let fcolor = Color::new(color[0] as i32, color[1] as i32, color[2] as i32);
                    buffer[0] = fpal.find_index(fcolor) as u8;
                    writer.write_all(&buffer)?;
                }

                writer.flush()?;
            }
        }
    }

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
        Commands::Export(exp_args) => command_export(exp_args)?,
        Commands::Convert(conv_args) => command_convert(conv_args)?,
    }

    Ok(())
}
