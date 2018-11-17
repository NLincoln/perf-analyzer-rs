use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
#[serde(untagged)]
pub enum ParamEntry {
  String(String),
  Integer(i64),
  Boolean(bool),
}

impl ToString for ParamEntry {
  fn to_string(&self) -> String {
    use self::ParamEntry::*;
    match self {
      String(val) => val.clone(),
      Integer(val) => val.to_string(),
      Boolean(val) => match val {
        true => "true".into(),
        false => "false".into(),
      },
    }
  }
}

#[derive(Debug, Clone, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
#[serde(untagged)]
pub enum ParamValue {
  Value(ParamEntry),
  List(Vec<ParamEntry>),
}

pub fn cross<K: Clone + Ord, V: Clone>(map: BTreeMap<K, Vec<V>>) -> Vec<BTreeMap<K, V>> {
  let mut result: Vec<BTreeMap<K, V>> = vec![];
  for (key, value) in map.into_iter() {
    if result.is_empty() {
      for item in value {
        let mut map = BTreeMap::new();
        map.insert(key.clone(), item);
        result.push(map);
      }
    } else {
      let mut next_result = vec![];
      for item in value {
        let mut result_copy = result.clone();
        for entry in result_copy.iter_mut() {
          entry.insert(key.clone(), item.clone());
        }
        next_result.append(&mut result_copy);
      }
      result = next_result;
    }
  }

  result
}

impl ParamValue {
  /// Transform a BTreeMap of Values into a list of key-value pairs
  /// So for example, the following yaml:
  /// ```yaml
  /// a:
  ///   - 1
  ///   - 2
  /// b:
  ///   - 3
  ///   - 4
  /// ```
  /// Would be transformed into the following:
  /// ```json
  /// [
  ///   { a: "1", b: "3" },
  ///   { a: "1", b: "4" },
  ///   { a: "2", b: "3" },
  ///   { a: "2", b: "4" },
  /// ]
  /// ```
  pub fn flatten(map: BTreeMap<String, ParamValue>) -> Vec<BTreeMap<String, String>> {
    cross(
      map
        .into_iter()
        .map(|(key, value)| (key, value.into_normalized()))
        .collect(),
    )
  }
  fn into_normalized(self) -> Vec<String> {
    match self {
      ParamValue::List(items) => items.iter().map(ToString::to_string).collect(),
      ParamValue::Value(val) => vec![val.to_string()],
    }
  }
}

#[derive(Debug, Clone, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
pub struct Experiment {
  pub url: Option<String>,
  pub headers: Option<BTreeMap<String, String>>,
  pub query: Option<BTreeMap<String, ParamValue>>,
  pub params: Option<BTreeMap<String, ParamValue>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
pub struct BaseConfig {
  pub url: Option<String>,
  pub headers: Option<BTreeMap<String, String>>,
  pub warmup: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
pub struct RootConfig {
  pub experiments: BTreeMap<String, Experiment>,
  pub config: Option<BaseConfig>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_yaml::from_str;
  use std::collections::BTreeMap;
  use std::iter::FromIterator;

  #[test]
  fn test_flatten() {
    assert_eq!(
      ParamValue::flatten(BTreeMap::from_iter(vec![
        (
          "a".into(),
          ParamValue::List(vec![ParamEntry::Integer(1), ParamEntry::Integer(2)])
        ),
        (
          "b".into(),
          ParamValue::List(vec![ParamEntry::Integer(3), ParamEntry::Integer(4)])
        )
      ])),
      vec![
        BTreeMap::from_iter(vec![("a".into(), "1".into()), ("b".into(), "3".into())]),
        BTreeMap::from_iter(vec![("a".into(), "2".into()), ("b".into(), "3".into())]),
        BTreeMap::from_iter(vec![("a".into(), "1".into()), ("b".into(), "4".into())]),
        BTreeMap::from_iter(vec![("a".into(), "2".into()), ("b".into(), "4".into())]),
      ]
    )
  }

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
        experiments: BTreeMap::from_iter(vec![(
          "sample".into(),
          Experiment {
            url: Some("http://localhost".into()),
            headers: Some(BTreeMap::from_iter(vec![(
              "Auth".into(),
              "Bearer foo".into()
            )])),
            params: None,
            query: Some(BTreeMap::from_iter(vec![
              (
                "user_id".into(),
                ParamValue::Value(ParamEntry::String("barbaz".into()))
              ),
              (
                "other_thing".into(),
                ParamValue::Value(ParamEntry::Integer(55))
              ),
              (
                "account_id".into(),
                ParamValue::List(vec![
                  ParamEntry::String("foobar".into()),
                  ParamEntry::Integer(6)
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
        experiments: BTreeMap::new()
      }
    )
  }
}
