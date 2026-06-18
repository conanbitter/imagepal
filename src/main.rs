use std::{
    thread,
    time::{Duration, SystemTime},
};

use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

fn main() {
    println!("IMAGEPAL v0.1\n");

    let m = MultiProgress::new();

    let title_spinner = m.add(ProgressBar::new_spinner());
    let load_bar = m.add(ProgressBar::new(100));

    title_spinner.set_style(ProgressStyle::with_template("{spinner:.green} {msg}...\n\n").unwrap());

    let load_template = format!(
        "{{msg}}\n{{pos:>3.yellow}}{}{{len:3.yellow}} {}{{bar:40.green}}{} ({{eta:.yellow}})",
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
    for i in 0..100 {
        thread::sleep(Duration::from_millis(100));
        load_bar.inc(1);
        load_bar.set_message(format!("C:\\images\\...\\test\\{:05}.png", i));
        title_spinner.tick();
    }
    load_bar.finish_and_clear();

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

    println!("Done!")
}
