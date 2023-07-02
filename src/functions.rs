use colored::Colorize;

use crate::cli::ListCategory;
use crate::data::{load_data, save_data};
use crate::types::{App, Category, Save, AppError};

use std::path::{PathBuf, Path};
use std::fs::{remove_dir_all, canonicalize, metadata, remove_file};

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
        auto: None
    });

    println!("Created category {}", name.bold().bright_green());
    save_data(&data_dir, app);
    Ok(())
}

pub fn delete(name: &String) -> AppError {

    let (data_dir, mut app) = load_data();

    let index = app.get_category_index(name)?;

    //delete all saves
    for save in &app.categories[index].saves[..] {
        remove_all(save.local_path(&app.categories[index], &data_dir))?;
    }

    //remove from categores and update
    app.categories.remove(index);
    
    println!("Deleted category {}", name.bold().bright_green());
    save_data(&data_dir, app);
    Ok(())
}

pub fn switch(name: &String) -> AppError {
    
    let (data_dir, mut app) = load_data();

    app.get_category(name)?; //ensure category exists
    app.current = Some(name.clone());
    
    print!("Switched active category to {}", name.bold().bright_green());
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
            list_saves(&app);
        }
    }
    
    Ok(())
}

fn list_saves(app: &App) {
    if app.current.is_none() { println!("No current category so no saves listed"); return }
    
    let category = app.current_category().unwrap();
    if category.saves.is_empty() { println!("No saves in {}", app.current.as_ref().unwrap()); return }

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

pub fn save(name: &Option<String>) -> AppError {
    
    //input validation
    if let Some(_name) = name {
        if _name.parse::<usize>().is_ok() { return Err("Save name must not be numeric") }
        if _name == "auto" { return Err("Save must not be named `auto`") }
    } 
    
    let (data_dir, mut app) = load_data();

    let current = app.current_category_mut()?;

    //copy path into local
    let source_path = &current.path;
    let new_save = Save {
        name: name.clone(),
        date: String::from("date")
    };
    
    current.saves.push(new_save.clone());

    let local_path = new_save.local_path(current, &data_dir);
    
    if metadata(&local_path).is_ok() { remove_all(&local_path)?; }
    copy_dir::copy_dir(source_path, local_path).unwrap();
    

    if let Some(_name) = name { println!("Saved {} in {}", _name, &current.name) }
    else { println!("Saved version in {}", &current.name) }
    save_data(&data_dir, app);
    Ok(())
       
}

pub fn load(name: &Option<String>) -> AppError {
    
    let (data_dir, app) = load_data();
    
    let current = app.current_category()?;
    if current.saves.is_empty() { return Err("No saves in current category") }

    let auto = current.get_auto_path(&data_dir);
    let name_numeric = if let Some(name) = name { name.parse::<usize>().ok() } else { None };
    
    let save = if let Some(name) = name {

        if let Some(index) = name_numeric {
            if let Some(save) = current.saves.get(index) { save }
            else { return Err("Save with this index does not exist") }

        } else if name == "auto" {
            if let Some(auto) = current.auto.as_ref() { auto }
            else { return Err("There is no current auto save") }

        } else { current.get_save(name)? }

    } else { current.saves.last().unwrap() };
    
    //copy to auto
    if metadata(&auto).is_ok() { remove_all(&auto)?; }
    copy_dir::copy_dir(&current.path, current.get_auto_path(&data_dir)).unwrap();
    
    //copy from save to path
    remove_all(&current.path)?;
    copy_dir::copy_dir(save.local_path(current, &data_dir), &current.path).unwrap();
    
    if let Some(index) = name_numeric {
        println!("Loaded version {} in {}",
            index.to_string().green(),
            current.name.to_string().bold().bright_green());
    } else if let Some("auto") = name.as_deref() {
        println!("Loaded {} in {}",
            "autosave".bold().green(),
            current.name.bold().bright_green());
    } else if let Some(name) = name {
        println!("Loaded version {} in {}",
            name.bold().green(),
            current.name.bold().bright_green());
    } else {
        println!("Loaded version {} in {}",
            current.saves.iter().position(|a| a == save).unwrap().to_string().bold().green(),
            current.name.bold().bright_green());
    }
    save_data(&data_dir, app);
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
    remove_all(save.local_path(current, &data_dir))?;
    
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
