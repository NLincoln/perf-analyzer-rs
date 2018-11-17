use crate::config;
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
    for _ in 0..25 {
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
