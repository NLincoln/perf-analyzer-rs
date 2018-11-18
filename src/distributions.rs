// Numbers in the t-table set go:
// .25 .20 .15 .10 .05 .025 .02 .01 .005 .0025 .001 .0005

use std::collections::BTreeMap;

type Data = BTreeMap<String, Vec<f64>>;

lazy_static! {
  static ref PARSED_DATA: Data = { serde_yaml::from_str(include_str!("t-table.yaml")).unwrap() };
}

fn key_for_dof(dof: u32) -> String {
  if dof <= 30 {
    return dof.to_string();
  }
  if dof <= 60 {
    // Round to the nearest 10
    return ((dof as f64 / 10.).round() as u64 * 10).to_string();
  }
  for i in &[80, 100, 1000] {
    if dof < *i {
      return i.to_string();
    }
  }
  return "z*".to_string();
}

fn index_for_alpha(alpha: f64) -> usize {
  if alpha < 0.0005 {
    return 11;
  }
  if alpha < 0.001 {
    return 10;
  }
  if alpha < 0.0025 {
    return 9;
  }
  if alpha < 0.005 {
    return 8;
  }
  if alpha < 0.01 {
    return 7;
  }
  if alpha < 0.02 {
    return 6;
  }
  if alpha < 0.025 {
    return 5;
  }
  if alpha < 0.05 {
    return 4;
  }
  if alpha < 0.10 {
    return 3;
  }
  if alpha < 0.15 {
    return 2;
  }
  if alpha < 0.20 {
    return 1;
  }
  if alpha < 0.25 {
    return 0;
  }
  panic!("No t-table data available for alpha {}", alpha);
}

pub fn lookup_value(dof: u32, alpha: f64) -> f64 {
  let key = key_for_dof(dof);
  let index = index_for_alpha(alpha);
  return PARSED_DATA[&key][index];
}
