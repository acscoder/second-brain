use oasysdb::prelude::*;
use std::collections::HashMap;

pub fn save_embeding(
    db_name: &str,
    collection_name: &str,
    doc_id: String,
    chunk_id: usize,
    v: Vec<f32>,
    content: String,
) {
    let mut db = docs_db(db_name);
    let mut collection = get_collection(&db, collection_name);
    let mut meta = HashMap::new();
    meta.insert("doc_id".to_owned(), Metadata::Text(doc_id));
    meta.insert("position".to_owned(), Metadata::Integer(chunk_id));
    meta.insert("content".to_owned(), Metadata::Text(content));

    let record = Record::new(&Vector(v), &Metadata::Object(meta));
    let _ = collection.insert(&record);
    db.save_collection(collection_name, &collection).unwrap();
}
pub fn docs_db(db_name: &str) -> Database {
    let db = Database::open(db_name).unwrap();
    db
}
pub fn get_collection(db: &Database, collection_name: &str) -> Collection {
    let res = db.get_collection(collection_name);
    match res {
        Ok(c) => c,
        _ => {
            let mut config = Config::default();
            config.distance = Distance::Cosine;
            let c = Collection::new(&config);
            c
        }
    }
}
