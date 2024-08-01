use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use std::collections::HashMap;
use std::fs::{OpenOptions,File,read_to_string};
use std::io::{Read,Write};
use std::path::Path;
use dotext::*;

use pdf_extract::extract_text;
pub fn read_lines(filename: &str) -> Vec<String> {
    let ext = file_extension_extract(filename);
    let s: Vec<String> = match ext.as_str() {
        "pdf" => {
            let text = extract_text(filename).unwrap_or("".to_owned());
            string_to_lines(text)
        },
        "docx"|"doc" => read_lines_docx(filename),
        _ => read_lines_txt(filename),
    };
    s
}
fn read_lines_txt(filename: &str) -> Vec<String> {
    let filename = Path::new(filename);
    if filename.exists() {
        let v: Vec<String> = read_to_string(filename)
            .unwrap()
            .lines()
            .map(String::from)
            .collect();
        return v;
    }
    vec![]
}
fn read_lines_docx(filename: &str) -> Vec<String> {
    let st = read_string_from_docx(filename);
    string_to_lines(st)
}
fn string_to_lines(s: String) -> Vec<String> {
    let r = s.split("\n");
    let rv = r
                .into_iter()
                .filter(|x| x.trim() != "")
                .map(|x| x.to_string())
                .collect();
            rv
}

fn split_txt_var(input: String) -> Option<(String, String)> {
    let split = &input.split('=');
    let c = split.clone().count();
    if c == 2 {
        let x: Vec<&str> = split.clone().map(|r| r.trim()).collect();
        if x[0] != "" && x[1] != "" {
            return Some((x[0].to_string(), x[1].to_string()));
        }
        return None;
    }
    None
}
pub fn read_config(filename: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let lines = read_lines(filename);
    for line in lines {
        if let Some((key, value)) = split_txt_var(line) {
            map.insert(key, value);
        }
    }
    map
}

fn string_to_string_identity(s: String) -> String {
    s
}
pub fn string_to_crypt(s: String) -> String {
    let mc = new_magic_crypt!("iNY2ps3f6Zo", 256);
    mc.encrypt_str_to_base64(s)
}
fn crypt_to_string(s: String) -> String {
    let mc = new_magic_crypt!("iNY2ps3f6Zo", 256);
    match mc.decrypt_base64_to_string(&s) {
        Ok(r) => r,
        Err(_) => s,
    }
}
pub fn read_config_crypt(filename: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let lines = read_lines(filename);
    for l in lines {
        let line = crypt_to_string(l);
        if let Some((key, value)) = split_txt_var(line) {
            map.insert(key, value);
        }
    }
    map
}
pub fn read_config_crypt_key(filename: &str, key: &str) -> Option<String> {
    let setting_cgf = read_config_crypt(filename);
    setting_cgf.get(key).map(|x|x.to_owned())     
}
fn write_config_filter(
    filename: &str,
    hm: HashMap<String, String>,
    filter: Box<dyn Fn(String) -> String>,
) {
    let ct: Vec<String> = hm
        .into_iter()
        .map(|s| filter(format!("{}={}", s.0, s.1)))
        .collect();
    write_string_to_txt(filename, ct.join("\n"));
}
pub fn write_config(filename: &str, hm: HashMap<String, String>) {
    write_config_filter(filename, hm, Box::new(string_to_string_identity));
}
pub fn write_config_crypt(filename: &str, hm: HashMap<String, String>) {
    write_config_filter(filename, hm, Box::new(string_to_crypt));
}

pub fn add_config_crypt(filename: &str, key:String,val:String) {
    let mut hm = read_config_crypt(filename);
    hm.insert(key,val);
    write_config_crypt(filename, hm);
}

pub fn write_string_to_txt(filename: &str, content: String) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(filename)
        .unwrap();
    file.write_all(content.as_bytes()).unwrap();
}
pub fn read_string_from_txt(filename: &str) -> String {
    let lines = read_lines(filename);
    lines.join("\n")
}

/*
let cf = read_config_crypt("./Cargo.toml");
    for (key, value) in cf.into_iter() {
        let line = format!("{}={}", key, value);
        println!("read var: {}", line);
    }
    write_config("./a.txt",cf);
*/

pub fn read_string_from_file(filename: &str) -> String {
    let ext = file_extension_extract(filename);
    let s = match ext.as_str() {
        "pdf" => {
            let text = extract_text(filename).unwrap();
            text
        },
        "docx"|"doc" => read_string_from_docx(filename),
        _ => read_string_from_txt(filename),
    };
    s
}
fn read_string_from_docx(filename: &str) -> String{
    let mut file = Docx::open(filename).unwrap();
    let mut isi = String::new();
    let _ = file.read_to_string(&mut isi); 
    isi
}
pub fn check_legal_ext(filename: &str, legal: &Vec<&str>) -> bool {
    let ext = file_extension_extract(filename);
    legal.contains(&ext.as_str())
}
fn file_extension_extract(filename: &str) -> String {
    let ext = Path::new(filename).extension().unwrap().to_str().unwrap();
    ext.to_string()
}
