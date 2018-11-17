extern crate clap;
extern crate hyper;
extern crate serde;
extern crate serde_yaml;
extern crate url;
#[macro_use]
extern crate serde_derive;
use clap::{App, Arg};
use serde_yaml::Value;
use std::fs::File;
use std::path::Path;

use self::experiment::Experiment;
use self::trial::Trial;
fn load_config(path: &Path) -> Value {
    return serde_yaml::from_reader(&mut File::open(&path).expect("Failed to open file"))
        .expect("Invalid Yaml");
}

mod experiment;
mod trial;
use experiment::*;
use trial::*;

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
    let experiments = Experiment::from_config(config);
    let trials: Vec<Vec<Trial>> = experiments
        .iter()
        .map(|experiment| Trial::gen_from_experiment(experiment.clone()))
        .collect();
    for trials in trials {
        for trial in trials {
            println!("Executing {}", trial);
            let result = trial.execute();
        }
    }
}
