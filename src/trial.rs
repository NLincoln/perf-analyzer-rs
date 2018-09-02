use experiment::Experiment;
use hyper;
use std;
use std::collections::HashMap;
use std::rc::Rc;
use url::Url;

#[derive(Debug, Clone)]
pub struct Trial {
  experiment: Rc<Experiment>,
  query: HashMap<String, String>,
}

impl Trial {
  pub fn gen_from_experiment(experiment: Rc<Experiment>) -> Vec<Trial> {
    let mut queries: Vec<HashMap<String, String>> = Vec::new();
    struct QueryIterator {
      key: String,
      index: usize,
    }

    impl QueryIterator {
      fn is_done(&self, values: &HashMap<String, Vec<String>>) -> bool {
        return values[&self.key].len() - 1 == self.index;
      }

      fn next(&mut self, values: &HashMap<String, Vec<String>>) {
        if self.is_done(values) {
          self.index = 0;
        } else {
          self.index += 1;
        }
      }
    }

    fn is_done(iterators: &Vec<QueryIterator>, values: &HashMap<String, Vec<String>>) -> bool {
      for it in iterators {
        if !it.is_done(values) {
          return false;
        }
      }
      return true;
    }

    let mut iterators: Vec<QueryIterator> = Vec::new();
    for key in experiment.query().keys() {
      iterators.push(QueryIterator {
        key: key.clone(),
        index: 0,
      });
    }

    let values = experiment.query();

    while !is_done(&iterators, values) {
      let mut query_map: HashMap<String, String> = HashMap::new();
      for it in iterators.iter() {
        query_map.insert(it.key.clone(), values[&it.key][it.index].clone());
      }
      queries.push(query_map);
      for it in iterators.iter_mut() {
        it.next(values);
      }
    }

    return queries
      .into_iter()
      .map(|query| Trial {
        experiment: experiment.clone(),
        query,
      })
      .collect();
  }

  pub fn name(&self) -> &String {
    &self.experiment.name()
  }
  pub fn headers(&self) -> &HashMap<String, String> {
    &self.experiment.headers()
  }
  pub fn url(&self) -> Url {
    let mut url = self.experiment.url().clone();
    for (key, value) in &self.query {
      url
        .query_pairs_mut()
        .append_pair(key.as_str(), value.as_str());
    }
    return url;
  }

  pub fn execute(&self) -> TrialResult {
    use hyper::{rt::Future, Body, Client, Request};
    use std::time::Instant;
    let mut req = Request::get(self.url().clone().into_string());
    for (key, value) in self.headers() {
      req.header(key.as_str(), value.as_str());
    }
    let client = Client::new();

    let start = Instant::now();
    let future = client
      .request(req.body(Body::empty()).unwrap())
      .map(|_| {
        println!("Successfully ran future");
      })
      .map_err(|err| {
        eprintln!("Error {}", err);
      });

    hyper::rt::run(future);

    TrialResult {
      trial: self.clone(),
      duration: Instant::now() - start,
    }
  }
}

#[derive(Debug, Clone)]
pub struct TrialResult {
  trial: Trial,
  duration: std::time::Duration,
}