use std::fs::{DirBuilder, OpenOptions, File, metadata};
use std::io::{Read, Write};
use std::path::Path;

use crate::types::App;

pub fn load_data() -> (Box<Path>, App) {
    
    //find/create data dir
    let data_dir = dirs::data_dir()
        .unwrap()
        .join("save-manager")
        .into_boxed_path();

    //if data dir doesn't exist, create it
    if !metadata(&data_dir).is_ok() {
        DirBuilder::new()
            .recursive(true)
            .create(&data_dir)
            .unwrap();
    }
    
    let json_pathbuf = data_dir.join("data.json");
    let json_path = json_pathbuf.as_path();
    
    //read/create data
    let mut file: File = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(json_path).unwrap();

    let mut json = String::new();
    file.read_to_string(&mut json).unwrap();
    
    //if file was just created (empty) populate it
    if json.is_empty() {
        json = serde_json::to_string(
            &App {current:None, categories: Vec::new()})
            .expect("failed to convert object to json string");
    }
    
    //parse json
    let app: App = serde_json::from_str(&json).unwrap();
    
    //return useful stuff
    (data_dir, app)
}

pub fn save_data(data_dir: &Path, app: App) {
    
    let data_json_string = serde_json::to_string(&app).unwrap();
    let json_pathbuf = data_dir.join("data.json");
    let json_path = json_pathbuf.as_path();

    let mut file: File = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(json_path.as_os_str()).unwrap();

    file.write_all(data_json_string.as_bytes()).unwrap();
}
