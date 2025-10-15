use std::{ collections::{ HashMap, HashSet }, fmt::{ Debug, Display } };
#[derive(Debug)]
#[allow(dead_code)]
pub enum Values {
    String(String),
    Integer(i32),
}

pub struct ParsedArgs {
    tags: HashSet<String>,
    values: HashMap<String, Values>,
    pub unordered: Vec<Values>,
    pub build: String,
}

impl Display for ParsedArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n TAGS: {:?} \n VALUES: {:?} \n UNORDERED: {:?} \n BUILD: {:?}",
            self.tags,
            self.values.iter().collect::<Vec<(&String, &Values)>>(),
            self.unordered,
            self.build
        )
    }
}

fn try_parse(target_str: &String) -> Values {
    let int_parse = target_str.parse::<i32>();
    if let Ok(int) = int_parse {
        return Values::Integer(int);
    }
    Values::String(target_str.clone())
}

fn is_attribute(str: &String) -> bool {
    str.starts_with("-") && !str.starts_with("--")
}

impl ParsedArgs {
    pub fn has_tag(&self, tag_names: &'static [&'static str]) -> bool {
        let itr = tag_names.into_iter();
        for i in itr {
            if self.tags.contains(*i) {
                return true;
            }
        }
        false
    }
    pub fn get_value(&self, index_names: &'static [&'static str]) -> Option<&Values> {
        let itr = index_names.into_iter();
        for i in itr {
            let rslt = self.values.get(*i);
            if let Some(v) = rslt {
                return Some(v);
            }
        }
        None
    }
    pub fn parse(args: impl Iterator<Item = String>) -> Self {
        let arg_vec: Vec<String> = args.collect();
        let mut tags = HashSet::new();
        let mut values = HashMap::new();
        let mut unordered = Vec::new();
        let mut build = String::new();
        for (index, val) in arg_vec.iter().enumerate() {
            if index == 0 {
                build.push_str(val);
                continue;
            }
            if val.starts_with("-") {
                if !is_attribute(val) {
                    tags.insert(val.clone());
                    continue;
                }
            } else {
                if index == 0 {
                    unordered.push(try_parse(val));
                    continue;
                }
                let prev = index - 1;
                if prev > 0 && let Some(str) = arg_vec.get(prev) && is_attribute(str) {
                    values.insert(str.clone(), try_parse(val));
                } else {
                    unordered.push(try_parse(val));
                }
            }
        }
        Self {
            tags,
            values,
            unordered,
            build,
        }
    }
}
