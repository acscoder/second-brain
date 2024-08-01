use probly_search::*;
use probly_search::score::bm25;
use std::borrow::Cow;
use rayon::prelude::*;
use std::collections::HashSet;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

pub fn word_tokenizer(s: &str) -> Vec<Cow<str>> {
    s.split(' ').map(|x|x.to_lowercase()).map(Cow::from).collect::<Vec<_>>()
}

#[macro_export]
macro_rules! documents_indexing{
    ($documents:ident,$func:ident,$word_tokenizer:ident) => {
        {
            let mut index = Index::<usize>::new(1);
            index = $documents.iter().fold(index,|mut index,doc|{
                index.add_document(
                    &[$func],
                    $word_tokenizer,
                    doc.id as usize,
                    &doc,
                );
                index
            });
            index
        }
    }
} 
 
#[macro_export]
macro_rules! bm25_multi_keyword{
    ($documents:ident,$func:ident,$word_tokenizer:ident) => {
        {
            let index = documents_indexing!($documents, $func, $word_tokenizer);
            bm25_query(index, $word_tokenizer)
        }
    } 
}
 
pub fn bm25_query(index:Index<usize>,word_tokenizer:fn(&str) -> Vec<Cow<str>> )->Box<dyn Fn(&str) -> Vec<QueryResult<usize>>>{
    let f = move|queries: &str|{
        let keywords_list = queries.split("+").collect::<Vec<_>>();
        let keywords_list_len = keywords_list.len();
        let mut final_results: Vec<QueryResult<usize>> = Vec::new();
        for query in keywords_list{
            let keywords = query.split(",").collect::<Vec<_>>();
            if keywords.len() > 1 {
                //Multi keywords search
                let results = keywords.into_par_iter().map(|query|{
                    let result: Vec<QueryResult<usize>> = index.query(
                        query,
                        &mut bm25::new(),
                        word_tokenizer,
                        &[1.],
                    );
                    result
                });
                //deduplication
                let ids = Arc::new(Mutex::new(HashSet::new()));
                let fr:Vec<QueryResult<usize>> = results.flatten().filter(|item|{
                    let vcl = ids.clone();
                    let mut vcl_guard = vcl.lock().unwrap();
                    if vcl_guard.contains(&item.key){
                        false
                    }else{
                        vcl_guard.insert(item.key);
                        true
                    }
                }).collect();
                final_results.extend(fr);

            }else{
                // single keyword search
                let result: Vec<QueryResult<usize>> = index.query(
                    query,
                    &mut bm25::new(),
                    word_tokenizer,
                    &[1.],
                );
                final_results.extend(result);
            }
        }
        if keywords_list_len > 1{
          
            //and group
            let ids: Arc<Mutex<HashMap<usize,usize>>> = Arc::new(Mutex::new(HashMap::new()));
            final_results.par_iter().for_each(|item|{
                let vcl = ids.clone();
                let mut vcl_guard = vcl.lock().unwrap();
                if let Some(v) = vcl_guard.get_mut(&item.key){
                    *v += 1;
                }else{
                    vcl_guard.insert(item.key, 1);
                }
            });

            let ids_hashset = Arc::new(Mutex::new(HashSet::new()));

            let fsx: Vec<QueryResult<usize>> = final_results.into_par_iter().filter(|item|{
                  let vcl = ids_hashset.clone();
                  let mut vcl_guard = vcl.lock().unwrap();
                  if vcl_guard.contains(&item.key){
                      false
                  }else{
                      vcl_guard.insert(item.key);
                      true
                  }
              }).filter_map(|mut item|{
                let vcl_guard = ids.lock().unwrap();
                if let Some(v) = vcl_guard.get(&item.key){
                    if v > &1{
                        let x = v.clone() as f64;
                        //item.score = item.score + x;
                        return Some(item);
                    }else{
                        return None;
                    }
                }
                Some(item)
            }).collect();
            
            fsx
        }else{
            final_results
        }
        
    };
    Box::new(f)
}

pub fn sort_by_score(results: &mut [QueryResult<usize>]) {
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
}
