use serde_yaml;
use serde_yaml::Value;
use std::collections::HashMap;
use std::rc::Rc;
use url::Url;

#[derive(Debug, Clone)]
struct Config {
  url: Option<Url>,
  headers: HashMap<String, String>,
  query: HashMap<String, Vec<String>>,
  branches: Option<Vec<String>>,
  samples: u64,
  warmup: u64,
}

fn get_number(config: &serde_yaml::Mapping, key: &str, default: u64) -> u64 {
  config
    .get(&Value::String(String::from(key)))
    .map_or(default, |samples| {
      samples
        .as_u64()
        .expect("The number of samples must be a number")
    })
}

impl Config {
  pub fn create(config: &serde_yaml::Mapping) -> Config {
    Config {
      url: config
        .get(&Value::String(String::from("url")))
        .map(|url| Url::parse(url.as_str().expect("Url must be a string")).expect("Invalid URL")),
      branches: None,
      query: process_query_params(config.get(&Value::String(String::from("query")))),
      headers: process_headers(config.get(&Value::String(String::from("headers")))),
      samples: get_number(&config, "samples", 25),
      warmup: get_number(&config, "warmup", 10),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Experiment {
  name: String,
  branches: Option<Vec<String>>,
  query: HashMap<String, Vec<String>>,
  headers: HashMap<String, String>,
  url: Url,

  samples: u64,
  warmup: u64,
}
fn array_of_opt_to_opt_array<T>(vec: Vec<Option<T>>) -> Option<Vec<T>> {
  let mut result = Vec::new();
  result.reserve_exact(vec.capacity());
  for item in vec {
    if let Some(value) = item {
      result.push(value);
    } else {
      return None;
    }
  }
  return Some(result);
}

impl Experiment {
  pub fn from_config(config: Value) -> Vec<Rc<Experiment>> {
    let root_config: Option<Config> = config.get("config").map(|config| {
      Config::create(
        config
          .as_mapping()
          .expect("root config key must be a mapping"),
      )
    });

    return config
      .get("experiments")
      .expect("Root experiments: key is required")
      .as_mapping()
      .expect("Experiments key must be a mapping")
      .iter()
      .map(|(key, value)| {
        Rc::new(Self::from_experiment_key(
          key.as_str().unwrap(),
          value
            .as_mapping()
            .expect("Each experiment must be a mapping"),
          &root_config,
        ))
      })
      .collect();
  }

  fn from_experiment_key(
    name: &str,
    contents: &serde_yaml::Mapping,
    config: &Option<Config>,
  ) -> Experiment {
    let url = contents.get(&Value::String(String::from("url")));
    let url = match url {
      Some(url) => {
        Url::parse(url.as_str().expect("Url must be a string")).expect("url must be a valid url")
      }
      None => match config.clone().and_then(|config| config.url) {
        Some(url) => url,
        None => panic!("Url is required"),
      },
    };

    let mut query = process_query_params(contents.get(&Value::String(String::from("query"))));

    match config {
      Some(config) => query.extend(config.clone().query),
      None => {}
    };

    let mut headers = process_headers(contents.get(&Value::String(String::from("headers"))));

    match config {
      Some(config) => headers.extend(config.clone().headers),
      None => {}
    }

    Experiment {
      name: String::from(name),
      branches: None,
      query,
      headers,
      url,
      samples: get_number(&contents, "samples", 25),
      warmup: get_number(&contents, "warmup", 10),
    }
  }

  pub fn query(&self) -> &HashMap<String, Vec<String>> {
    &self.query
  }

  pub fn name(&self) -> &String {
    &self.name
  }

  pub fn headers(&self) -> &HashMap<String, String> {
    &self.headers
  }

  pub fn url(&self) -> &Url {
    &self.url
  }

  pub fn samples(&self) -> u64 {
    self.samples
  }
  pub fn warmup(&self) -> u64 {
    self.warmup
  }
}

fn yaml_primitive_to_string(value: &Value) -> Option<String> {
  match value {
    Value::Number(number) => Some(format!("{}", number)),
    Value::String(string) => Some(string.clone()),
    _ => None,
  }
}

fn yaml_value_to_array(value: &Value) -> Option<Vec<String>> {
  match value {
    Value::Sequence(seq) => array_of_opt_to_opt_array(
      seq
        .iter()
        .map(|value| yaml_primitive_to_string(value))
        .collect(),
    ),
    Value::String(string) => Some(vec![string.clone()]),
    Value::Number(num) => Some(vec![format!("{}", num)]),
    _ => None,
  }
}

fn process_query_params(query: Option<&Value>) -> HashMap<String, Vec<String>> {
  if let Some(query) = query {
    let query = query.as_mapping().expect("Query Params must be a mapping");
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for (key, value) in query.iter() {
      map.insert(
        yaml_primitive_to_string(key).expect("Keys for the query params must be strings"),
        yaml_value_to_array(value).expect("Failed to parse query param list."),
      );
    }
    map
  } else {
    HashMap::new()
  }
}

fn process_headers(headers: Option<&Value>) -> HashMap<String, String> {
  if let Some(headers) = headers {
    let headers = headers
      .as_mapping()
      .expect("Headers must be a mapping of strings to strings");
    let mut map: HashMap<String, String> = HashMap::new();
    for (key, value) in headers.iter() {
      map.insert(
        yaml_primitive_to_string(key).expect("Keys for the headers must be strings"),
        yaml_primitive_to_string(value).expect("Each value for each header must be a string"),
      );
    }
    map
  } else {
    HashMap::new()
  }
}
