extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate reqwest;
extern crate serde;
extern crate serde_yaml;
extern crate statistical;
extern crate termion;
extern crate url;
#[macro_use]
extern crate serde_derive;
use clap::{App, Arg};
use std::{fs::File, path::Path, rc::Rc};

use self::trial::Trial;
fn load_config(path: &Path) -> self::config::RootConfig {
    return serde_yaml::from_reader(&mut File::open(&path).expect("Failed to open file"))
        .expect("Invalid Yaml");
}

mod config;
mod distributions;
mod trial;

fn main() {
    let matches = App::new("Perf Analyzer")
        .version("0.1.0")
        .author("Nathan Lincoln <nlincoln@intellifarms.com>")
        .about("Measure your webapps performance")
        .arg(
            Arg::with_name("EXPERIMENT")
                .required(true)
                .help("The config file of the experiment"),
        )
        .arg(
            Arg::with_name("baseline")
                .long("baseline")
                .takes_value(true)
                .default_value("master"),
        )
        .arg(
            Arg::with_name("compare")
                .short("c")
                .long("compare")
                .takes_value(true),
        )
        .get_matches();

    let config = load_config(&Path::new(matches.value_of("EXPERIMENT").unwrap()));
    let trials: Vec<Rc<Trial>> = From::from(config);
    for trial in trials {
        println!("Executing {}", trial);
        let result: self::trial::TrialResultSet = trial.execute();
        let durations: Vec<_> = result
            .results
            .iter()
            .map(|result| {
                let millis = result.duration.subsec_millis();
                let secs = result.duration.as_secs();
                (secs * 1000 + millis as u64) as f64
            })
            .collect();
        let mean = statistical::mean(&durations);
        let stddev = statistical::standard_deviation(&durations, Some(mean));
        println!("Mean: {}ms", mean);
        let n = durations.len();
        let alpha = 0.025; // a 5% confidence interval.
        let dof = n - 1;
        let t = distributions::lookup_value(dof as u16, alpha);
        let err = t * stddev / (n as f64).sqrt();
        let round = |val: f64| {
            let val = val * 1000.0;
            let val = val.round();
            val / 1000.0
        };
        println!(
            "95% CI: ({}ms - {}ms)",
            round(mean - err),
            round(mean + err)
        );
    }
}
