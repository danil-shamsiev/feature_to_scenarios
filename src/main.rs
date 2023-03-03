use gherkin::{Background, Examples, Feature, GherkinEnv, Scenario, Step};

use std::collections::HashMap;
use std::fs;

const FEATURE_FILES_PATH: &str = "./features/";
const TAG: &str = "tc_login_001";

#[derive(Debug)]
pub struct Example {
    pub data: Vec<HashMap<String, String>>,
}

impl Example {
    fn from(examples: &Vec<Examples>) -> Self {
        let mut tables = vec![];
        for example in examples.iter() {
            if let Some(table) = &example.table {
                tables.push(table);
            }
        }

        let vec_tables: Vec<Vec<String>> = tables
            .clone()
            .into_iter()
            .flat_map(|table| table.rows.clone())
            .collect();

        assert!(vec_tables
            .iter()
            .all(|ref v| v.len() == vec_tables[0].len()));

        let mut data = vec![];
        let mut titles = vec![];
        let mut values = vec![];

        for (i, vec_table) in vec_tables.iter().enumerate() {
            if i == 0 {
                titles = vec_table.clone();
            } else {
                values.push(vec_table.clone());
            }
        }

        for value in values.iter() {
            let mut dd = HashMap::new();
            for (i, title) in titles.iter().enumerate() {
                dd.insert(title.clone(), value[i].clone());
            }
            data.push(dd);
        }

        Example { data }
    }

    fn expand_scenario(&self, scenario: &Scenario) -> Vec<Scenario> {
        let mut scenarios = vec![];
        for example in self.data.iter() {
            let mut steps = scenario.steps.clone();
            let mut values = vec![];
            for step in steps.iter() {
                let mut new_step_value = step.value.clone();
                for key in example.keys() {
                    let pattern = format!("<{}>", key);
                    new_step_value = new_step_value.replace(&pattern, example.get(key).unwrap());
                }
                values.push(new_step_value);
            }
            for (i, value) in values.iter().enumerate() {
                steps[i] = Step {
                    value: value.to_string(),
                    ..steps[i].clone()
                };
            }
            scenarios.push(Scenario {
                steps,
                ..scenario.clone()
            });
        }
        scenarios
    }
}

pub fn prepend_background_steps(background: &Background, scenario: &Scenario) -> Scenario {
    let background_steps = background.steps.clone();
    let scenario_steps = scenario.steps.clone();
    let merged_steps = background_steps
        .iter()
        .cloned()
        .chain(scenario_steps.iter().cloned())
        .collect();
    Scenario {
        steps: merged_steps,
        ..scenario.clone()
    }
}

pub fn filter_and_expand(feature: &Feature, tag: &String) -> Vec<Scenario> {
    let background = &feature.background;
    feature
        .scenarios
        .iter()
        .filter(|&scenario| scenario.tags.contains(tag))
        .map(|scenario| match &background {
            None => scenario.clone(),
            Some(background) => prepend_background_steps(background, scenario),
        })
        .collect()
}

fn main() {
    fs::read_dir(FEATURE_FILES_PATH)
        .unwrap()
        .into_iter()
        .map(|path| path.unwrap().path().display().to_string())
        .map(|file| Feature::parse_path(file, GherkinEnv::new("formal").unwrap()).unwrap())
        .flat_map(|feature| filter_and_expand(&feature, &TAG.to_string()))
        .flat_map(|scenario| {
            let example = Example::from(&scenario.examples);
            example.expand_scenario(&scenario)
        })
        .for_each(|_| println!("1"));
}
