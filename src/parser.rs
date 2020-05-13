use yaml_rust::yaml::Yaml;
use linked_hash_map::LinkedHashMap;

/// The abstract syntax tree of a dialogue
pub enum Ast {
    Msg(String),
    Choice(Vec<(String, Vec<Ast>)>),
    Wait,
    Call(String),
}

/// The representation of a dialogue file
pub struct File {
    pub procs : LinkedHashMap<String, Vec<Ast>>,
}

// heart of the parser
fn parse_yaml_command(src : Yaml) -> Ast {
    match src {
        Yaml::String(x) if x.trim() == "wait" => Ast::Wait,
        Yaml::Hash(mut map) if map.len() == 1 => {
            match map.pop_back() {
                Some((Yaml::String(cmd), Yaml::String(msg))) if cmd.trim() == "print" => Ast::Msg(msg),
                Some((Yaml::String(cmd), Yaml::String(id))) if cmd.trim() == "call" => Ast::Call(id),
                Some((Yaml::String(cmd), Yaml::Array(options))) if cmd.trim() == "choose" => {
                    let branches =
                    options.into_iter()
                    .map(
                        |x| {
                            match x {
                                Yaml::Hash(mut map) if map.len() == 1 => {
                                    match map.pop_back() {
                                        Some((Yaml::String(name), Yaml::Array(code))) => (name, code.into_iter().map(|x| parse_yaml_command(x)).collect()),
                                        _ => panic!("In the choice map the key must be a string and the value must be an array"),
                                    }
                                },
                                _ => panic!("the choice branch must be a hash with one key-value pair"),
                            }
                        }
                    ).collect();
                    Ast::Choice(branches)
                },
                _ => panic!("The command doesn't satisfy an possible format"),
            }
        },
        _ => panic!("The command doesn't satisfy an possible format"),
    }
}

pub fn parse_yaml(yaml_ast : Yaml) -> File {
    if let Yaml::Hash(map) = yaml_ast {
        let procs =
        map.into_iter()
        .map(
            |(name, code)| {
                let name = {
                    if let Yaml::String(x) = name { x }
                    else { panic!("The dialogue file must be keyed with strings") }
                };
                let code = {
                    if let Yaml::Array(x) = code { x }
                    else { panic!("The code of {} is not an array", name) }
                };
                (name, code.into_iter().map(&parse_yaml_command).collect())
            }
        ).collect();
        File { procs }
    } else { panic!("The file's root must be a hashmap") }
}
