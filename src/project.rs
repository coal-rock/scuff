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
            targets: vec![Target::default()],
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
    pub variables: HashMap<String, (String, Value)>,
    pub lists: HashMap<String, (String, Vec<Value>)>,
    // TODO
    pub broadcasts: HashMap<String, String>,
    pub blocks: HashMap<String, Block>,
    pub comments: HashMap<String, Comment>,
    pub current_costume: i32,
    pub costumes: Vec<Costume>,
    pub sounds: Vec<Sound>,
    pub layer_order: i32,
    pub volume: i32,
}

impl Target {
    fn default() -> Target {
        Target {
            is_stage: true,
            name: "Stage".to_string(),
            variables: HashMap::new(),
            lists: HashMap::new(),
            broadcasts: HashMap::new(),
            blocks: HashMap::new(),
            comments: HashMap::new(),
            current_costume: 0,
            costumes: vec![Costume::default()],
            sounds: Vec::new(),
            layer_order: 0,
            volume: 100,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub opcode: String,
    pub next: Option<String>,
    pub parent: Option<String>,
    pub inputs: HashMap<String, Vec<serde_json::Value>>,
    pub fields: HashMap<String, Vec<Option<String>>>,
    pub shadow: bool,
    pub top_level: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation: Option<Mutation>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Mutation {
    // this struct is fucked up, and it's not my fault
    // scratch demands these values as the string casted,
    // quote-escaped serialized versions of their json-equivalent
    // why this is i have absolutely no clue
    pub tag_name: String,
    pub children: Vec<String>,
    pub proccode: String,
    pub argumentids: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argumentnames: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argumentdefaults: Option<String>,
    pub warp: String,
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Asset {
    // MD5 hash of the asset file
    asset_id: String,
    name: String,
    md5ext: String,
    data_format: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Costume {
    name: String,
    data_format: String,
    asset_id: String,
    md5ext: String,
    rotation_center_x: i32,
    rotation_center_y: i32,
}

impl Costume {
    fn default() -> Costume {
        Costume {
            name: "backdrop1".to_string(),
            data_format: "svg".to_string(),
            asset_id: "cd21514d0531fdffb22204e0ec5ed84a".to_string(),
            md5ext: "cd21514d0531fdffb22204e0ec5ed84a.svg".to_string(),
            rotation_center_x: 240,
            rotation_center_y: 180,
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
