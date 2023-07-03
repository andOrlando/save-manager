use std::path::{Path, PathBuf};
use std::clone::Clone;
use serde::{Deserialize, Serialize};

pub type AppError = Result<(), &'static str>;

#[derive(Serialize, Deserialize)]
pub struct App {
    pub current: Option<String>,
    pub categories: Vec<Category>
}
impl App {
    pub fn current_category(&self) -> Result<&Category, &'static str> {
        if self.current.is_none() { return Err("No current category"); }
        Ok(self.get_category(self.current.as_ref().unwrap())?)
    }
    pub fn current_category_mut(&mut self) -> Result<&mut Category, &'static str> {
        if self.current.is_none() { return Err("No current category"); }
        let index = self.get_category_index(self.current.as_ref().unwrap())?;
        Ok(self.categories.get_mut(index).unwrap())
        
    }
    pub fn get_category(&self, name: &str) -> Result<&Category, &'static str> {
        let category = self.categories.iter().find(|a| &a.name == name);
        if category.is_none() { return Err("Category does not exist") }
        Ok(category.unwrap())
    }
    pub fn get_category_index(&self, name: &str) -> Result<usize, &'static str> {
        let index = self.categories.iter().position(|a| &a.name == name);
        if index.is_none() { return Err("Category does not exist") }
        Ok(index.unwrap())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub paths: Vec<String>,
    pub saves: Vec<Save>,
    pub auto: Option<String>, //auto is just the date of autosave
    pub max: usize
}
impl Category {
    #[allow(dead_code)]
    pub fn get_save(&self, name: &str) -> Result<&Save, &'static str> {
        let save = self.saves.iter().find(|a| a.name == Some(String::from(name)));
        if save.is_none() { return Err("Save by this name does not exist") }
        Ok(save.unwrap())
    }
    pub fn get_save_index(&self, name: &str) -> Result<usize, &'static str> {
        let index = self.saves.iter().position(|a| a.name == Some(String::from(name)));
        if index.is_none() { return Err("Save with this index does not exist") }
        Ok(index.unwrap())
    }
    pub fn get_auto_paths(&self, file: &Path) -> Vec<PathBuf> {
        let mut i=0;
        self.paths.iter().map(|_| { i += 1; file.join(format!("{}_{}_auto", self.name, i)) })
            .collect::<Vec<_>>()
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Save {
    pub name: Option<String>,
    pub real_index: usize,
    pub date: String,
}
impl Save {
    pub fn get_paths(&self, category: &Category, file: &Path) -> Vec<PathBuf> {
        (1..=category.paths.len()).map(|i| file.join(format!("{}_{}_{}", category.name, i, self.real_index)))
            .collect::<Vec<_>>()
    }
}