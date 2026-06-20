use std::{
    path::Path,
    time::{Duration, SystemTime},
};

use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use unicode_ellipsis::truncate_str_leading;

pub fn count_digits(val: u64) -> u32 {
    val.checked_ilog10().unwrap_or(0) + 1
}

pub struct LoadStatus {
    load_bar: ProgressBar,

    max_filename_length: u32,
}

impl LoadStatus {
    pub fn new(multi: &MultiProgress, filecount: usize) -> LoadStatus {
        let filecount_length = count_digits(filecount as u64);
        let max_filename_length = 40 + 3 + filecount_length * 2;

        let load_bar = multi.add(ProgressBar::new(filecount as u64));
        let load_template = format!(
            "{{msg}}\n{{pos:>{}.yellow}}{}{{len:.yellow}} {}{{bar:40.green}}{} ({{eta:.yellow}})",
            filecount_length,
            style("/").yellow(),
            style("▕").green(),
            style("▏").green()
        );
        load_bar.set_style(
            ProgressStyle::with_template(&load_template)
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );

        LoadStatus {
            load_bar,
            max_filename_length,
        }
    }

    pub fn step_before(&self, filename: &Path) {
        let namestr = filename.to_str().unwrap_or_default();
        let namestr = truncate_str_leading(namestr, self.max_filename_length as usize);
        self.load_bar.set_message(namestr.into_owned());
    }

    pub fn step_after(&self) {
        self.load_bar.inc(1);
    }

    pub fn finish(&self) {
        self.load_bar.finish_and_clear();
    }
}

pub struct CalcStatus {
    step_bar: ProgressBar,
    attempt_bar: ProgressBar,
    moved_spinner: ProgressBar,
    distance_spinner: ProgressBar,

    time_step: SystemTime,
    total_steps: u64,
}

impl CalcStatus {
    const MAX_UPDATE_PERIOD: Duration = Duration::from_millis(500);

    pub fn new(multi: &MultiProgress, max_attempts: u64, max_steps: u64) -> CalcStatus {
        let moved_spinner = multi.add(ProgressBar::new_spinner());
        let distance_spinner = multi.add(ProgressBar::new_spinner());
        let step_bar = multi.add(ProgressBar::new(max_steps));
        let total_steps = max_attempts * max_steps;
        let attempt_bar = multi.add(ProgressBar::new(total_steps));

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

            time_step: SystemTime::UNIX_EPOCH,
            total_steps,
        }
    }

    pub fn new_attempt(&mut self, attempt: u64) {
        self.step_bar.set_position(0);
        //self.attempt_bar.set_position(attempt * self.max_steps);
        self.attempt_bar.set_message((attempt + 1).to_string());

        self.time_step = SystemTime::UNIX_EPOCH;
    }

    pub fn step(&mut self, moved: u64, distance: f64, force: bool) {
        self.step_bar.inc(1);
        self.attempt_bar.inc(1);

        if force || self.time_step.elapsed().unwrap() > CalcStatus::MAX_UPDATE_PERIOD {
            self.moved_spinner.set_message(moved.to_string());
            self.distance_spinner.set_message(distance.to_string());
            self.time_step = SystemTime::now();
        }
    }

    pub fn reduce_total(&mut self, diff: u64) {
        self.total_steps -= diff;
        self.attempt_bar.set_length(self.total_steps);
    }

    pub fn finish(&self) {
        self.moved_spinner.finish_and_clear();
        self.distance_spinner.finish_and_clear();
        self.step_bar.finish_and_clear();
        self.attempt_bar.finish_and_clear();
    }
}
