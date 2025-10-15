use std::{ env, ffi::OsStr, fs::{ self, File }, io::Read, path::{ Path, PathBuf } };
use clap::Parser;
use colored::{ Color, Colorize };
use glob::glob;
use regex::Regex;
use serde_json::{ Map, Value };
mod cli;
use crate::cli::Cli;

fn read_file(file_path: &OsStr) -> anyhow::Result<String> {
    let path_str = Path::new(file_path);
    let mut file = File::open(path_str)?;
    let mut file_data = String::new();
    file.read_to_string(&mut file_data)?;
    Ok(file_data)
}
fn println_tag(tag: &str, info: &str, color: colored::Color) {
    println!(
        "{} -> {}",
        format!(" {tag} ").bold().color(colored::Color::BrightBlack).on_color(color),
        info.trim().color(color)
    )
}
fn write_ts_file(path: &PathBuf, out: String, ext: &str) -> anyhow::Result<bool> {
    let out_path = path.as_path().with_extension(ext);
    let out_path_copy = out_path.to_owned();
    let regex = Regex::new(r"(\/\/)+")?;
    let prefix_str = String::from(
        r#"
// AUTO-GENERATED FILE
// PLEASE DO NOT EDIT UNLESS YOU KNOW WHAT YOU'RE DOING!
// OMIT FROM LINTER!
// :)"#
    );
    (match ext {
        "lua" | "luau" =>
            fs::write(
                out_path,
                format!(
                    "{} \n return {}",
                    regex.replace_all(&prefix_str, "--").to_string(),
                    json2lua::parse(&out)?
                )
            ),
        "js" | "ts" => fs::write(out_path, format!("{prefix_str} \n export default {out}")),
        _ => fs::write(out_path, out),
    })?;

    println_tag("FILE SAVED", out_path_copy.to_str().unwrap(), Color::Cyan);
    Ok(true)
}
fn to_map(path: PathBuf) -> anyhow::Result<Option<Map<String, Value>>> {
    if let Some(ext) = path.extension() {
        let contents = read_file(path.as_os_str())?;
        let cont_as_str = contents.as_str();

        let json: anyhow::Result<Option<Map<String, Value>>> = match
            ext.to_str().expect("FAILED TO CONVERT VALUE")
        {
            "yml" | "yaml" => {
                match serde_yaml::from_str::<Map<String, Value>>(cont_as_str) {
                    Ok(x) => Ok(Some(x)),
                    Err(x) => Err(anyhow::Error::new(x)),
                }
            }

            "json" =>
                match serde_json::from_str::<Map<String, Value>>(cont_as_str) {
                    Ok(x) => Ok(Some(x)),
                    Err(x) => Err(anyhow::Error::new(x)),
                }
            "toml" =>
                match toml::from_str(contents.as_str()) {
                    Ok(x) => Ok(Some(x)),
                    Err(x) => Err(anyhow::Error::new(x)),
                }
            _ => Err(anyhow::Error::msg("INVALID FILE EXTENSION")),
        };
        return match json {
            Ok(option) => Ok(option),
            Err(error) => Err(error),
        };
    }
    return Err(anyhow::Error::msg("FAILED TO GET EXPRESSION"));
}

fn iter_n_load<'map>(
    path: &Path,
    result_map: &'map mut serde_json::Map<String, serde_json::Value>,
    omit: &Path
) -> &'map mut serde_json::Map<String, serde_json::Value> {
    let canon_path_in = path.canonicalize().unwrap();
    let canon_path_omit = omit.canonicalize().unwrap();
    if !canon_path_in.starts_with(canon_path_omit.to_owned()) {
        panic!("PATH MISMATCH?");
    }
    let canon_path_omit_iter: Vec<_> = canon_path_omit.iter().collect();
    let total = canon_path_in.iter().count() - 1;
    let mut counter = 0;
    let comp_iter = canon_path_in.iter().map(|path_comp| {
        let some_val = Some(path_comp.to_string_lossy().to_string());
        let res = match canon_path_omit_iter.get(counter) {
            Some(x) => {
                if *x != path_comp {
                    return some_val;
                }
                None
            }
            None => some_val,
        };
        counter += 1;
        res
    });
    comp_iter.take(total).fold(result_map, |result_map, path_component| {
        let Some(path_comp_deref) = path_component else {
            return result_map;
        };
        match result_map.entry(&path_comp_deref) {
            serde_json::map::Entry::Vacant(slot) => {
                slot.insert(serde_json::Value::Object(Default::default()));
            }
            serde_json::map::Entry::Occupied(slot) => {
                if !slot.get().is_object() {
                    return result_map;
                }
            }
        }
        result_map[&path_comp_deref].as_object_mut().unwrap()
    })
}
fn main() {
    let args = Cli::parse();
    let (rel, files) = {
        let path = args.path_in.as_path();
        if !path.is_dir() || path.is_file() {
            panic!("PATH IS NOT A FOLDER!");
        }
        let in_path_rel = path.to_owned();
        let path = path.join("**/*.*");
        (in_path_rel, glob(path.to_str().expect("FAILED TO PARSE PATH")))
    };
    let mut r: Map<String, Value> = Map::new();
    for file in files.expect("FAILED TO GRAB FILES") {
        match file {
            Ok(path) => {
                let cloned_path = path.to_owned();
                let fin = iter_n_load(&cloned_path.as_path(), &mut r, &rel.as_path());
                let data = match to_map(cloned_path) {
                    Ok(x) => {
                        println_tag("SUCCESSFULLY PARSED", path.to_str().unwrap(), Color::Green);
                        x.unwrap()
                    }
                    Err(x) => {
                        println_tag(
                            "FAILED TO PARSE FILE",
                            format!(
                                "{} -x-> {}",
                                path.to_str().unwrap(),
                                x.to_string().as_str()
                            ).as_str(),
                            Color::Red
                        );
                        continue;
                    }
                };
                fin.insert(
                    path.file_stem().unwrap().to_string_lossy().to_string(),
                    Value::Object(data)
                );
            }
            Err(e) => panic!("{e}"),
        }
    }
    let out = serde_json::to_string_pretty(&r).expect("FAILED TO STRINGIFY MAP");
    let file_ext = args.output_ext.as_str();
    write_ts_file(&args.path_out, out, file_ext).expect("FAILED TO WRITE TO FILE");
}
