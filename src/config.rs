use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum QueryEntry {
  String(String),
  Integer(i64),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum QueryValue {
  Value(QueryEntry),
  List(Vec<QueryEntry>),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Experiment {
  url: Option<String>,
  headers: Option<HashMap<String, String>>,
  query: Option<HashMap<String, QueryValue>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct BaseConfig {
  url: Option<String>,
  headers: Option<HashMap<String, String>>,
  query: Option<HashMap<String, QueryValue>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RootConfig {
  pub experiments: HashMap<String, Experiment>,
  pub config: Option<BaseConfig>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_yaml::from_str;
  use std::collections::HashMap;
  use std::iter::FromIterator;

  fn parse(text: &str) -> RootConfig {
    from_str(text).expect("Failed to parse assumed-valid input")
  }

  #[test]
  fn test_one_experiment() {
    assert_eq!(
      parse(
        r#"
experiments:
  sample:
    url: http://localhost
    headers:
      Auth: Bearer foo
    query:
      user_id: barbaz
      other_thing: 55
      account_id:
        - foobar
        - 6
"#
      ),
      RootConfig {
        config: None,
        experiments: HashMap::from_iter(vec![(
          "sample".into(),
          Experiment {
            url: Some("http://localhost".into()),
            headers: Some(HashMap::from_iter(vec![(
              "Auth".into(),
              "Bearer foo".into()
            )])),
            query: Some(HashMap::from_iter(vec![
              (
                "user_id".into(),
                QueryValue::Value(QueryEntry::String("barbaz".into()))
              ),
              (
                "other_thing".into(),
                QueryValue::Value(QueryEntry::Integer(55))
              ),
              (
                "account_id".into(),
                QueryValue::List(vec![
                  QueryEntry::String("foobar".into()),
                  QueryEntry::Integer(6)
                ])
              )
            ]))
          }
        )])
      }
    )
  }

  #[test]
  fn test_minimal() {
    assert_eq!(
      parse(
        r#"
experiments: {}
"#
      ),
      RootConfig {
        config: None,
        experiments: HashMap::new()
      }
    )
  }
}
