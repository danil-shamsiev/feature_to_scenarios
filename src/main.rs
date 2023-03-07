use gherkin::{Background, Examples, Feature, GherkinEnv, Scenario, Step};

use std::collections::HashMap;
use std::fs;

const FEATURE_FILES_PATH: &str = "./features/";
const TEMP_FEATURE_FILES_PATH: &str = "./features/temp/";
const TAG: &str = "automated";

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
        if self.data.len() == 0 {
            vec![scenario.clone()]
        } else {
            let mut scenarios = vec![];
            for example in self.data.iter() {
                let mut steps = scenario.steps.clone();
                let mut values = vec![];
                for step in steps.iter() {
                    let mut new_step_value = step.value.clone();
                    for key in example.keys() {
                        let pattern = format!("<{}>", key);
                        new_step_value =
                            new_step_value.replace(&pattern, example.get(key).unwrap());
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

pub fn filter_and_prepend_background(feature: &Feature, tag: &String) -> Vec<Scenario> {
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

pub fn feature_to_string(feature: &Feature) -> String {
    let mut output = String::new();
    output.push_str(&format!("{}: {}\n\n", feature.keyword, feature.name));
    for scenario in feature.scenarios.iter() {
        output.push_str(&format!("  Scenario: {}\n", scenario.name));
        for step in scenario.steps.iter() {
            output.push_str(&format!("    {}\n", step.to_string()));
        }
        output.push_str("\n");
    }
    output
}

pub fn expand_feature(feature: &Feature) -> Feature {
    let scenarios: Vec<Scenario> = feature
        .scenarios
        .iter()
        .flat_map(|scenario| {
            let example = Example::from(&scenario.examples);
            example.expand_scenario(&scenario)
        })
        .collect();
    Feature {
        scenarios,
        ..feature.clone()
    }
}

pub fn split_feature(feature: &Feature) -> Vec<Feature> {
    let mut features = vec![];
    for scenario in feature.scenarios.iter() {
        features.push(Feature {
            scenarios: vec![scenario.clone()],
            ..feature.clone()
        });
    }
    features
}

pub fn write_features(features: &Vec<Feature>) {
    fs::create_dir_all(TEMP_FEATURE_FILES_PATH).unwrap();
    for (i, feature) in features.iter().enumerate() {
        fs::write(
            &format!("{}{}.feature", TEMP_FEATURE_FILES_PATH, i),
            feature_to_string(feature),
        )
        .unwrap();
    }
}

fn main() {
    let features: Vec<Feature> = fs::read_dir(FEATURE_FILES_PATH)
        .unwrap()
        .into_iter()
        .map(|path| path.unwrap().path().display().to_string())
        .map(|file| Feature::parse_path(file, GherkinEnv::new("formal").unwrap()).unwrap())
        .map(|feature| {
            let scenarios = filter_and_prepend_background(&feature, &TAG.to_string());
            Feature {
                scenarios,
                ..feature.clone()
            }
        })
        .filter(|feature| feature.scenarios.len() != 0)
        .map(|feature| expand_feature(&feature))
        .flat_map(|feature| split_feature(&feature))
        .collect();
    write_features(&features);
}
