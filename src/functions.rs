use crate::cli::ListCategory;
use crate::data::{load_data, save_data};
use crate::types::{App, Category, Save, AppError};

use chrono::Utc;
use colored::Colorize;
use copy_dir::copy_dir;
use itertools::izip;
use std::env::temp_dir;
use std::fs::{remove_dir_all, canonicalize, metadata, remove_file, rename};
use std::path::{PathBuf, Path};

pub fn create(name: &String, paths: &Vec<PathBuf>) -> AppError {
    
    //ensure paths exist and are nonempty
    if paths.is_empty() { return Err("You must provide at least one path") }
    for path in paths { if !path.exists() { return Err("Path does not exist") }}

    let (data_dir, mut app) = load_data();
    
    //ensure category does not exist
    if app.get_category(name).is_ok() { return Err("Category already exists") }
    
    // create category
    app.categories.push(Category {
        name: name.clone(),
        saves: Vec::new(),
        auto: None,
        max: 0,
        paths: paths.iter()
            .map(|path| String::from(canonicalize(path).unwrap().to_string_lossy()))
            .collect::<Vec<_>>(),
    });

    println!("Created category {}", name.bold().bright_green());
    save_data(&data_dir, app);
    Ok(())
}

pub fn delete(name: &String) -> AppError {

    let (file, mut app) = load_data();

    let index = app.get_category_index(name)?;
    let category = &app.categories[index];
    
    //delete all saves
    for save in &category.saves[..] {
        for path in save.get_paths(category, &file) {
            remove_all(path).unwrap();
        }
    }
    
    //remove autosave if autosave exists
    if category.auto.is_some() {
        for path in category.get_auto_paths(&file) {
            remove_all(path).unwrap();
        }
    }

    //remove from categores and update
    app.categories.remove(index);
    
    //if current is category, set current to none
    if app.current.as_ref() == Some(name) { app.current = None; }
    
    println!("Deleted save {}", name.bold().bright_green());
    save_data(&file, app);
    Ok(())
}

pub fn update_name(name: &String) -> AppError {
    
    let (file, mut app) = load_data();

    //ensure category does not exist
    if app.get_category(name).is_ok() { return Err("Category already exists") }
    
    let category = app.current_category_mut()?;
    let oldname = category.name.clone();
    
    //old paths to move to new paths later
    let oldsaves = category.saves.iter()
        .map(|s| s.get_paths(category, &file))
        .collect::<Vec<_>>();
    let oldautos = category.get_auto_paths(&file);
    
    //update category and current
    category.name = name.clone();
    
    //rename literally everything
    let newsaves = category.saves.iter()
        .map(|s| s.get_paths(category, &file))
        .collect::<Vec<_>>();
    let newautos = category.get_auto_paths(&file);
    
    //now rename everything
    for (oldsave, newsave) in izip!(oldsaves, newsaves) {
        for (old, new) in izip!(oldsave, newsave) {
            rename(old, new).unwrap();
        }
    }
    for (old, new) in izip!(oldautos, newautos) {
        rename(old, new).unwrap();
    }
    
    
    //rename current
    app.current = Some(name.clone());
    
    println!("Changed name from {} to {}",
        oldname.bold().bright_green(),
        name.bold().bright_green());
    save_data(&file, app);
    Ok(())
}

pub fn switch(name: &String) -> AppError {
    
    let (data_dir, mut app) = load_data();

    app.get_category(name)?; //ensure category exists
    app.current = Some(name.clone());
    
    println!("Switched active category to {}", name.bold().bright_green());
    save_data(&data_dir, app);
    Ok(())
}

pub fn list(category: &Option<ListCategory>) -> AppError {
    
    let (_, app) = load_data();

    match category {
        Some(ListCategory::Saves) => list_categories(&app),
        Some(ListCategory::Versions) => list_saves(&app),
        None => {
            list_categories(&app);
            println!("");
            list_saves(&app);
        }
    }
    
    Ok(())
}

fn list_saves(app: &App) {
    if app.current.is_none() { println!("No current save :/"); return; }
    
    let category = app.current_category().unwrap();
    println!("{}", format!("Files in {}", category.name).bold().bright_green());
    
    for path in &category.paths {
        println!("{}", path);
    }
    println!("");
    
    if category.saves.is_empty() && category.auto.is_none() {
        println!("No versions in {}", app.current.as_ref().unwrap()); return
    }
    
    let digits = { let len=category.saves.len() as f64; len.log10() as usize + 1 };

    println!("{}", "Revisions".bold().bright_green());
    if let Some(date) = &category.auto { println!("{} {:digits$} auto", date, ""); }

    for (i, save) in category.saves.iter().enumerate() { 
        match &save.name {
            Some(name) => println!("{} {:digits$} {name}", save.date, i),
            _ => println!("{} {:digits$}", save.date, i)
        }
    }
}

fn list_categories(app: &App) {
    if app.categories.is_empty() { println!("No saves yet :/"); return; }
    
    println!("{}", "Categories".bold().bright_green());
    for category in &app.categories[..] {
        if app.current.as_ref() == Some(&category.name) { println!("->{}", category.name); }
        else { println!("  {}", category.name); }
    }
}

pub fn save(name: &Option<String>) -> AppError {

    let (file, mut app) = load_data();
    let current = app.current_category_mut()?;
    
    //input validation
    if let Some(name) = name {
        if name.parse::<usize>().is_ok() { return Err("Save name must not be numeric") }
        if name == "auto" { return Err("Save must not be named `auto`") }
        if current.get_save(name).is_some() { return Err("Save already exists"); }
    } 

    //copy path into local
    let new_save = Save {
        real_index: current.max,
        name: name.clone(),
        date: format!("{}", Utc::now().format("%m/%d %H:%M")),
    };
    
    current.max += 1;
    current.saves.push(new_save.clone());

    // copy_dir::copy_dir(source_path, new_save.path).unwrap();
    for (source, local) in izip!(&current.paths, new_save.get_paths(current, &file)) {
        copy_dir(source, local).unwrap();
    }

    println!("Saved version {} in {}",
        name.as_ref().unwrap_or(&(current.saves.len()-1).to_string()).bold().green(),
        current.name.bold().bright_green());
    save_data(&file, app);
    Ok(())
       
}

pub fn load_name(current: &mut Category, file: &Path, name: &str) -> AppError {
    
    //get index from name
    let index = if let Ok(index) = name.parse::<usize>() { index }
    else { current.get_save_index(name)? };
    
    let save = &current.saves[index];

    //every vector we need
    let locals = save.get_paths(current, file);
    let sources = &current.paths;
    let autos = current.get_auto_paths(file);
        
    //update auto
    if current.auto.is_some() {
        for auto in &autos {
            remove_all(auto)?;
        }
    }
    
    //move source to auto and copy local to source
    for (local, source, auto) in izip!(locals, sources, autos) {
        rename(&source, &auto).unwrap();
        copy_dir(&local, &source).unwrap();
    }

    current.auto = Some(format!("{}", Utc::now().format("%_m/%d %k:%M")));
    println!("Loaded version {} in {}",
        save.name.as_ref().unwrap_or(&index.to_string()).bold().green(),
        current.name.bold().bright_green());
    Ok(())
}
pub fn load_auto(current: &mut Category, file: &Path) -> AppError {
    
    if current.auto.is_none() { return Err("No autosave in current category") }
    
    let sources = &current.paths;
    let autos = current.get_auto_paths(&file);
    let _autos = autos.iter().map(|path| temp_dir().join(path.file_name().unwrap()))
        .collect::<Vec<_>>();
    
    //move auto to _auto, save to auto, _auto to save
    for (source, auto, _auto) in izip!(sources, autos, _autos) {
        rename(&auto, &_auto).unwrap();
        rename(&source, &auto).unwrap();
        rename(&_auto, &source).unwrap();
    }
    
    current.auto = Some(format!("{}", Utc::now().format("%_m/%d %k:%M")));
    println!("Loaded {} in {}",
        "autosave".bold().green(),
        current.name.bold().bright_green());
    Ok(())
}

pub fn load(name: &Option<String>) -> AppError {
    
    let (file, mut app) = load_data();
    let current = app.current_category_mut()?;
    if current.saves.is_empty() { return Err("No saves in current category") }

    match name.as_deref() {
        Some("auto") => load_auto(current, &file)?,
        Some(name) => load_name(current, &file, name)?,
        _ => load_name(current, &file, &(current.saves.len()-1).to_string())?
    }
    
    save_data(&file, app);
    Ok(())
}

pub fn overwrite(name: &String) -> AppError {
    
    let (file, mut app) = load_data();

    let current = app.current_category_mut()?;
    
    let index = if let Ok(index) = name.parse::<usize>() {
        if current.saves.get(index).is_some() { index }
        else { return Err("Save with this index does not exist") }

    } else { current.get_save_index(&name)? };

    let save = &current.saves[index];
    let filename = save.name.clone().unwrap_or(index.to_string());
    
    let locals = save.get_paths(current, &file);
    let sources = &current.paths;
    let autos = current.get_auto_paths(&file);
    
    for (local, source, auto) in izip!(locals, sources, autos) {
        //move local to auto, copy source to local
        if current.auto.is_some() { remove_all(&auto)?; }
        rename(&local, &auto).unwrap();
        copy_dir(&source, &local).unwrap();
    }
    
    //update date
    let mut_save = current.saves.get_mut(index).unwrap();
    mut_save.date = format!("{}", Utc::now().format("%m/%d %H:%M"));
    
    println!("Overwrote version {} in {}",
        filename.bold().green(),
        current.name.bold().bright_green());
    save_data(&file, app);
    Ok(())
}

pub fn remove(name: &String) -> AppError {
    
    let (file, mut app) = load_data();

    let current = app.current_category_mut()?;
    
    let index = if let Ok(index) = name.parse::<usize>() {
        if current.saves.get(index).is_some() { index }
        else { return Err("Save with this index does not exist") }

    } else { current.get_save_index(&name)? };

    let save = &current.saves[index];
    let filename = save.name.clone(); //save is removed so we keep name
    
    //remove the things
    for path in save.get_paths(current, &file) {
        remove_all(&path).unwrap();
    }
    
    //remove it from the other thing
    current.saves.remove(index);

    println!("Removed version {} in {}",
        filename.unwrap_or(index.to_string()).bold().green(),
        current.name.bold().bright_green());
    save_data(&file, app);
    Ok(())
}


fn remove_all<P: AsRef<Path>>(path: P) -> AppError {
    let res = match metadata(&path).unwrap().is_dir() {
        true => remove_dir_all(&path),
        false => remove_file(&path)
    };
    
    if res.is_ok() { return Ok(()) }
    Err("Failed to remove save")
}
