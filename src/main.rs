extern crate clap;
extern crate hyper;
extern crate serde;
extern crate serde_yaml;
extern crate url;

use clap::{App, Arg};
use serde_yaml::Value;
use std::fs::File;
use std::path::Path;

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
            Arg::with_name("branches")
                .short("b")
                .long("branches")
                .multiple(true)
                .takes_value(true),
        )
        .get_matches();

    let branches: Option<Vec<String>> = matches.values_of("branches").map(|branches| {
        branches
            .into_iter()
            .map(|branch| String::from(branch))
            .collect()
    });

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
