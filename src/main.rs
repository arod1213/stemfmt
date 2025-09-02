mod name;
mod utils;

use clap::Parser;
use name::Instrument;
use std::{fs, io::Read};

use crate::{
    name::{get_new_name, instrument_from_str, write_name},
    utils::strip_name,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    folder: String,

    #[arg(short, long, default_value_t = false)]
    write: bool,
}

fn get_name(file_name: &str, instruments: &Vec<Instrument>) -> Option<String> {
    let mut file_str = file_name.to_string();
    strip_name(&mut file_str);
    let file_name = &file_str;

    let Some(inst) = instrument_from_str(&file_name, &instruments) else {
        return None;
    };
    get_new_name(file_name, &inst)
}

fn main() {
    let settings_file = dirs::document_dir().unwrap().join("srt/settings.json");
    let mut file = fs::File::open(settings_file).unwrap();

    let mut settings_str: String = String::new();
    file.read_to_string(&mut settings_str).unwrap();
    let instruments: Vec<Instrument> = serde_json::from_str(&settings_str).unwrap();

    let args = Args::parse();
    let dir = fs::read_dir(args.folder).unwrap();

    let mut info: Vec<(bool, (String, String))> = vec![];
    for d in dir {
        let entry = d.unwrap();
        let path = entry.path();
        let file_name = path.file_stem().unwrap().to_str().unwrap();
        let new_name = match get_name(&file_name, &instruments) {
            Some(v) => v,
            None => {
                info.push((false, (file_name.to_string(), String::new())));
                continue;
            }
        };
        if new_name == file_name {
            continue;
        }

        info.push((true, (file_name.to_string(), new_name.clone())));
        let mut new_path = path.clone();
        let new_name = match path.extension() {
            Some(v) => {
                let ext = v.to_str().unwrap();
                let name = format!("{}.{}", new_name, ext);
                name.to_string()
            }
            None => new_name.to_string(),
        };

        if !args.write {
            continue;
        };
        new_path.set_file_name(&new_name);
        write_name(&path, &mut new_path);
    }

    let descriptor: String = match args.write {
        true => "Wrote".to_string(),
        _ => "Would rename".to_string(),
    };
    info.iter()
        .filter(|(k, _)| *k == true)
        .for_each(|(_, (a, b))| {
            println!("{}: {} -> {}", descriptor, a, b);
            ()
        });
    info.iter()
        .filter(|(k, _)| *k == false)
        .for_each(|(_, (a, _))| {
            println!("invalid name {}", a);
            ()
        });
}
