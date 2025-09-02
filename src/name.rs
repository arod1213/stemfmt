use crate::utils::get_regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Instrument<'a> {
    pub prefix: &'a str,
    // pub super_types: Vec<&'a str>,
    pub keywords: Vec<Keyword>,
    pub descriptors: Vec<&'a str>,
    pub invalid_names: Vec<&'a str>,
}

pub fn instrument_from_str<'a>(
    name: &'a str,
    instruments: &'a Vec<Instrument<'a>>,
) -> Option<&'a Instrument<'a>> {
    instruments.iter().find(|v| {
        let super_strs: Vec<&str> = v.keywords.iter().flat_map(|s| s.as_vec()).collect();
        let reg = get_regex(&super_strs);
        if reg.find(name) == None {
            return false;
        }
        true
    })
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Keyword {
    name: String,
    aliases: Vec<String>,
}

impl Keyword {
    // pub fn contains(&self, name: &str) -> bool {
    //     &self.name == name || self.aliases.contains(&name.to_string())
    // }

    pub fn as_vec<'a>(&'a self) -> Vec<&'a str> {
        let mut result: Vec<&str> = self.aliases.iter().map(|s| s.as_str()).collect();
        result.push(self.name.as_str());
        result
    }
}

pub fn get_new_name(name: &str, inst: &Instrument) -> Option<String> {
    let keywords: Vec<&str> = inst.keywords.iter().flat_map(|x| x.as_vec()).collect();
    let keyword_regex = get_regex(&keywords);
    let keyword_matches: Vec<&str> = keyword_regex.find_iter(name).map(|x| x.as_str()).collect();
    if keyword_matches.len() == 0 {
        return None;
    }

    let rest_str = keyword_regex
        .replace_all(name, "")
        .replace("  ", " ") // single space max
        .to_string();

    let main_keywords: Vec<String> = inst.keywords.iter().map(|x| x.name.clone()).collect();
    let name_parts: HashSet<&str> = keyword_matches
        .iter()
        .filter(|x| !inst.invalid_names.contains(x) && !x.is_empty())
        .map(|x| match main_keywords.contains(&x.to_string()) {
            true => x,
            _ => keyword_from_alias(&x, inst).unwrap(),
        })
        .collect();
    let mut name_parts = Vec::from_iter(name_parts);

    let descriptors_regex = get_regex(&inst.descriptors);
    let mut descriptors_matches: Vec<&str> = descriptors_regex
        .find_iter(&rest_str)
        .filter(|x| !x.is_empty())
        .map(|x| x.as_str())
        .collect();

    let rest_str = descriptors_regex.replace_all(&rest_str, "").to_string();
    name_parts.append(&mut descriptors_matches);
    if !rest_str.is_empty() {
        name_parts.push(&rest_str.trim());
    }

    Some(
        format!("{} - {}", inst.prefix, name_parts.join(" "))
            .trim()
            .to_string(),
    )
}

fn keyword_from_alias<'a>(alias: &str, instrument: &'a Instrument) -> Option<&'a str> {
    match instrument
        .keywords
        .iter()
        .find(|k| k.aliases.contains(&alias.to_string()))
    {
        Some(x) => Some(&x.name),
        None => None,
    }
}

// IMPLEMENT HASh CHECK TO NOT ITERATE NUM OVER SAME FILE
fn iterate_path(path: &mut PathBuf) {
    if !path.exists() {
        return;
    }
    let parent = path.parent().unwrap_or(Path::new("")).to_path_buf();
    let file_stem = path
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("file");
    let ext = path.extension().unwrap_or(OsStr::new("")).to_str().unwrap();

    let mut count = 0;
    loop {
        count += 1;
        let new_stem = match ext.is_empty() {
            true => format!("{} {}", file_stem, count),
            _ => format!("{} {}.{}", file_stem, count, ext),
        };
        let new_path = parent.join(new_stem);
        if !new_path.exists() {
            *path = new_path;
            return;
        }
        count += 1;
    }
}
pub fn write_name(old_path: &PathBuf, new_path: &mut PathBuf) {
    iterate_path(new_path);
    let _ = fs::rename(old_path, new_path);
}
//
mod test {

    #[cfg(test)]
    use super::*;

    #[test]
    fn test_rename() {
        use std::io::Read;

        let mut file = fs::File::open("settings.json").unwrap();
        let mut settings_str: String = String::new();
        file.read_to_string(&mut settings_str).unwrap();
        let instruments: Vec<Instrument> = serde_json::from_str(&settings_str).unwrap();

        let drums = &instruments[0];

        let old_name = "tamb tambourine vintage";
        let new_name = get_new_name(&old_name, drums).unwrap();
        assert_eq!(new_name, "drm - tamb vintage".to_string());

        let old_name = "drum beater kik drm";
        let new_name = get_new_name(&old_name, drums).unwrap();
        assert_eq!(new_name, "drm - kick beater".to_string());

        let old_name = "cabasa vintage break";
        let new_name = get_new_name(&old_name, drums).unwrap();
        assert_eq!(new_name, "drm - shaker break vintage".to_string());

        let old_name = "sidestick 1";
        let new_name = get_new_name(&old_name, drums).unwrap();
        assert_eq!(new_name, "drm - sidestick 1".to_string());
    }
}
