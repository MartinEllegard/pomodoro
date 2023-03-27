use clap::{App, Arg};
use std::io::{stdout, Write};
use async_std::task::sleep;
use async_std::task;
use std::time::Duration;

fn main() {
    let matches = App::new("Pomodoro CLI")
        .version("0.1.0")
        .author("Your Name <your.email@example.com>")
        .about("A simple Pomodoro Technique timer for the command line")
        .arg(
            Arg::with_name("work")
                .short('w')
                .long("work")
                .value_name("WORK")
                .help("Sets the duration of work intervals in minutes")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("break")
                .short('b')
                .long("break")
                .value_name("BREAK")
                .help("Sets the duration of break intervals in minutes")
                .takes_value(true),
        )
        .get_matches();

    // You can use the following to get the values for the work and break durations
    let work_duration = matches
        .value_of("work")
        .unwrap_or("25")
        .parse::<u32>()
        .expect("Invalid work duration");
    let break_duration = matches
        .value_of("break")
        .unwrap_or("5")
        .parse::<u32>()
        .expect("Invalid break duration");

    println!("Work: {} minutes, Break: {} minutes", work_duration, break_duration);
    task::block_on(pomodoro(work_duration, break_duration));
}

async fn pomodoro(work_duration: u32, break_duration: u32) {
    let work_duration_secs = work_duration * 60;
    let break_duration_secs = break_duration * 60;

    loop {
        println!("Work time!");

        for remaining_secs in (0..work_duration_secs).rev() {
            print!("\r{}:{} remaining", remaining_secs / 60, remaining_secs % 60);
            stdout().flush().unwrap();
            sleep(Duration::from_secs(1)).await;
        }

        println!("\nBreak time!");

        for remaining_secs in (0..break_duration_secs).rev() {
            print!("\r{}:{} remaining", remaining_secs / 60, remaining_secs % 60);
            stdout().flush().unwrap();
            sleep(Duration::from_secs(1)).await;
        }
    } 
}
