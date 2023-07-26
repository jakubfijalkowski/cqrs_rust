use convert_case::{Case, Casing};

pub struct FullName {
    pub namespaces: Vec<String>,
    pub name: String,
}

pub fn split_fullname(name: &str) -> FullName {
    let mut split: Vec<_> = name.split('.').map(to_namespace).collect();
    let name = split.pop().unwrap();
    FullName {
        namespaces: split,
        name: to_type(&name),
    }
}

pub fn to_internal_name(name: &str) -> String {
    let mut split: Vec<_> = name.split('.').map(to_namespace).collect();
    let last = split.last_mut().unwrap();
    *last = to_type(last);

    split.join("::")
}

pub fn get_type(name: &str) -> &str {
    name.split('.').rev().next().unwrap()
}

pub fn to_namespace(n: &str) -> String {
    n.to_case(Case::Snake)
}

pub fn to_type(n: &str) -> String {
    n.to_case(Case::Pascal)
}

pub fn to_const(n: &str) -> String {
    n.to_case(Case::UpperSnake)
}

pub fn to_field(n: &str) -> String {
    let name = n.to_case(Case::Snake);
    if name == "type" {
        "r#type".to_string()
    } else {
        name
    }
}
