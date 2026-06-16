use std::{
    thread,
    time::{Duration, SystemTime},
};

use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rand::RngExt;

fn main() {
    println!("IMAGEPAL v0.1\n");

    let m = MultiProgress::new();

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

    for i in 0..5 {
        op_bar.set_position(0);
        total_bar.set_position(i * 300);
        total_bar.set_message((i + 1).to_string());
        let mut now = SystemTime::UNIX_EPOCH;
        for j in 0..200 {
            thread::sleep(Duration::from_millis(50));
            op_bar.inc(1);
            total_bar.inc(1);
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

    println!("Done!")
}
