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
  let mut analyses: Vec<trial::TrialAnalysis> = vec![];

  for trial in trials {
    println!("Executing {}", trial);
    let result: self::trial::TrialResultSet = trial.execute();
    let analysis = trial::TrialAnalysis::from(result);

    let round = |val: f64| {
      let val = val * 1000.0;
      let val = val.round();
      val / 1000.0
    };
    println!("Mean {}ms", analysis.mean);
    println!(
      "95% CI: ({}ms - {}ms)",
      round(analysis.confidence_interval.0),
      round(analysis.confidence_interval.1)
    );
    analyses.push(analysis);
  }
  println!("=> Comparing trials against each other");

  for i in 0..analyses.len() {
    for j in 0..analyses.len() {
      if i == j {
        continue;
      }
      let a = &analyses[i];
      let b = &analyses[j];
      if a.is_statistically_equivalent_to(b) {
        println!(
          "No significant difference was found between trials {} and {}",
          a.result_set.trial, b.result_set.trial
        );
        println!("This usually means you can remove some parameters from the config file");
        println!("");
      }
    }
  }
}
