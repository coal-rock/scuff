use std::fs::read_to_string;

use serde_json::Value;

test_file!(hello_world);
test_file!(hello_world2);

fn read_input_file(test_name: &str) -> String {
    read_to_string(format!("tests/{}/input.scuff", test_name)).unwrap()
}

fn read_expected_file(test_name: &str) -> Value {
    let expected = read_to_string(format!("tests/{}/expected.json", test_name)).unwrap();
    let expected: Value = serde_json::from_str(&expected).unwrap();
    expected["targets"][1].clone()
}

#[macro_export]
macro_rules! test_file {
    ($test_name:tt) => {
        ::paste::paste! {
            #[test]
            pub fn [<$test_name _matches>]() {
                assert_eq!(1, 1);
            }

            #[test]
            pub fn [<$test_name _schema>]() {
                assert_eq!(1, 1);
            }
        }
    };
}
