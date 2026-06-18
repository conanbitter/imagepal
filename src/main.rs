use clap::Parser;
use console::style;
use image::ImageReader;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
    path::PathBuf,
    thread,
    time::{Duration, SystemTime},
};
use unicode_ellipsis::truncate_str_leading;

use crate::color::ColorCube;

mod color;

#[derive(Parser, Debug)]
struct Args {
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

fn count_digits(val: usize) -> u32 {
    val.checked_ilog10().unwrap_or(0) + 1
}

fn main() -> anyhow::Result<()> {
    println!("= IMAGEPAL v0.1 =\n");

    let args = Args::parse_from(wild::args());

    let filecount_size = count_digits(args.files.len());
    let max_filename_size = 40 + 3 + filecount_size * 2;

    let m = MultiProgress::new();

    let title_spinner = m.add(ProgressBar::new_spinner());
    title_spinner.set_style(ProgressStyle::with_template("{spinner:.green} {msg}...\n\n").unwrap());

    // LOAD IMAGES

    let load_bar = m.add(ProgressBar::new(args.files.len() as u64));
    let load_template = format!(
        "{{msg}}\n{{pos:>{}.yellow}}{}{{len:.yellow}} {}{{bar:40.green}}{} ({{eta:.yellow}})",
        filecount_size,
        style("/").yellow(),
        style("▕").green(),
        style("▏").green()
    );
    load_bar.set_style(
        ProgressStyle::with_template(&load_template)
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  "),
    );

    title_spinner.set_message("Loading images");
    title_spinner.enable_steady_tick(Duration::from_millis(100));

    let mut cube = ColorCube::new();

    for filename in args.files {
        let namestr = filename.to_str().unwrap_or_default();
        let namestr = truncate_str_leading(namestr, max_filename_size as usize);
        load_bar.set_message(namestr.into_owned());

        let img = ImageReader::open(filename)?.decode()?.to_rgb8();
        cube.update(&img);

        load_bar.inc(1);
    }
    load_bar.finish_and_clear();
    title_spinner.finish_and_clear();

    let mut colors = 0;
    for r in 0..256 {
        for g in 0..256 {
            for b in 0..256 {
                if cube.0[r][g][b] > 0 {
                    colors += 1;
                }
            }
        }
    }
    println!("Unique colors: {}", colors);
    Ok(())

    // CALCULATE PALETTE
    /*
        let moved_spin = m.add(ProgressBar::new_spinner());
        let dist_spin = m.add(ProgressBar::new_spinner());
        let op_bar = m.add(ProgressBar::new(200));
        let total_bar = m.add(ProgressBar::new(5 * 300));

        moved_spin.set_style(ProgressStyle::with_template("Colors changed: {msg:.yellow}").unwrap());
        dist_spin.set_style(ProgressStyle::with_template("Total error:    {msg:.yellow}").unwrap());

        let total_template = format!(
            "\nStep {{pos:>3.yellow}}{}{{len:3.yellow}} {}{{bar:40.green}}{} Elapsed:   {{elapsed_precise:.yellow}}",
            style("/").yellow(),
            style("▕").green(),
            style("▏").green()
        );

        op_bar.set_style(
            ProgressStyle::with_template(&total_template)
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );

        let total_template = format!(
            "\nAttempt  {{msg:.yellow}}{} {}{{bar:40.green}}{} Remaining: {{eta_precise:.yellow}}",
            style("/5").yellow(),
            style("▕").green(),
            style("▏").green()
        );

        total_bar.set_style(
            ProgressStyle::with_template(&total_template)
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );

        const MAX_DURATION: Duration = Duration::from_millis(500);

        title_spinner.set_message("Calculating colors");

        for i in 0..5 {
            op_bar.set_position(0);
            total_bar.set_position(i * 300);
            total_bar.set_message((i + 1).to_string());
            let mut now = SystemTime::UNIX_EPOCH;
            for j in 0..200 {
                thread::sleep(Duration::from_millis(5));
                op_bar.inc(1);
                total_bar.inc(1);
                title_spinner.tick();
                if now.elapsed().unwrap() > MAX_DURATION {
                    moved_spin.set_message((j * 10).to_string());
                    dist_spin.set_message((200.0 / (j as f64 + 0.0001)).to_string());
                    now = SystemTime::now();
                }
            }
        }
        moved_spin.finish_and_clear();
        dist_spin.finish_and_clear();
        op_bar.finish_and_clear();
        total_bar.finish_and_clear();

        title_spinner.finish_and_clear();

        println!("Done!");
    */
}
