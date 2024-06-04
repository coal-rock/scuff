use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Deserialize, Serialize, Debug)]
pub enum Extension {
    Pen,
    Music,
    VideoSensing,
    Text2Speech,
    Translate,
    Makeymakey,
    Microbit,
    EV3,
    Boost,
    Wedo2,
    Gdxfor,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Asset {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Stage {
    pub name: String,
    pub script: PathBuf,
    pub backdrops: Vec<Asset>,
    pub sounds: Vec<Asset>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Sprite {
    pub name: String,
    pub script: PathBuf,
    pub costumes: Vec<Asset>,
    pub sounds: Vec<Asset>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Makefile {
    pub project_name: String,
    pub stage: Vec<Stage>,
    pub sprite: Vec<Sprite>,
    pub extensions: Vec<Extension>,
}
