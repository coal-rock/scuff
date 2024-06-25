use crate::test_file;

use assert_json_diff::assert_json_eq;
use serde_json::Value;
use std::fs::read_to_string;

test_file!(hello_world);
test_file!(join_string);
test_file!(join_variables);
test_file!(functions);
test_file!(function_args);

#[macro_export]
macro_rules! test_file {
    ($test_name:tt) => {
        ::paste::paste! {
            #[test]
            pub fn [<$test_name _matches>]() {
                let path = format!("tests/{}/project.toml", stringify!($test_name));
                let (project, _) = crate::compile_project(path);

                let actual = serde_json::to_value(&project).unwrap();

                println!("{} project.json:\n{}", stringify!($test_name), serde_json::to_string_pretty(&project).unwrap());

                let expected = read_to_string(format!("tests/{}/expected.json", stringify!($test_name))).unwrap();

                let expected: Value = serde_json::from_str(&expected).unwrap();
                assert_json_eq!(actual, expected);
            }

            #[test]
            pub fn [<$test_name _schema>]() {
                let path = format!("tests/{}/project.toml", stringify!($test_name));
                let (project, _) = crate::compile_project(path);
                crate::validate_project(&project);
            }
        }
    };
}
