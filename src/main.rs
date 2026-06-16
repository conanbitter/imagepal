use std::{thread, time::Duration};

use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rand::RngExt;

fn main() {
    let m = MultiProgress::new();

    let op_bar = m.add(ProgressBar::new(200));
    let total_bar = m.add(ProgressBar::new(5 * 300));

    let total_template = format!(
        "Step {{pos:>3.yellow}}{}{{len:3.yellow}} {}{{wide_bar:.green}}{}Elapsed:   {{elapsed_precise:.yellow}}",
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
        "\nAttempt  {{msg:.yellow}}{} {}{{wide_bar:.green}}{}Remaining: {{eta_precise:.yellow}}",
        style("/5").yellow(),
        style("▕").green(),
        style("▏").green()
    );

    total_bar.set_style(
        ProgressStyle::with_template(&total_template)
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  "),
    );

    for i in 0..5 {
        op_bar.set_position(0);
        total_bar.set_position(i * 300);
        total_bar.set_message((i + 1).to_string());
        for _ in 0..200 {
            thread::sleep(Duration::from_millis(50));
            op_bar.inc(1);
            total_bar.inc(1);
        }
        op_bar.finish();
    }
    total_bar.finish();
}
