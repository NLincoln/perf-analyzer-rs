use crate::{config, distributions};
use std;
use std::collections::BTreeMap;
use std::rc::Rc;
use url::Url;

#[derive(Debug, Clone)]
pub struct Trial {
  pub name: String,
  pub query: BTreeMap<String, String>,
  pub headers: BTreeMap<String, String>,
  pub url: Url,
  pub samples: u64,
  pub warmup: u64,
}

impl From<config::RootConfig> for Vec<Rc<Trial>> {
  fn from(config: config::RootConfig) -> Vec<Rc<Trial>> {
    let mut result: Vec<Rc<Trial>> = Vec::new();
    let backup_url = if let Some(config) = config.config {
      config.url
    } else {
      None
    };
    for (name, contents) in config.experiments {
      let query_map = config::ParamValue::flatten(contents.query.unwrap_or_default());
      let params_map = config::ParamValue::flatten(contents.params.unwrap_or_default());
      result.reserve(query_map.len() * params_map.len());
      for query in query_map {
        result.push(Rc::new(Trial {
          name: name.clone(),
          query: query.clone(),
          samples: 25,
          warmup: 10,
          headers: contents.headers.clone().unwrap_or_default(),
          url: Url::parse(
            contents
              .url
              .clone()
              .unwrap_or_else(|| backup_url.clone().expect("Could not find URL for trial"))
              .as_str(),
          )
          .expect("Invalid URL"),
        }));
      }
    }
    result
  }
}

impl Trial {
  fn execute_once(&self) -> TrialResult {
    use reqwest::Client;

    use std::time::Instant;
    let req = Client::new()
      .get(&self.url.clone().into_string())
      .headers({
        use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

        let mut headers = HeaderMap::new();
        for (key, value) in self.headers.iter() {
          headers.insert(
            HeaderName::from_bytes(key.as_bytes()).unwrap(),
            HeaderValue::from_str(value.as_str()).unwrap(),
          );
        }
        headers
      })
      .query(&self.query);

    let start = Instant::now();
    let response = req.send().unwrap();
    let duration = Instant::now() - start;

    eprint!("{}\r", termion::clear::CurrentLine);
    eprint!("Time: {:?}", duration);

    TrialResult {
      duration,
      status_code: response.status(),
    }
  }

  pub fn execute(&self) -> TrialResultSet {
    let mut result_set = TrialResultSet {
      trial: self.clone(),
      results: Vec::new(),
    };
    for _ in 0..self.warmup {
      self.execute_once();
    }
    // TODO :: Dynamically calculate # of needed samples
    for _ in 0..self.samples {
      let result = self.execute_once();
      result_set.results.push(result);
    }
    eprint!("{}\r", termion::clear::CurrentLine);

    return result_set;
  }
}

use std::fmt;

impl fmt::Display for Trial {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{} on URL {} with Query {:?}",
      self.name, self.url, self.query
    )
  }
}

#[derive(Debug)]
pub struct TrialResultSet {
  pub trial: Trial,
  pub results: Vec<TrialResult>,
}

#[derive(Debug, Clone)]
pub struct TrialResult {
  pub duration: std::time::Duration,
  pub status_code: reqwest::StatusCode,
}

pub struct TrialAnalysis {
  pub result_set: TrialResultSet,
  pub stddev: f64,
  pub mean: f64,
  pub confidence_interval: (f64, f64),
}

impl From<TrialResultSet> for TrialAnalysis {
  fn from(set: TrialResultSet) -> TrialAnalysis {
    let durations: Vec<_> = set
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
    let n = durations.len();
    // Alpha represents our confidence in the results.
    // Here we have a 95% confidence level. This means
    // we take the 0.5 and divide it by 2 to get 0.025
    let alpha = 0.025;
    // Degrees of freedom is always number of samples - 1
    let dof = n - 1;
    // Look up the value in a t-table. This is more or less magic.
    let t = distributions::lookup_value(dof as u32, alpha);
    // Calculate the error
    let err = t * stddev / (n as f64).sqrt();
    TrialAnalysis {
      result_set: set,
      stddev,
      mean,
      confidence_interval: (mean - err, mean + err),
    }
  }
}

impl TrialAnalysis {
  pub fn is_statistically_equivalent_to(&self, other: &TrialAnalysis) -> bool {
    // Sp is the variance of both of our datasets
    let n1 = self.result_set.results.len() as f64;
    let n2 = other.result_set.results.len() as f64;
    let s1_2 = self.stddev.powi(2);
    let s2_2 = other.stddev.powi(2);

    let s1_over_n1 = s1_2 / n1;
    let s2_over_n2 = s2_2 / n2;

    let normed_variances = s1_over_n1 + s2_over_n2;

    let sp = (((n1 - 1.) * s1_2 + (n2 - 1.) as f64 * s2_2) / (n1 + n2 - 2.)).sqrt();

    // Our test statistic. We use this for determining whether the change is significant
    let t0 = (self.mean - other.mean) / (sp * normed_variances.sqrt());

    let dof = normed_variances.powi(2)
      / ((s1_over_n1).powi(2) / (n1 - 1.) + s2_over_n2.powi(2) / (n2 - 1.));
    // Our DOF should be an integer. If it's not, we round down
    // dof should also never be negative if we did the above correctly.
    assert!(dof > 0.);
    let dof = dof as u32;
    let t = distributions::lookup_value(dof, 0.025);

    t0.abs() < t
  }
}
