use std::{thread, time::Duration};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rand::RngExt;

fn main() {
    let m = MultiProgress::new();

    let op_bar = m.add(ProgressBar::new(200));
    let spacer = m.add(ProgressBar::hidden());
    let total_bar = m.add(ProgressBar::new(5 * 300));

    spacer.set_style(ProgressStyle::with_template(" ").unwrap());

    op_bar.set_style(
        ProgressStyle::with_template("Step {pos:>4}/{len:4} {bar:.green.on_white} Elapsed:   {elapsed_precise}")
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  "),
    );

    total_bar.set_style(
        ProgressStyle::with_template("\nAttempt {msg}/5    {bar:.green.on_white} Remaining: {eta_precise}")
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
