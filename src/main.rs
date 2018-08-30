extern crate clap;
extern crate hyper;
extern crate serde;
extern crate serde_yaml;
extern crate url;

use clap::{App, Arg};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;
use url::Url;

fn load_config(path: &Path) -> Value {
    return serde_yaml::from_reader(&mut File::open(&path).expect("Failed to open file"))
        .expect("Invalid Yaml");
}

#[derive(Clone)]
struct Trial {
    experiment: Rc<Experiment>,
    query: HashMap<String, String>,
}

impl Trial {
    pub fn gen_from_experiment(experiment: Rc<Experiment>) -> Vec<Trial> {
        /**
         * Iterate over each possible query param in the experiment
         */
        fn recurse(name: String, values: Vec<String>) -> HashMap<String, String> {}
    }

    pub fn name(&self) -> &String {
        &self.experiment.name
    }
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.experiment.headers
    }
    pub fn url(&self) -> Url {
        let mut url = self.experiment.url.clone();
        for (key, value) in &self.query {
            url.query_pairs_mut()
                .append_pair(key.as_str(), value.as_str());
        }
        return url;
    }

    pub fn execute(&self) -> TrialResult {
        use hyper::header::{HeaderMap, HeaderName, HeaderValue};
        use hyper::{rt::Future, Body, Client, Request, Uri};
        use std::time::Instant;
        let mut req = Request::get(self.url().clone().into_string());
        for (key, value) in self.headers() {
            req.header(key.as_str(), value.as_str());
        }
        let client = Client::new();

        let start = Instant::now();
        let future = client.request(req.body(Body::empty()).unwrap());
        let response = future.wait();
        TrialResult {
            trial: self.clone(),
            duration: Instant::now() - start,
        }
    }
}

#[derive(Clone)]
struct TrialResult {
    trial: Trial,
    duration: Duration,
}

#[derive(Debug, Clone)]
struct Experiment {
    name: String,
    branches: Option<Vec<String>>,
    query: HashMap<String, Vec<String>>,
    headers: HashMap<String, String>,
    url: Url,
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
                ))
            })
            .collect();
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
                seq.iter()
                    .map(|value| Self::yaml_primitive_to_string(value))
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
                    Self::yaml_primitive_to_string(key)
                        .expect("Keys for the query params must be strings"),
                    Self::yaml_value_to_array(value).expect("Failed to parse query param list."),
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
                    Self::yaml_primitive_to_string(key)
                        .expect("Keys for the headers must be strings"),
                    Self::yaml_primitive_to_string(value)
                        .expect("Each value for each header must be a string"),
                );
            }
            map
        } else {
            HashMap::new()
        }
    }

    fn from_experiment_key(name: &str, contents: &serde_yaml::Mapping) -> Experiment {
        Experiment {
            name: String::from(name),
            branches: None,
            query: Self::process_query_params(contents.get(&Value::String(String::from("query")))),
            headers: Self::process_headers(contents.get(&Value::String(String::from("headers")))),
            url: Url::parse(
                contents
                    .get(&Value::String(String::from("url")))
                    .expect("Url required")
                    .as_str()
                    .expect("Url must be a string"),
            ).expect("Url argument must be a valid URL"),
        }
    }
}

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

    println!("{:?}", experiments);
}