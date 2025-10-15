use std::{ env, ffi::OsStr, fs::{ self, File }, io::Read, path::{ Path, PathBuf } };
use colored::{ Color, Colorize };
use glob::glob;
use serde_json::{ Map, Value };
use crate::OUTDATED_parse_args::{ ParsedArgs, Values };
mod OUTDATED_parse_args;
mod cli;
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
fn write_ts_file(path: &str, out: String, ext: &str) -> anyhow::Result<bool> {
    let out_path = Path::new(&path).with_extension(ext);
    let out_path_copy = out_path.to_owned();
    let prefix_str = String::from(
        r#"
// AUTO-GENERATED FILE
// PLEASE DO NOT EDIT UNLESS YOU KNOW WHAT YOU'RE DOING!
// OMIT FROM LINTER!
// :)"#
    );
    (match ext {
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
    omit: &str
) -> &'map mut serde_json::Map<String, serde_json::Value> {
    let omit_components: Vec<_> = Path::new(omit).iter().collect();
    let count = path.iter().count() - 1;
    let mut counter: usize = 0;
    let new_iter = path.iter().map(|str| {
        let res = match omit_components.get(counter) {
            Some(x) => {
                if *x != str {
                    return Some(str.to_string_lossy().to_string());
                }
                None
            }
            None => Some(str.to_string_lossy().to_string()),
        };
        counter += 1;
        res
    });

    new_iter.take(count).fold(result_map, |result_map, path_component| {
        //  println_tag("PATH COMP:", format!("{:?}", path_component).as_str(), Color::Yellow);
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
    let iter = env::args();
    let parsed = ParsedArgs::parse(iter);

    if parsed.has_tag(&["--debug"]) {
        println!("{}", parsed.to_string().yellow());
    }
    let dir_in = parsed.unordered.get(0).expect("NO ARG0 VALUE SUPPLIED");
    let dir_out = parsed.unordered.get(1).expect("NO ARG1 VALUE SUPPLIED");
    let Values::String(dir_out_str) = dir_out else { panic!("ARG1 IS OF INVALID TYPE") };
    let (rel, files) = if let Values::String(dir_in_str) = dir_in {
        let path = Path::new(dir_in_str);
        if !path.is_dir() || path.is_file() {
            panic!("PATH IS NOT A FOLDER!");
        }
        let path = path.join("**/*.*");
        (dir_in_str, glob(path.to_str().expect("FAILED TO PARSE PATH")))
    } else {
        panic!("ARG0 IS OF INVALID TYPE");
    };
    let mut r: Map<String, Value> = Map::new();
    for file in files.expect("FAILED TO GRAB FILES") {
        match file {
            Ok(path) => {
                let cloned_path = path.to_owned();
                let fin = iter_n_load(&cloned_path.as_path(), &mut r, &rel);
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
    let file_ext = parsed.get_value(&["-x", "-ext", "-extension"]);
    if let Some(Values::String(str)) = file_ext {
        write_ts_file(dir_out_str, out, str).expect("FAILED TO WRITE TO FILE");
        return;
    }
    write_ts_file(dir_out_str, out, "json").expect("FAILED TO WRITE TO FILE");
}
