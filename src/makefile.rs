use serde::{Deserialize, Serialize};
use std::{
    fs::{read, read_to_string},
    path::{Path, PathBuf},
};

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
struct Asset {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Deserialize, Serialize, Debug)]
struct Stage {
    pub name: String,
    pub script: PathBuf,
    pub backdrops: Vec<Asset>,
    pub sounds: Vec<Asset>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Sprite {
    pub name: String,
    pub script: PathBuf,
    pub costumes: Vec<Asset>,
    pub sounds: Vec<Asset>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Makefile {
    pub project_name: String,
    pub stage: Vec<Stage>,
    pub sprite: Vec<Sprite>,
    pub extensions: Vec<Extension>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AssetData {
    name: String,
    content: Vec<u8>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SpriteStageData {
    pub name: String,
    pub is_stage: bool,
    pub script: String,
    pub costumes_backdrops: Vec<AssetData>,
    pub sounds: Vec<AssetData>,
}

// like Makefile, but contains data from each file listed instead of paths
#[derive(Debug)]
pub struct MakefileData {
    pub project_name: String,
    pub sprites_stages: Vec<SpriteStageData>,
    pub extensions: Vec<Extension>,
}

impl MakefileData {
    pub fn parse(makefile_path: PathBuf) -> MakefileData {
        let makefile: Makefile =
            toml::from_str(&read_to_string(makefile_path.clone()).unwrap()).unwrap();

        let project_path = makefile_path.clone();
        let project_path = project_path.parent().unwrap();

        let mut sprites_stages: Vec<SpriteStageData> = vec![];

        for stage in makefile.stage {
            sprites_stages.push(SpriteStageData {
                name: stage.name,
                script: read_to_string(MakefileData::get_path(project_path, stage.script)).unwrap(),
                costumes_backdrops: MakefileData::read_assets(project_path, stage.backdrops),
                sounds: MakefileData::read_assets(project_path, stage.sounds),
                is_stage: true,
            });
        }

        for sprite in makefile.sprite {
            sprites_stages.push(SpriteStageData {
                name: sprite.name,
                is_stage: false,
                script: read_to_string(MakefileData::get_path(project_path, sprite.script))
                    .unwrap(),
                costumes_backdrops: MakefileData::read_assets(project_path, sprite.costumes),
                sounds: MakefileData::read_assets(project_path, sprite.sounds),
            });
        }

        MakefileData {
            project_name: makefile.project_name,
            sprites_stages,
            extensions: makefile.extensions,
        }
    }

    fn get_path(project_path: &Path, file_path: PathBuf) -> String {
        project_path.join(file_path).to_str().unwrap().to_string()
    }

    fn read_assets(project_path: &Path, assets: Vec<Asset>) -> Vec<AssetData> {
        assets
            .iter()
            .map(|asset| AssetData {
                name: asset.name.clone(),
                content: read(MakefileData::get_path(project_path, asset.path.clone())).unwrap(),
            })
            .collect()
    }
}
