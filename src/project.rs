use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub targets: Vec<Target>,
    pub monitors: Vec<String>,
    pub extensions: Vec<String>,
    pub meta: Meta,
}

impl Project {
    pub fn new() -> Project {
        Project {
            targets: vec![],
            monitors: Vec::new(),
            extensions: Vec::new(),
            meta: Meta::new(),
        }
    }
}

type Value = String;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub is_stage: bool,
    pub name: String,
    pub variables: HashMap<String, serde_json::Value>,
    pub lists: HashMap<String, (String, Vec<Value>)>,
    // TODO
    pub broadcasts: HashMap<String, String>,
    pub blocks: HashMap<String, Block>,
    pub comments: HashMap<String, Comment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_costume: Option<i32>,
    pub costumes: Vec<Costume>,
    pub sounds: Vec<Sound>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer_order: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<i32>,
}

impl Target {
    pub fn default() -> Target {
        Target {
            is_stage: true,
            name: "Stage".to_string(),
            variables: HashMap::new(),
            lists: HashMap::new(),
            broadcasts: HashMap::new(),
            blocks: HashMap::new(),
            comments: HashMap::new(),
            current_costume: None,
            costumes: Vec::new(),
            sounds: Vec::new(),
            layer_order: None,
            volume: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub opcode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// An object associating names with arrays representing inputs into
    /// which other blocks may be dropped, including C mouths.
    ///
    /// The first element of each array is 1 if the input is a shadow,
    /// 2 if there is no shadow, and 3 if there is a shadow but it is
    /// obscured by the input.
    ///
    /// The second is either the ID of the input or an array representing
    /// it as described in the table below.
    ///
    /// If there is an obscured shadow, the third element is its ID or
    /// an array representing it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_level: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation: Option<Mutation>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Mutation {
    // this struct is fucked up, and it's not my fault
    // scratch demands these values as the string casted,
    // quote-escaped serialized versions of their json-equivalent
    // why this is i have absolutely no clue
    //
    // in all seriousness, i should prob just skip
    // typing on this struct, and replace it with a:
    // serde::Value
    pub tag_name: String,
    pub children: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hasnext: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proccode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argumentids: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argumentnames: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argumentdefaults: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warp: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    block_id: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    minimized: bool,
    text: String,
}

// #[derive(Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// struct Asset {
//     // MD5 hash of the asset file
//     asset_id: String,
//     name: String,
//     md5ext: String,
//     data_format: String,
// }

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Costume {
    pub name: String,
    pub data_format: String,
    pub asset_id: String,
    pub md5ext: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_center_x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_center_y: Option<i32>,
}

impl Costume {
    pub fn default() -> Costume {
        Costume {
            name: "backdrop1".to_string(),
            data_format: "svg".to_string(),
            asset_id: "cd21514d0531fdffb22204e0ec5ed84a".to_string(),
            md5ext: "cd21514d0531fdffb22204e0ec5ed84a.svg".to_string(),
            rotation_center_x: None,
            rotation_center_y: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Sound {
    rate: i32,
    sample_count: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Meta {
    semver: String,
    vm: String,
    agent: String,
}

impl Meta {
    fn new() -> Meta {
        Meta {
            semver: "3.0.0".to_string(),
            vm: "0.2.0".to_string(),
            agent: "scuff".to_string(),
        }
    }
}
