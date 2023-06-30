use std::env::args;
use std::io::{Read, Write};
use std::fs::{DirBuilder, OpenOptions, File, canonicalize};
use std::fs;
use std::path::{Path, PathBuf};
use std::clone::Clone;
use serde::{Deserialize, Serialize};
use colored::Colorize;

#[derive(Serialize, Deserialize, Debug)]
struct App {
    current: Option<String>,
    categories: Vec<Category>
}
impl App {
    fn current_category(&self) -> Option<&Category> {
        if self.current.is_none() { return None; }
        self.categories.iter().find(|a| &a.name == self.current.as_ref().unwrap())
    }
    fn current_category_mut(&mut self) -> Option<&mut Category> {
        if self.current.is_none() { return None; }
        let index = self.categories.iter().position(|a| &a.name == self.current.as_ref().unwrap()).unwrap();
        self.categories.get_mut(index)
    }
    fn get_category(&self, category: &str) -> Option<&Category> {
        self.categories.iter().find(|a| &a.name == category)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Category {
    name: String,
    path: String,
    saves: Vec<Save>,
    auto: Option<Save>
}
impl Category {
    fn get_save_from_name(&self, save: &str) -> Option<&Save> {
        self.saves.iter().find(|a| a.name == Some(String::from(save)))
    }
    fn get_auto_path(&self, data_dir: &Path) -> Box<Path> {
        data_dir.join(format!("{}_auto", self.name))
            .into_boxed_path()
    }
    fn get_save_from_arg(&self, arg: &String) -> Result<&Save, &'static str> {
        
        let save: Option<&Save>;
        let index = arg.parse::<usize>();

        if index.is_ok() {
            //if we've got a number load by index
            save = self.saves.get(index.unwrap());
            if save.is_none() { return Err("Invalid save index"); }
        } else if arg == "auto" {
            //if it's "auto" load autosave
            save = self.auto.as_ref();
            if save.is_none() { return Err("There is no current auto save"); }
        } else {
            //otherwise treat it as a name
            save = self.get_save_from_name(arg);
            if save.is_none() { return Err("Invalid save name"); }
        }
        
        Ok(save.unwrap())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Save {
    name: Option<String>,
    date: String,
}
impl Save {
    fn local_path(&self, category: &Category, data_dir: &Path) -> Box<Path> {
        data_dir.join(format!("{}_{}",
            category.name,
            category.saves.iter().position(|a| a.name == self.name).unwrap()))
            .into_boxed_path()
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    
    //input validation
    if args.len() <= 1 { help(); }
    println!("arg len {} {}", args.len(), args[1]);
    
    match &args[1][..] {
        "create" => create(args),
        "delete" => delete(args),
        "switch" => switch(args),
        "list" => list(args),
        
        "save" => save(args),
        "load" => load(args),
        "remove" => remove(args),

        _ => todo!("unrecognized")
    }
}

fn create(args: Vec<String>) {
    
    //ensure correct number of args
    if args.len() != 4 { todo!("incorrect number of args, 4 != {}", args.len()); }
    
    // let (name, path) = (&args[2], &args[3]);
    let name = &args[2];
    let path = canonicalize(PathBuf::from(&args[3]));

    //check existance of path
    if path.is_err() { todo!("say path doesn't exist") }
    
    let (data_dir, mut app) = load_data();
    
    //ensure category does not exist
    if app.categories.iter().any(|a| a.name == *name) { todo!("say cateogry already exists") }
    
    //create category
    app.categories.push(Category {
        name: name.clone(),
        path: String::from(path.unwrap().to_string_lossy()),
        saves: Vec::new(),
        auto: None
    });

    save_data(&data_dir, app);
}

fn delete(args: Vec<String>) {
    //ensure correct number of args
    if args.len() != 3 { todo!("print help method for delete and die")}
    
    let name = &args[2];
    
    let (data_dir, mut app) = load_data();

    let index = app.categories.iter().position(|a| a.name == *name)
        .unwrap_or_else(|| todo!("say category does not exist"));
    let category = &app.categories[index];
    
    //delete all saves
    for save in &category.saves[..] {
        fs::remove_dir_all(save.local_path(category, &data_dir))
            .expect("failed to remove file");
    }
    
    //remove from categores and update
    app.categories.remove(index);
    
    save_data(&data_dir, app);
}

fn switch(args: Vec<String>) {
    if args.len() != 3 { todo!("")}
    
    let (data_dir, mut app) = load_data();
    let category = app.get_category(&args[2][..]);
    
    if category.is_none() {
        println!("Category `{}` does not exists", args[2]);
        return;
    }

    app.current = Some(args[2].clone());
    
    save_data(&data_dir, app);
}

fn list(args: Vec<String>) {
    
    let (_, app) = load_data();
    
    match args.len() {
        2 => {
            list_categories(&app);
            list_saves(&app);
        }
        3 => {
            match &args[2][..] {
                "categories" => list_categories(&app),
                "saves" => list_saves(&app),
                _ => todo!("idk die or something")
            }
        }
        _ => todo!("improper number of arguments")
    }
}

fn list_saves(app: &App) {
    if app.current.is_none() {
        println!("No current category so no saves listed");
        return;
    }
    
    let category = app.current_category().unwrap();
    if category.saves.is_empty() {
        println!("No saves in {}", app.current.as_ref().unwrap());
        return;
    }

    println!("{} {}", "Saves in".bold().bright_green(), app.current.as_ref().unwrap());
    
    if category.auto.is_some() {
        let auto = category.auto.as_ref().unwrap();
        match &auto.name {
            Some(_) => println!("{} {}", auto.date, auto.name.as_ref().unwrap()),
            _ => println!("{}", auto.date)
        }
    }

    for (i, save) in category.saves.iter().enumerate() { 
        match &save.name {
            Some(_) => println!("{} {} {}", save.date, i, save.name.as_ref().unwrap()),
            _ => println!("{} {}", save.date, i)
        }
    }
}

fn list_categories(app: &App) {
    println!("{}", "Categories".bold().bright_green());
    for category in &app.categories[..] {
        println!("{}", category.name);
    }
}

//TODO input validation
fn save(args: Vec<String>) {
    
    if args.len() > 3 { todo!("do something idk"); }

    let (data_dir, mut app) = load_data();

    // let mut current = app.current_category_mut();
    if app.current.is_none() { todo! ("it brokey") }
    let current = app.current_category_mut().unwrap();

    //copy path into local
    let source_path = &current.path;
    let mut new_save = Save {
        name: None,
        date: String::from("date")
    };
    
    if args.len() == 3 { new_save.name = Some(args[3].clone()); }
    current.saves.push(new_save.clone());

    let local_path = new_save.local_path(current, &data_dir);
    
    if fs::metadata(&local_path).is_ok() { remove_all(&local_path); }
    copy_dir::copy_dir(source_path, local_path).unwrap();
    
    save_data(&data_dir, app);
       
}
fn load(args: Vec<String>) {
    
    if args.len() > 3 { todo!("do someting idk "); }
    
    let (data_dir, app) = load_data();
    
    if app.current.is_none() { todo!("no current category"); }
    let current = app.current_category().unwrap();
    
    if current.saves.len() == 0 { todo!("no saves in category"); }

    let save: Result<&Save, _>;
    let auto = current.get_auto_path(&data_dir);
    
    match args.len() {
        2 => save = Ok(current.saves.last().unwrap()),
        3 => save = current.get_save_from_arg(&args[2]),
        _ => todo!("unreachable")
    }
    
    save.unwrap();
    
    //copy to auto
    if fs::metadata(&auto).is_ok() { remove_all(&auto); }
    copy_dir::copy_dir(&current.path, current.get_auto_path(&data_dir)).unwrap();
    
    //copy from save to path
    remove_all(&current.path);
    copy_dir::copy_dir(save.unwrap().local_path(current, &data_dir), &current.path).unwrap();
    
    //save everything
    save_data(&data_dir, app);
    print!("successfully loaded save")
    
    
}
fn remove(args: Vec<String>) {
    
    if args.len() != 3 { todo!("args are bad") }
    
    let (data_dir, mut app) = load_data();

    if app.current.is_none() { todo!("no current category") }
    let current = app.current_category_mut().unwrap();
    
    let save = current.get_save_from_arg(&args[2]);
    save.unwrap();
    
    //remove the thing
    fs::remove_dir(save.unwrap().local_path(current, &data_dir)).unwrap();
    
    //remove it from the other thing
    let index = current.saves.iter().position(|a| a.name == save.unwrap().name).unwrap();
    current.saves.remove(index);
    
    save_data(&data_dir, app);
}

fn load_data() -> (Box<Path>, App) {
    
    //find/create data dir
    let data_dir = dirs::data_dir()
        .unwrap()
        .join("save-manager")
        .into_boxed_path();

    //if data dir doesn't exist, create it
    if !fs::metadata(&data_dir).is_ok() {
        DirBuilder::new()
            .recursive(true)
            .create(&data_dir)
            .unwrap();
    }
    
    let json_pathbuf = data_dir.join("data.json");
    let json_path = json_pathbuf.as_path();
    println!("path {:?}", json_pathbuf);
    
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

fn save_data(data_dir: &Path, app: App) {
    
    let data_json_string = serde_json::to_string(&app).unwrap();
    let json_pathbuf = data_dir.join("data.json");
    let json_path = json_pathbuf.as_path();

    let mut file: File = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(json_path.as_os_str()).unwrap();

    file.write_all(data_json_string.as_bytes()).unwrap();
}

fn remove_all<P: AsRef<Path>>(path: P) {
    match fs::metadata(&path).unwrap().is_dir() {
        true => fs::remove_dir(&path).unwrap(),
        false => fs::remove_file(&path).unwrap()
    };
}

fn help() {
    println!("this is a help message")
}

