use clap::Parser;
use console::style;
use image::ImageReader;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};
use unicode_ellipsis::truncate_str_leading;

use crate::{color::ColorCube, palgen::PalGen};

mod color;
mod palgen;

#[derive(Parser, Debug)]
struct Args {
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

fn count_digits(val: u64) -> u32 {
    val.checked_ilog10().unwrap_or(0) + 1
}

pub struct CalcStatus {
    step_bar: ProgressBar,
    attempt_bar: ProgressBar,
    moved_spinner: ProgressBar,
    distance_spinner: ProgressBar,

    max_steps: u64,
    time_step: SystemTime,
}

impl CalcStatus {
    const MAX_UPDATE_PERIOD: Duration = Duration::from_millis(500);

    pub fn new(multi: &MultiProgress, max_attempts: u64, max_steps: u64) -> CalcStatus {
        let moved_spinner = multi.add(ProgressBar::new_spinner());
        let distance_spinner = multi.add(ProgressBar::new_spinner());
        let step_bar = multi.add(ProgressBar::new(max_steps));
        let attempt_bar = multi.add(ProgressBar::new(max_attempts * max_steps));

        moved_spinner.set_style(ProgressStyle::with_template("Colors changed: {msg:.yellow}").unwrap());
        distance_spinner.set_style(ProgressStyle::with_template("Total error:    {msg:.yellow}").unwrap());

        const LABELS_LENGTH_DIFF: u32 = 3;

        let steps_length = count_digits(max_steps);
        let att_length = count_digits(max_attempts);
        let (steps_padding, att_padding) = {
            let steps_label_length = steps_length * 2 + 1;
            let att_label_length = att_length * 2 + 1;
            if steps_label_length - att_label_length > LABELS_LENGTH_DIFF {
                (0, steps_label_length - att_label_length - LABELS_LENGTH_DIFF)
            } else if steps_label_length - att_label_length < LABELS_LENGTH_DIFF {
                (att_label_length + LABELS_LENGTH_DIFF - steps_label_length, 0)
            } else {
                (0, 0)
            }
        };

        let step_template = format!(
            "\nStep {}{{pos:>{}.yellow}}{}{{len:.yellow}} {}{{bar:40.green}}{} Elapsed:   {{elapsed_precise:.yellow}}",
            " ".repeat(steps_padding as usize),
            steps_length,
            style("/").yellow(),
            style("▕").green(),
            style("▏").green()
        );

        step_bar.set_style(
            ProgressStyle::with_template(&step_template)
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );

        let attempt_template = format!(
            "\nAttempt {}{{msg:>{}.yellow}}{} {}{{bar:40.green}}{} Remaining: {{eta_precise:.yellow}}",
            " ".repeat(att_padding as usize),
            att_length,
            style(format!("/{}", max_attempts)).yellow(),
            style("▕").green(),
            style("▏").green()
        );

        attempt_bar.set_style(
            ProgressStyle::with_template(&attempt_template)
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );

        CalcStatus {
            step_bar,
            attempt_bar,
            moved_spinner,
            distance_spinner,

            max_steps,
            time_step: SystemTime::UNIX_EPOCH,
        }
    }

    pub fn new_attempt(&mut self, attempt: u64) {
        self.step_bar.set_position(0);
        self.attempt_bar.set_position(attempt * self.max_steps);
        self.attempt_bar.set_message((attempt + 1).to_string());

        self.time_step = SystemTime::UNIX_EPOCH;
    }

    pub fn step(&mut self, moved: u64, distance: f64) {
        self.step_bar.inc(1);
        self.attempt_bar.inc(1);

        if self.time_step.elapsed().unwrap() > CalcStatus::MAX_UPDATE_PERIOD {
            self.moved_spinner.set_message(moved.to_string());
            self.distance_spinner.set_message(distance.to_string());
            self.time_step = SystemTime::now();
        }
    }

    pub fn finish(&self) {
        self.moved_spinner.finish_and_clear();
        self.distance_spinner.finish_and_clear();
        self.step_bar.finish_and_clear();
        self.attempt_bar.finish_and_clear();
    }
}

fn main() -> anyhow::Result<()> {
    println!("= IMAGEPAL v0.1 =\n");

    let args = Args::parse_from(wild::args());

    let filecount_size = count_digits(args.files.len() as u64);
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

    let mut calc_status = CalcStatus::new(&m, 5, 5000);

    palgen.run(&mut calc_status, 5, 5000)?;

    calc_status.finish();
    title_spinner.finish_and_clear();

    println!("Done!");

    Ok(())
}
