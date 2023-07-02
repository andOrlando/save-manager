use colored::Colorize;

use crate::cli::ListCategory;
use crate::data::{load_data, save_data};
use crate::types::{App, Category, Save, AppError};

use std::path::{PathBuf, Path};
use std::fs::{remove_dir_all, canonicalize, metadata, remove_file, rename};
use chrono::Utc;

pub fn create(name: &String, path: &PathBuf) -> AppError {

    if !path.exists() { return Err("Path does not exist") }

    let (data_dir, mut app) = load_data();
    
    //ensure category does not exist
    if app.get_category(name).is_ok() { return Err("Category already exists") }
    
    // create category
    app.categories.push(Category {
        name: name.clone(),
        path: String::from(canonicalize(path).unwrap().to_string_lossy()),
        saves: Vec::new(),
        auto: None,
        max: 0
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
    for save in &category.saves[..] { remove_all(&save.path)?; }
    
    //remove autosave if autosave exists
    if category.auto.is_some() { remove_all(category.get_auto_path(&file))?; }

    //remove from categores and update
    app.categories.remove(index);
    
    //if current is category, set current to none
    if app.current.as_ref() == Some(name) { app.current = None; }
    
    println!("Deleted category {}", name.bold().bright_green());
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
    if app.current.is_none() { println!("No current category so no versions listed"); return; }
    
    let category = app.current_category().unwrap();
    if category.saves.is_empty() && category.auto.is_none() {
        println!("No versions in {}", app.current.as_ref().unwrap()); return
    }

    println!("{}", format!("Versions in {}", app.current.as_ref().unwrap()).bold().bright_green());
    
    if let Some(date) = &category.auto { println!("{} auto", date); }

    for (i, save) in category.saves.iter().enumerate() { 
        match &save.name {
            Some(name) => println!("{} {} {name}", save.date, i),
            _ => println!("{} {}", save.date, i)
        }
    }
}

fn list_categories(app: &App) {
    if app.categories.is_empty() { println!("No categories yet :/"); return; }
    
    println!("{}", "Categories".bold().bright_green());
    for category in &app.categories[..] {
        if app.current.as_ref() == Some(&category.name) { println!("->{}", category.name); }
        else { println!("  {}", category.name); }
    }
}

pub fn save(name: &Option<String>) -> AppError {
    
    //input validation
    if let Some(name) = name {
        if name.parse::<usize>().is_ok() { return Err("Save name must not be numeric") }
        if name == "auto" { return Err("Save must not be named `auto`") }
    } 
    
    let (file, mut app) = load_data();

    let current = app.current_category_mut()?;

    //copy path into local
    let source_path = &current.path;
    let new_save = Save {
        path: String::from(file.join(format!("{}_{}", current.name, current.max)).to_string_lossy()),
        name: name.clone(),
        date: format!("{}", Utc::now().format("%_m/%d %k:%M")),
    };
    
    current.max += 1;
    current.saves.push(new_save.clone());

    copy_dir::copy_dir(source_path, new_save.path).unwrap();

    if let Some(name) = name {
        println!("Saved {} in {}",
            name.bold().green(),
            &current.name.bold().bright_green());
    } else {
        println!("Saved version in {}",
            &current.name.bold().bright_green());
    }
    save_data(&file, app);
    Ok(())
       
}

pub fn load_name(current: &mut Category, file: &Path, name: &str) -> AppError {
    
    //get index from name
    let index = if let Ok(index) = name.parse::<usize>() { index }
    else { current.get_save_index(name)? };
    
    let save = &current.saves[index];
    let auto = current.get_auto_path(&file);
    
    //update auto
    if metadata(&auto).is_ok() { remove_all(&auto)?; }
    rename(&current.path, &auto).unwrap();

    //copy from save to path
    copy_dir::copy_dir(&save.path, &current.path).unwrap();

    current.auto = Some(format!("{}", Utc::now().format("%_m/%d %k:%M")));
    println!("Loaded version {} in {}",
        name.bold().green(),
        current.name.bold().bright_green());
    Ok(())
}
pub fn load_auto(current: &mut Category, file: &Path) -> AppError {
    
    if current.auto.is_none() { return Err("No autosave in current category") }
    
    let auto = current.get_auto_path(&file);
    let _auto = auto.with_file_name("_auto");
    
    //move auto to _auto, save to auto, _auto to save
    rename(&auto, &_auto).unwrap();
    rename(&current.path, &auto).unwrap();
    rename(&_auto, &current.path).unwrap();
    
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

pub fn remove(name: &String) -> AppError {
    
    let (data_dir, mut app) = load_data();

    let current = app.current_category_mut()?;
    
    let index = if let Ok(index) = name.parse::<usize>() {
        if current.saves.get(index).is_some() { index }
        else { return Err("Save with this index does not exist") }

    } else { current.get_save_index(&name)? };

    let save = &current.saves[index];
    
    //remove the thing
    remove_all(&save.path)?;
    
    //remove it from the other thing
    current.saves.remove(index);

    println!("Removed version {} in {}",
        name.bold().green(),
        current.name.bold().bright_green());
    save_data(&data_dir, app);
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
