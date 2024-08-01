use crate::rw_text::rw_txt::{read_string_from_txt, write_string_to_txt};
use crate::vector_db::{docs_db, get_collection, save_embeding};
use oasysdb::prelude::*;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

static DOCUMENTS_LIST: &str = "./db/document_list.json";
static DOCUMENTS_COLLECTION_NAME: &str = "documents";
static DATABASE_NAME: &str = "./db/vector_db";

#[derive(Serialize, Deserialize,Clone, Debug)]
pub struct Doc {
    pub id: String,
    pub file_path: String,
    pub title: String,
    pub description: String,
    pub categories: String,
}
impl Doc {
    pub fn new(id: String, file_path: String) {
        let s = Self {
            id,
            file_path,
            title: String::new(),
            description: String::new(),
            categories: String::new(),
        };
        let text = read_string_from_txt(DOCUMENTS_LIST);
        let deserialized: Result<Vec<Doc>, serde_json::Error> = serde_json::from_str(&text);
        if let Ok(mut deserialized) = deserialized {
            deserialized.push(s);
            let json_string = serde_json::to_string(&deserialized).unwrap();
            write_string_to_txt(DOCUMENTS_LIST, json_string);
        }else{
            let d = vec![s];
            let json_string = serde_json::to_string(&d).unwrap();
            write_string_to_txt(DOCUMENTS_LIST, json_string);
        }
    }
    pub fn update(doc_id: String, title: String, desc: String, cats: String) {
        let text = read_string_from_txt(DOCUMENTS_LIST);
        let deserialized: Result<Vec<Doc>, serde_json::Error> = serde_json::from_str(&text);
        if let Ok(mut deserialized) = deserialized {
            for doc in deserialized.iter_mut() {
                if doc.id == doc_id {
                    doc.title = title;
                    doc.description = desc;
                    doc.categories = cats.clone();
                    break;
                }
            }
            let json_string = serde_json::to_string(&deserialized).unwrap();
            write_string_to_txt(DOCUMENTS_LIST, json_string);

        }
    }
    pub fn get(doc_id: String) -> Option<Doc> {
        let text = read_string_from_txt(DOCUMENTS_LIST);
        let deserialized: Result<Vec<Doc>, serde_json::Error> = serde_json::from_str(&text);
        if let Ok(deserialized) = deserialized {
            for doc in deserialized {
                if doc.id == doc_id {
                    return Some(doc);
                }
            }
        }
        None
    }
    pub fn delete(doc_id: String) {
        let text = read_string_from_txt(DOCUMENTS_LIST);
        let deserialized: Result<Vec<Doc>, serde_json::Error> = serde_json::from_str(&text);
        if let Ok(deserialized) = deserialized {
            let s:Vec<Doc> = deserialized.into_iter().filter(|doc|doc.id!=doc_id).collect();
            let json_string = serde_json::to_string(&s).unwrap();
            write_string_to_txt(DOCUMENTS_LIST, json_string);
            DocChunk::delete_by_doc_id(doc_id);
        }
    }
    pub fn get_id(file_path: &str) -> String {
        let text = read_string_from_txt(DOCUMENTS_LIST);
        let deserialized: Result<Vec<Doc>, serde_json::Error> = serde_json::from_str(&text);
        if let Ok(deserialized) = deserialized {
            for doc in deserialized {
                if &doc.file_path == file_path {
                    return doc.id.clone();
                }
            }
        }
        "".to_owned()
    }
    pub fn getAll()->Vec<Doc>{
        let text = read_string_from_txt(DOCUMENTS_LIST);
        let deserialized: Result<Vec<Doc>, serde_json::Error> = serde_json::from_str(&text);
        if let Ok(deserialized) = deserialized {
            return deserialized;
        }
        Vec::new()
    }
    pub fn get_by_category(cat: &String) -> Vec<Doc> {
        let text = read_string_from_txt(DOCUMENTS_LIST);
        let deserialized: Result<Vec<Doc>, serde_json::Error> = serde_json::from_str(&text);
        if let Ok(deserialized) = deserialized {
            return deserialized.into_iter().filter(|doc| {
                doc.categories.to_lowercase().split(",").collect::<Vec<&str>>().contains(&cat.to_lowercase().as_str())
            }).collect();
        }
        Vec::new() 
    }
}
#[derive(Clone)]
pub struct DocChunk {
    pub doc_id: String,
    pub chunk_id: usize,
    pub content: String,
    embeding_vector:Vec<f32>
}
impl DocChunk {
    pub fn new(doc_id: String, chunk_id: usize, content: String,embeding_vector: Vec<f32>) -> Self {
        Self {
            doc_id,
            chunk_id,
            content,
            embeding_vector
        }
    }
    
    pub fn save_embeding(&self){
        save_embeding(
            DATABASE_NAME,
            DOCUMENTS_COLLECTION_NAME,
            self.doc_id.clone(),
            self.chunk_id,
            self.embeding_vector.clone(),
            self.content.clone(),
        );
        println!("Embeding saved for chunk_id: {},vector {}", self.chunk_id,self.embeding_vector.len());
    }
    pub fn get_by_doc_id(doc_id: String) -> HashMap<VectorID, Record> {
        let collection = get_doc_collection();
        let mut s = std::collections::HashMap::new();
        s.insert("doc_id".to_owned(), Metadata::Text(doc_id) );
        let m = Metadata::Object(s);
        let docs: HashMap<VectorID, Record> = collection.filter (&m).unwrap();
        docs
    }
    pub fn delete_by_doc_id(doc_id: String){
        let docs = Self::get_by_doc_id(doc_id);
        let mut db = docs_db(DATABASE_NAME);
        let mut collection = get_collection(&db, DOCUMENTS_COLLECTION_NAME);
        for d in docs.into_iter(){
            collection.delete(&d.0);
        };
        db.save_collection(DOCUMENTS_COLLECTION_NAME, &collection).unwrap();
    }
    
}

pub fn get_doc_collection() -> Collection {
    let db = docs_db(DATABASE_NAME);
    let collection = get_collection(&db, DOCUMENTS_COLLECTION_NAME);
    collection
}
 
pub fn get_categories(docs:&Vec<Doc>)->Vec<String>{
    let mut cate = get_categories_raw(docs);
    let hs: HashSet<String> = HashSet::from_iter(cate.iter().cloned());
    cate = hs.into_iter().map(|item|capitalize(&item)).collect();
    cate
}
pub fn get_categories_raw(docs:&Vec<Doc>)->Vec<String>{
    let cats:Vec<String> = docs.iter().map(|d|d.categories.clone().trim().to_lowercase().to_owned()).collect();
    let cats_vec = cats.into_iter().map(|item|item.split(",").map(|s|s.to_owned()).collect::<Vec<String>>()).collect::<Vec<Vec<String>>>();
    let cate = cats_vec.into_iter().flatten().collect::<Vec<String>>();
    cate
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}