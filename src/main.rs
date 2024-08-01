#![allow(non_snake_case)]

use std::{collections::HashMap, thread, collections::HashSet};
 

use dioxus::prelude::*;
use std::time::SystemTime; 
  
use std::pin::Pin;
use std::future::Future;
use std::{boxed::Box, rc::Rc};

mod vector_db;
use oasysdb::prelude::*;
use tokio::runtime::Runtime;

use tracing::Level;
use chrono::prelude::*;
mod llm;
mod keywords;
mod rw_text;
mod utils;
mod os_command; 
mod our_document;
use os_command::*;

use rw_text::rw_txt::*;
use keywords::keywords_search::*;
use keywords::keywords_extraction::get_keywords;
 
use our_document::{Doc,DocChunk,get_doc_collection};

use llm::prompt_template::*;
 
use std::{fs, vec};

use serde_json; 
use rayon::prelude::*;

use tiktoken_rs::p50k_base;
use llm::llm_api::*;

use probly_search::*;
 
use chat_templates::{apply_template,Message,ChatRole,ChatTemplate};

use std::path::{Path, PathBuf};
use  std::sync::{Mutex,Arc};

static LLM_API_SETTING: &str = "./db/api_setting.cgf";

static CHAT_HISTORY_DIR:&str = "./db/chat_history/";
static MAX_CHUNK_TOKEN:usize = 500;

static  DEFAULT_LLM_MODEL:&str = "Ollama::llama3";
static  DEFAULT_EMBEDDING_MODEL:&str = "Ollama::nomic-embed-text";



#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/setting")]
    Setting {},
    #[route("/documents")]
    YourDocuments {},

    //#[route("/blog/:id")]
    //Blog { id: i32 },
}
struct VecResultSearch{
    id:usize,
    content:String
}
 
fn main() {
    let _ = ollama_command(&["serve"]);
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");

    let cfg = dioxus::desktop::Config::new()
        .with_custom_head(r#"<script type="text/javascript" src="./public/js/script.js"></script><link rel="stylesheet" href="./public/css/tailwind.css"/><link rel="stylesheet" href="./public/css/main.css"/>"#.to_string());
    LaunchBuilder::desktop().with_cfg(cfg).launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
   
    let mut all_history:Vec<Message> = vec![];
    
    let get_history_current_filename = basename_to_history_path(&get_today_string()) ;

    let c1 = get_history(&get_history_current_filename);
    let mut chat_content = use_signal(move || c1);
    
    

    let s : Vec<Message> = get_default_history_result();
    let mut search_result = use_signal(move || s);

    let u : Vec<String> = vec![];
    let mut search_keyword_results = use_signal(move||u);

    let mut checkbox_select_doc_manually = use_signal(||false) ;

    
    let mut docs_hook: Signal<Vec<Doc>> = use_signal( ||vec![]);
    let mut selected_docs_hook: Signal<Vec<String>> = use_signal( ||vec![]);
    eval(
        r#"
        setTimeout(scrollToBottom_chatbox, 500); 
        "#,
    );
    rsx! {
                div{
                    class:"container mx-auto shadow-lg rounded-lg",
                  Header{}
                    div{class:"flex flex-row justify-between bg-white",
                      div{class:"flex flex-col w-2/5 border-r-2 overflow-y-auto ",
                        div{class:"border-b-2 py-4 px-2",
                          input{
                            r#type:"text",
                            placeholder:"search history",
                            name:"search_history",
                            class:"py-2 px-2 border-2 border-gray-200 rounded-xl w-full",
                            onchange:move|e|{
                                let query_data = e.data.value().to_string() ;
                                let query_lowercase = &query_data.to_lowercase();
                                if query_lowercase.as_str() == ""{
                                    *search_result.write() = get_default_history_result();
                                    return;
                                }
                                if all_history.len() == 0 {
                                    all_history = get_all_history();
                                }
                                let bm25_query = bm25_multi_keyword!(all_history,get_content_chat_item,word_tokenizer);
                                
                                let result = bm25_query(query_lowercase);

                                search_result.clear();
                                for r in result{
                                    for item in all_history.iter(){
                                        if item.id as usize == r.key{
                                            search_result.push(item.clone());
                                            break;
                                        }
                                    }  
                                }  
                            }
                          }
                        }
                        div{class:"px-2 mt-4 text-md font-semibold","Chat History"}
                        for item in search_result.iter(){
                            ChatItemSearch{ 
                                id:item.id,
                                role:item.role.clone(),
                                content:item.content.clone()
                            }
                        }   
                      }

                      div{class:"w-full px-5 flex flex-col justify-between",
                        div{
                        class:"flex flex-col mt-5 min-h-[700px] max-h-[900px] overflow-y-auto",
                        id:"chatbox",
                             for item in chat_content.iter(){
                                ChatItem{ 
                                    id:item.id,
                                    role:item.role.clone(),
                                    content:item.content.clone()
                                }
                            }
                        }
                        form{
                        class:"py-5",
                        onsubmit: move|event|{ let event_data = event.data.values();
                        let user_query = event_data.get("user_query").unwrap().as_value(); 
                        let user_query = user_query.trim().to_string();
                        let user_query_1 = user_query.clone();
                        
                        if user_query != "".to_owned(){
                            chat_content.push(Message::new(ChatRole::User, user_query.clone()));
                            chat_content.push(Message::new(ChatRole::System,  "Thinking ...".to_owned()+"<span class='dot-fire'></span>" ));
                            eval(
                                r#"
                                setTimeout(scrollToBottom_chatbox, 100);
                                clear_input("user_query");
                                "#,
                            );
                            let selected_docs:Vec<String> = (*selected_docs_hook()).to_vec();

                             spawn(async move {
                                println!("selected_docs {:?}",selected_docs);
                                //let keywords = get_keywords(is_eng_str(is_en),&user_query_1);
                                //let sw_string = keywords.into_iter().map(|s| s.keyword).collect::<Vec<String>>().join(",").to_lowercase();
                               
                                let collection = get_doc_collection();
                                let embeding = get_embedding();
                                let v = embeding(user_query_1).await;
                                let query = Vector::from(v);
                                let result = collection.search(&query, 30).unwrap();
                                let lim = 5 ;
                               /*  let data_doc:Vec<VecResultSearch> = result.iter().map(|item|{VecResultSearch{id:item.id as usize,content:get_content_vector_search(item).to_string()}}).collect();
                                let bm25_query = bm25_multi_keyword!( data_doc,get_data_doc_content,word_tokenizer);
                                let mut bm25_result = bm25_query(&sw_string);
                                sort_by_score(&mut bm25_result);
                                
                                let res:Vec<_> = bm25_result.into_iter().filter_map(|item|result.get(item.key)).collect();
                            */
                           
                                let dtx = result.iter().take(lim).map(|item| {get_content_vector_search(item)}).collect::<Vec<&str>>().join("\n ------------------------- \n");
                                       println!("dtx {}", dtx);

                                let system_promt = Message::new(
                                     ChatRole::System,
                                     system_prompt_template("chatting",vec![dtx])
                                );
                                let mut chat_content_vec = vec![system_promt];

                                let chat_content_limited = content_token_limit((*chat_content()).to_vec(),512);
                                
                                chat_content_vec.extend(chat_content_limited);

                                let user_query_cp = apply_template(ChatTemplate::ChatML,&chat_content_vec,true).unwrap();
                                
                                let llm = get_llm();
                                let res = llm(user_query_cp);
                                let result = res.await;
                                 
                                chat_content.pop(); 
                                chat_content.push(Message::new(ChatRole::Assistant, result));
                              
                                save_chat_history(&(*chat_content()).to_vec());  
                                
                                eval(
                                    r#"
                                    setTimeout(scrollToBottom_chatbox, 500); 
                                    "#,
                                );
                                 
                            });
                              
                         } 
                        } ,
                          textarea{
                            name:"user_query",
                            class:"w-full bg-gray-300 py-5 px-3 rounded-xl",
                            placeholder:"type your message here..."
                          }
                          button{
                            class:"text-white float-right bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800",
                            r#type: "submit",
                            "Send"
                          }
                        }
                      }

                      div{class:"w-2/5 border-l-2 px-5",
                    DocumentAdding{}

                          div{class:"flex flex-col",
                          div{class:"font-semibold text-xl py-4 border-t-2 mt-5","Related Documents"}
                          div {class:"flex items-start",
              input{
                r#type:"checkbox",
                onchange:move|e|{
                    let query_data = e.data.value();
                     if query_data == "true"{
                        checkbox_select_doc_manually.set(true);
                     }else {
                        checkbox_select_doc_manually.set(false);
                     }
                },
              class:"w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"}
              label{ class:"ms-2 text-xs font-medium text-gray-900 dark:text-gray-300",
              "Choose relative documents manually, uncheck to find its automatically base on your question."
                }
                    
                        }
                        if checkbox_select_doc_manually() {
                            div{
                                class:"py-4 px-2",
                                input{
                                    r#type:"text",
                                    placeholder:"Search documents",
                                    name:"search_documents",
                                    class:"py-2 px-2 border-2 border-gray-200 rounded-xl w-full",
                                    onchange:move|e|{
                                        let query_data = e.data.value().to_string() ;
                                        let query_lowercase = &query_data.trim().to_lowercase();
                                        let docs = Doc::getAll(); 
                                        
                                        if query_lowercase != ""{ 
                                            let docs_search = to_doc_search(&docs);
                                            let bm25_query = bm25_multi_keyword!(docs_search,get_content_docs_search,word_tokenizer);     
                                            let result = bm25_query(query_lowercase);
                                            
                                            let docs_result:Vec<Doc> = result.into_iter().map(|item|docs.get(item.key)).filter_map(|x|x).map(|doc|doc.clone()).collect();
                                            docs_hook.set(docs_result);
                                        }else{
                                            docs_hook.set(vec![]);
                                            }
                                        }
                                    }
                                }
                                for doc in docs_hook.iter(){
                                    {
                                    let id = doc.id.clone();
                                    rsx!{div{
                                        class:"flex flex-row",
                                        input{
                                            class:"mr-2 mt-1",
                                            r#type:"checkbox",
                                            
                                            onchange:move|e|{
                                                let query_data = e.data.value();
                                                let idc = id.clone();
                                                 if query_data == "true"{
                                                    selected_docs_hook.push(idc);
                                                 }else {
                                                    selected_docs_hook.retain(|x| *x != idc); 
                                                 }
                                                  
                                                }
                                            },
                                            label{class:"text-sm font-semibold", "{doc.title}"}
                                            }

                                        }
                                    }

                                }
                              }
                          }
                      }
                    }
                    }
            }
}
#[derive(PartialEq, Props, Clone)] 
struct MessageShow{
    id:i64,
    role:ChatRole,
    content:String
} 
#[component]
fn ChatItem(props: MessageShow) -> Element {
    let md_content = markdown::to_html(&props.content);
    return match props.role {
        ChatRole::Assistant => rsx! {
        div{class:"flex justify-start mb-4 chat_item",
                img{
                  src:"./public/images/ai-icon.png",
                  class:"object-cover h-8 w-8 ",
                  alt:""
                }
                div{
                  class:"ml-2 py-3 px-4 bg-gray-600 rounded-br-3xl rounded-tr-3xl rounded-tl-xl text-white", 
                  dangerous_inner_html:"{md_content}"
                }
              }
        },
        ChatRole::User => rsx! {
        div{
            class:"flex justify-end mb-4 chat_item",
                div{
                class:"mr-2 py-3 px-4 bg-gray-300 rounded-bl-3xl rounded-tl-3xl rounded-tr-xl text-gray-800",
                dangerous_inner_html:"{md_content}"
                    }
                img{
                    src:"./public/images/human-icon.png",
                    class:"object-cover h-8 w-8 ",
                    alt:""
                }
            }
        },
        ChatRole::System => rsx! {
            div{class:"flex justify-start mb-4 chat_item",
            img{
              src:"./public/images/ai-icon.png",
              class:"object-cover h-8 w-8 ",
              alt:""
            }
            div{
              class:"ml-2 py-3 px-4 bg-gray-600 rounded-br-3xl rounded-tr-3xl rounded-tl-xl text-white", 
              dangerous_inner_html:"{props.content}"
            }
          }
        },
        _ => rsx! {

        } 
    };
}

 
#[component]
fn ChatItemSearch(props: MessageShow) -> Element {
    let ct = truncate_text(props.content,30);
    let dtime = get_date_from_timestamp(props.id);
    let loadhistory_file =  basename_to_history_path(&dtime);
    rsx! {
        div{
            onclick:move|_|{
                println!("{}",loadhistory_file);
            },
            class:"w-full text-xs p-2 border-dotted border-b border-slate-200 hover:bg-slate-200 cursor-pointer",
            span{ class:"text-gray-400 font-bold", "{dtime}" }
            p{ class:"text-gray-500 ",{ct}}
        }
    }
}
 
#[component]
fn Header() -> Element {
    rsx!(
        div{class:"px-5 py-5 flex justify-between items-center bg-white border-b-2",
         
          Link { to: Route::Home {},
          class:"inline-flex items-center gap-3",
           img{
            src:"./public/images/logo.png",
            class:"",
            width:"50"},
            alt:"logo",
            span{
            class:"font-bold text-md",
            "Second Brain"}
        }
     

      div{ class:"",
      ul{ class:"py-2 text-sm text-gray-700 dark:text-gray-200 inline-flex gap-5",
        li{
          Link { to: Route::YourDocuments {}, "Documents" }
        }
        li{
            Link { to: Route::Setting {}, "Setting" }
          }
      }
    }

        }
    )
}
#[component]
fn DocumentAdding() -> Element {
    let u : Vec<String> = vec![];
    let mut uploading_file = use_signal(move || u);
    let mut is_loading_document = use_signal(||"".to_owned());
 
    rsx! {
        div{class:"font-semibold text-xl py-4","Add your Documents"}
        div{class:"",
          ul{class:"inline-flex gap-3 mb-3 simple_tabs documents_adding_tabs",
              li{class:"py-2 simple_tab active",id:"uploading", "Upload"}
              li{class:"py-2 simple_tab", id:"scraping","Scraping"}
              li{class:"py-2 simple_tab", id:"take-note","Take Note"} 
          }
        }
          form{
              id:"uploading-content",
              class:"flex flex-col mb-5 simple_tab_content active",
          onsubmit: move|_evt|{
            
                  for f in uploading_file.read().iter(){
                      let file_path = f.clone();
                      if Doc::get_id(&file_path) != "" {
                        continue;
                      }
                      *is_loading_document.write() = file_path.clone(); 
                       
                      thread::spawn(move|| {
                        
                          let rv = read_lines(&file_path);
                          
                          let frv:Vec<(usize,String)> = rv.par_iter().map(|item|(token_count(&item),item.to_owned())).collect();
                         
                          let fold_init:(String,usize,Vec<String>) = ("".to_owned(),0,vec![]);
                        
                          let mut doc_para = frv.into_iter().fold(fold_init,|fold_value,item|{
                              let mut fold_value = fold_value;
                             
                              if fold_value.1 + item.0 > MAX_CHUNK_TOKEN{
                                  fold_value.2.push(fold_value.0);
                                  fold_value.0 = item.1;
                                  fold_value.1 = item.0;
                              }else{
                                  fold_value.0 = fold_value.0 + "\n" +&item.1;
                                  fold_value.1 = fold_value.1 + item.0;
                              }
                              fold_value
                          });
                         
                            if &doc_para.0 != ""{
                              doc_para.2.push(doc_para.0);
                              doc_para.0 = "".to_owned();
                          }
                          
                          
                          let sample:String = doc_para.2.iter().enumerate().take_while(|x|x.0 < 3).fold("".to_owned(),|s,item|{let r=s +&item.1;r});
                         
                          
                          let doc_id = get_basename(&file_path);
                          
                          Doc::new(doc_id.clone(),file_path);
                          let vx :Vec<(usize,String)> =  doc_para.2.into_iter().enumerate().collect();
                          
                          let rt  = Runtime::new().unwrap();
                          let docChunks:Vec<DocChunk> = vx.into_par_iter().map(|chunk| {
                              let c_tx = chunk.1.clone();
                              let embeding_vector = rt.block_on(async move{
                                  let embeding = get_embedding();
                                  embeding(c_tx).await
                              });

                              DocChunk::new(doc_id.clone(), chunk.0,chunk.1, embeding_vector)

                          }).collect();

                          thread::spawn(move||{
                              docChunks.into_iter().for_each(|item|item.save_embeding());
                          });

                          let llm = get_llm();

                          let system_promt = Message::new(
                               ChatRole::System,
                              system_prompt_template("get_title_description",vec![])
                          );
                          let user_promt = Message::new(ChatRole::User,sample);
                          let prmt = apply_template(ChatTemplate::ChatML,&vec![system_promt,user_promt],true).unwrap();
                           
                          let res = llm(prmt);
                          let c = rt.block_on(async move{
                              res.await
                          });
                          let (title,desc,cats)  = extract_title_description_category(c);
                          println!("title {title},desc {desc},cats {cats}");
                          if &title != ""{
                              Doc::update(doc_id, title, desc, cats);
                          }
                          
                         
                         
                      });
                      
                  }
                  
                  uploading_file.write().clear(); 
                                          
          },
          

input{
r#type:"file",
accept: ".txt,.pdf,.csv,.xlsx,.docx,.odt,.html",
class:"block w-full text-sm text-gray-900 border border-gray-300 rounded-lg cursor-pointer bg-gray-50 dark:text-gray-400 focus:outline-none dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400",
onchange: move |evt| {
async move {
  if let Some(file_engine) = &evt.files() {
      let files = file_engine.files();
      for file_name in files {
          uploading_file.write().push(file_name); 
      }
  }
}
}
}
p {class:"mt-1 text-sm text-gray-500 dark:text-gray-300" ,
"Document files .docx, .pdf, .txt, .csv"}

button{
class:"text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 mt-4 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800",
r#type: "submit",
"Process"
}
            }

            form{
              id:"scraping-content",
              class:"flex flex-col simple_tab_content",
              input{
                  class:"py-2 px-2 border-2 border-gray-200 rounded-xl w-full",
                  r#type:"text",
                  placeholder:"Website URL",
                  value:""
              } 
              p {class:"mt-1 text-sm text-gray-500 dark:text-gray-300" ,
                  "Website or youtube video url, ex:https://en.wikipedia.org/wiki/Web_scraping"}
                  button{
                      class:"text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 mt-4 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800",
                      r#type: "submit",
                      "Process"
                  }
           } 
           form{
            onsubmit:move|evt|{
                let event_data = evt.data.values();
                let note_content = event_data["note_content"].join("\n");
                let description = event_data["note_description"].join("\n");
                
                thread::spawn(move|| {
                    
                    let doc_id = get_now_timestamp().to_string();
                    let embeding = get_embedding();
                    let rt  = Runtime::new().unwrap();
                    Doc::new(doc_id.clone(), "".to_owned());
                    Doc::update(doc_id.clone(),note_content.clone(), description.clone(), "Note".to_owned());
                    let c_tx = note_content.clone()+"\n"+&description;
                    let embeding_vector = rt.block_on(async move{
                        embeding(c_tx).await
                    });

                    let dc = DocChunk::new(doc_id.clone(),0,note_content, embeding_vector);
                    dc.save_embeding();

                });

                eval(
                    r#"
                    clear_input("note_content");
                    clear_input("note_description");
                    "#,
                );

            },
              id:"take-note-content",
              class:"flex flex-col simple_tab_content",
              textarea{
                class:"py-2 px-2 border-2 border-gray-200 rounded-xl w-full",
                name:"note_content",
                placeholder:"Note",
                value:""
            } ,
              textarea{
                  class:"py-2 px-2 border-2 border-gray-200 rounded-xl w-full",
                  name:"note_description",
                  placeholder:"Short Description",
                  value:""
              } 
              button{
                  class:"text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 mt-4 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800",
                  r#type: "submit",
                  "Submit"
              }
           }
           if &*is_loading_document.read()!="" {
            div{
                class:"modal-box rounded",
                id:"document-loading-progress",
                form{
                    onsubmit:move|evt|{
                        *is_loading_document.write() = "".to_owned();
                    },
                    class:"self-center bg-white p-5",
                    h2{
                        "Your document is loading", 
                    },
                    img{
                        class:"mx-auto mb-4",
                        src:"./public/images/progress.gif",
                        width:240
                    },
                    p{
                        class:"text-left text-xs",
                        "Your document is loading, the form below is optional, you can leave it empty"
                    }
                    input{
                        r#type:"text",
                        class:"py-2 px-2 my-2 border-2 border-gray-300 bg-gray-300 rounded-xl w-full",
                        placeholder:"Document title",
                        value:""
                    },
                    textarea{
                        name:"user_query",
                        class:"w-full bg-gray-300 py-5 px-3 rounded-xl",
                        placeholder:"Document Description"
                      }
                      button{
                        class:"text-white float-right bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800",
                        r#type: "submit",
                        "Close"
                      }
                }
            }
        }  
    }
}
#[derive(Debug,PartialEq, Props,Clone)]
pub struct DocShow {
     id: String,
     file_path: String,
     title: String,
     description: String,
     categories: String,
}
#[component]
fn DocumentItem(props:DocShow)-> Element{
    rsx!(
        div{class:"flex flex-col mb-4",  id:"item-{&props.id}",
        div{class:"flex flex-row justify-between",
        h3{class:"text-md font-semibold", "{&props.title}"}
        }
       
        p{class:"text-sm text-gray-500",dangerous_inner_html:"{&props.description}"}
            div{
              
                class:"mt-3 inline-flex justify-between w-full",
                a{
                    class:"text-blue-300 hover:underline italic",
                    href:"{&props.file_path}",
                    "View orginal file >>",
                }
                button{
                    onclick:move|_|{
                        Doc::delete(props.id.clone());
                        let ev = format!(r#"document.getElementById("item-{}").remove();"#,&props.id);
                        eval(
                            &ev
                        );
                    },
                    class:"bg-white hover:bg-red-300 hover:text-white hover:border-red-300 text-gray-800 font-semibold py-2 px-4 border border-gray-400 rounded",
                    r#type: "button",
                    img{
                        src:"./public/images/trash.png",
                        class:"",
                        width:"20"
                    }
                }
            }
        }
    )
}

#[derive(Debug)]
struct DocSearch{
    id:usize,
    doc_id:String,
    content:String
}
fn to_doc_search(docs:&Vec<Doc>)->Vec<DocSearch>{
    docs.iter().enumerate().map(|item|DocSearch{id:item.0,doc_id:item.1.id.clone(),content:format!("{} \n {} \n {}",item.1.title.to_lowercase(),item.1.description.to_lowercase(),item.1.categories.to_lowercase()) }).collect()
}
#[component]
fn YourDocuments() -> Element {
    let docs = Doc::getAll();
    let cats = our_document::get_categories(&docs);
     
    let mut docs_hook = use_signal(move||docs);    
    let cats_hook = use_signal(move||cats);
    rsx!(
        div{
            class:"container mx-auto mb-6",
          Header{}
            div{class:"flex flex-row justify-between bg-white",
            div{class:"flex flex-col w-2/5 border-r-2 overflow-y-auto ",
            
            div{class:"border-b-2 py-4 px-2",
              input{
                r#type:"text",
                placeholder:"search documents",
                name:"search_documents",
                class:"py-2 px-2 border-2 border-gray-200 rounded-xl w-full",
                onchange:move|e|{
                    let query_data = e.data.value().to_string() ;
                    let query_lowercase = &query_data.trim().to_lowercase();
                    let docs = Doc::getAll();

                    if query_lowercase != ""{ 
                        let docs_search = to_doc_search(&docs);
                        let bm25_query = bm25_multi_keyword!(docs_search,get_content_docs_search,word_tokenizer);     
                        let result = bm25_query(query_lowercase);
                        
                        let docs_result:Vec<Doc> = result.into_iter().map(|item|docs.get(item.key)).filter_map(|x|x).map(|doc|doc.clone()).collect();
                        docs_hook.set(docs_result);
                    }else{
                        docs_hook.set(docs);
                    }
                }
              }
              p {class:"mt-1 text-sm text-gray-500 dark:text-gray-300" ,
"you can add multi keywords by comma ex : Key1,Key2,Key3 \n or use plus + mean and ex:Key1+Key2,Key3 mean (Key1 and Key2) or Key3"}
            }
            div{class:"px-2 mt-4 ",
            h3{class:"text-md font-semibold","Document Categories"},
                ul{
                for idx in 0..cats_hook.len()  {
                    li{
                        class:"cursor-pointer",
                        onclick:move|_|{
                            let cat = cats_hook.index(idx);
                            let cat = cat.to_string();
                            let docs = Doc::get_by_category(&cat);
                            docs_hook.set(docs);
                        },
                        "{cats_hook.index(idx)}"
                    }
                }
                }
            }
        }
            div{class:"w-full px-5 py-4 flex flex-col justify-between",
            for doc in docs_hook.iter(){
                DocumentItem{
                    id:doc.id.clone(),
                    file_path:doc.file_path.clone(),
                    title:doc.title.clone(),
                    description:doc.description.clone(),
                    categories:doc.categories.clone(),
                }
            }
                }
                div{class:"w-2/5 border-l-2 px-5",
                DocumentAdding{}
                }
            }
        
        }
    
    )
}

#[component]
fn Setting() -> Element {
    let setting_cgf = read_config_crypt(LLM_API_SETTING);
    let OpenAI_key = setting_cgf
        .get("OpenAI_api_key")
        .unwrap_or(&"".to_owned())
        .to_owned();
    let hf_key = setting_cgf
        .get("Ollama_api_key")
        .unwrap_or(&"".to_owned())
        .to_owned();
    let gemini_key = setting_cgf
        .get("Gemini_api_key")
        .unwrap_or(&"".to_owned())
        .to_owned();
    let claude_key = setting_cgf
        .get("Claude_api_key")
        .unwrap_or(&"".to_owned())
        .to_owned();
    
   
    let llm_model = setting_cgf
        .get("llm_model")
        .unwrap_or(&DEFAULT_LLM_MODEL.to_owned())
        .to_owned();
    let mut llm_model_iter = llm_model.split("::");
    let llm_model_type_str = llm_model_iter.next().unwrap_or("Ollama").to_owned();
    let llm_model_name_str = llm_model_iter.next().unwrap_or("llama3").to_owned();
    let llm_list_vec = get_models_llm_list(&llm_model_type_str);
    let mut llm_list = use_signal(move||llm_list_vec);
    let mut llm_model_type = use_signal(move||llm_model_type_str);
    let mut llm_model_name = use_signal(move||llm_model_name_str);

    let embedding_model = setting_cgf
        .get("embedding_model")
        .unwrap_or(&DEFAULT_EMBEDDING_MODEL.to_owned())
        .to_owned();
    let mut embedding_model_iter = embedding_model.split("::");
    let embedding_model_type_str = embedding_model_iter.next().unwrap_or("Ollama").to_owned();
    let embedding_model_name_str = embedding_model_iter.next().unwrap_or("nomic-embed-text").to_owned();
    let embedding_list_vec = get_models_embedding_list(embedding_model_type_str.as_str());

    let mut embedding_list = use_signal(move||embedding_list_vec);
    let embedding_model_type = use_signal(move||embedding_model_type_str);
    let embedding_model_name = use_signal(move||embedding_model_name_str);

    
    
    let mut OpenAI_api_key = use_signal(move || OpenAI_key);
    let mut Huggingface_api_key = use_signal(move || hf_key);
    let mut Gemini_api_key = use_signal(move || gemini_key);
    let mut Claude_api_key = use_signal(move || claude_key);

    let aki_keys_handle_submit = move |_| {
        let mut hm = read_config_crypt(LLM_API_SETTING);
         hm.insert( "OpenAI_api_key".to_owned(),OpenAI_api_key.read().to_string()); 
        hm.insert("Huggingface_api_key".to_owned(),Huggingface_api_key.read().to_string() ); 
        hm.insert(  "Gemini_api_key".to_owned(),Gemini_api_key.read().to_string() ); 
        hm.insert("Claude_api_key".to_owned(),Claude_api_key.read().to_string()); 
            
        write_config_crypt(LLM_API_SETTING, hm);
    };
 
    let mut tab_status = use_signal(move || "select_llm");

    let tab_item_class ="inline-block cursor-pointer p-4 rounded-t-lg hover:text-gray-600 hover:bg-gray-50 dark:hover:bg-gray-800 dark:hover:text-gray-300";
    let tab_item_class_active = "active inline-block cursor-pointer p-4 text-blue-600 bg-gray-100 rounded-t-lg dark:bg-gray-800 dark:text-blue-500 font-semibold";
    rsx! {
        div{
            class:"container mx-auto mb-6",
          Header{}
        }
        div{
            class:"max-w-lg mx-auto pt-6",
            ul{
                class:"mt-5 flex flex-wrap text-sm font-medium text-center text-gray-500 border-b border-gray-200 dark:border-gray-700 dark:text-gray-400 mb-5",
                
                li{
                    class:"me-2",
                    onclick: move|_|{*tab_status.write() = "select_llm";},
                    if tab_status.read().to_string().as_str() == "select_llm"{
                        a{ class:"{tab_item_class_active}", "LLM Model"}
                    }else{
                        a{ class:"{tab_item_class}","LLM Model" }
                    }
                },

                li{
                    class:"me-2",
                    onclick: move|_|{*tab_status.write() = "select_embedding_model";},
                    if tab_status.read().to_string().as_str() == "select_embedding_model"{
                        a{ class:"{tab_item_class_active}", "Embedding Model"}
                    }else{
                        a{ class:"{tab_item_class}","Embedding Model" }
                    }
                },
                li{
                    class:"me-2",
                    onclick: move|_|{*tab_status.write() = "api_keys";},
                    if tab_status.read().to_string().as_str() == "api_keys"{
                        a{ class:"{tab_item_class_active}", "API Keys"}
                    }else{
                        a{ class:"{tab_item_class}", "API Keys" }
                    }
                },
            }
        }
        if tab_status.read().to_string().as_str() == "api_keys" {
            form {
            class:"max-w-sm mx-auto ",
            onsubmit: aki_keys_handle_submit,
                div {
                    class:"mb-5",
                    label{
                        class:"mb-1",
                        class:"block mb-2 font-medium text-gray-900 dark:text-white",
                        "Huggingface API Key"
                    }
                    input{
                        class:"bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                        r#type: "text",
                        placeholder: "Optional",
                        value: "{Huggingface_api_key}",
                        oninput: move |event| Huggingface_api_key.set(event.value())
                    }
                }

                div {
                    class:"mb-5",
                    label{
                        class:"mb-1",
                        class:"block mb-2 font-medium text-gray-900 dark:text-white",
                        "OpenAI API Key"
                    }
                    input{
                        class:"bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                        r#type: "text",
                        placeholder: "Paste Your API Key Here..",
                        value: "{OpenAI_api_key}",
                        oninput: move |event| OpenAI_api_key.set(event.value())
                    }

                }
                div {
                    class:"mb-5",
                    label{
                        class:"mb-1",
                        class:"block mb-2 font-medium text-gray-900 dark:text-white",
                        "Gemini API Key"
                    }
                    input{
                        class:"bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                        r#type: "text",
                        placeholder: "Paste Your API Key Here..",
                        value: "{Gemini_api_key}",
                        oninput: move |event| Gemini_api_key.set(event.value())
                    }

                }
                div {
                    class:"mb-5",
                    label{
                        class:"mb-1",
                        class:"block mb-2 font-medium text-gray-900 dark:text-white",
                        "Claude AI API Key"
                    }
                    input{
                        class:"bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                        r#type: "text",
                        placeholder: "Paste Your API Key Here..",
                        value: "{Claude_api_key}",
                        oninput: move |event| Claude_api_key.set(event.value())
                    }

                }
                button{
                    class:"text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm w-full sm:w-auto px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800",
                    r#type: "submit",
                    "Save Changed"
                }
            }
        }else if tab_status.read().to_string().as_str() == "select_llm"{
            form{
                onsubmit:move|_|{
                    add_config_crypt(LLM_API_SETTING,"llm_model".to_owned(), format!("{}::{}",llm_model_type.read(),llm_model_name.read()));
                },
                class:"max-w-sm mx-auto ",
                    div {
                        class:"mb-5",
                        h3{
                            class:"mb-1",
                            "Select LLM Services"
                        }
                        select{
                            onchange:move|e|{
                                let selected_data = e.data.value().to_string() ;
                                let llm_list_vec = get_models_llm_list(selected_data.as_str());
                                llm_model_type.set(selected_data);
                                llm_list.set(llm_list_vec);
                            },
                            class:"bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                            if *llm_model_type.read() == "Ollama"{
                                option{ selected:true,"Ollama" }
                            }else{
                                option{"Ollama"}
                            } 
                            if *OpenAI_api_key.read() != ""{
                                if *llm_model_type.read() == "OpenAI"{
                                    option{ selected:true,"OpenAI" }
                                }else{
                                    option{"OpenAI"}
                                }
                            }
                            if *Claude_api_key.read() != ""{
                                if *llm_model_type.read() == "Claude"{
                                    option{ selected:true,"Claude" }
                                }else{
                                    option{"Claude"}
                                }
                            }
                            if *Gemini_api_key.read() != ""{
                                if *llm_model_type.read() == "Gemini"{
                                    option{ selected:true,"Gemini" }
                                }else{
                                    option{"Gemini"}
                                }
                            }
                             
                        }
                            
                    },
                 
                    div {
                        class:"",
                        label{
                            class:"mb-1",
                            "LLM model"
                        }
                        select{
                            class:"bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                            onchange:move|e|{
                                let selected_data = e.data.value().to_string() ;
                                llm_model_name.set(selected_data);
                            },
                            for model in llm_list.iter(){
                                if *llm_model_name.read() == *model{
                                    option{
                                        selected:true,
                                        "{model}"
                                    }
                                }else{
                                    option{
                                        "{model}"
                                    }
                                }
                                
                            }
                        },
                    }
                button{
                    class:"mt-5 text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm w-full sm:w-auto px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800",
                    r#type: "submit",
                    "Save Changed"
                }
            }
        
        }else if tab_status.read().to_string().as_str() == "select_embedding_model"{
            form{
                onsubmit:move|_|{
                    add_config_crypt(LLM_API_SETTING,"embedding_model".to_owned(), format!("{}::{}",embedding_model_type.read(),embedding_model_name.read()));
                },
                class:"max-w-sm mx-auto ",
                div {
                    class:"mb-5",
                    h3{
                        class:"mb-1",
                        "Select Embedding Services"
                    }
                    select{
                        onchange:move|e|{
                            let selected_data = e.data.value().to_string() ;
                            let embedding_list_vec = get_models_embedding_list(selected_data.as_str()); 
                            embedding_list.set(embedding_list_vec);
                        },
                        class:"bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                        if *embedding_model_type.read() == "Ollama"{
                            option{ selected:true,"Ollama" }
                        }else{
                            option{"Ollama"}
                        }
                        if *OpenAI_api_key.read() != ""{
                            if *llm_model_type.read() == "OpenAI"{
                                option{ selected:true,"OpenAI" }
                            }else{
                                option{"OpenAI"}
                            }
                        }
                        if *Claude_api_key.read() != ""{
                            if *llm_model_type.read() == "Claude"{
                                option{ selected:true,"Claude" }
                            }else{
                                option{"Claude"}
                            }
                        }
                        if *Gemini_api_key.read() != ""{
                            if *llm_model_type.read() == "Gemini"{
                                option{ selected:true,"Gemini" }
                            }else{
                                option{"Gemini"}
                            }
                        }
                        
                    }
                        
                },
                div {    
                    label{
                    class:"mb-1",
                    "Embedding model"
                    },
                    select{
                        class:"bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                        value: "{embedding_model_name}",
                       for model in embedding_list.iter(){
                                if *embedding_model_name.read() == *model{
                                    option{
                                        selected:true,
                                        "{model}"
                                    }
                                }else{
                                    option{
                                        "{model}"
                                    }
                                }
                                
                            
                        }
                    }
                },
                button{
                    class:"mt-5 text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm w-full sm:w-auto px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800",
                    r#type: "submit",
                    "Save Changed"
                }    
                
            }
     
        }else{
            div{
                "LLM"
            }
        }
    }
}

fn get_llm()->impl Fn(String)->Pin<Box<dyn Future<Output=String>>>{
    let llm_type = get_llm_type("llm_model",DEFAULT_LLM_MODEL);
    let llm = LLM_chat_config(llm_type);
    llm
}
fn get_embedding()->impl Fn(String) -> Pin<Box<dyn Future<Output=Vec<f32>>>>{
    let llm_type: LLMType = get_llm_type("embedding_model",DEFAULT_EMBEDDING_MODEL);
    let llm = LLM_embeding_config(llm_type);
    llm
}
fn get_llm_type(name:&str,default_model:&str)->LLMType{
    let setting_cgf = read_config_crypt(LLM_API_SETTING);
    let llm_string = setting_cgf.get(name).unwrap_or(&(default_model.to_owned())).to_string();
    let mut m = llm_string.split("::");
    let model_type = m.next().unwrap();
    let model_name = m.next().unwrap();
    let apikey = setting_cgf.get(&format!("{}_{}",model_type,"api_key")).unwrap_or(&"".to_owned()).to_owned();
    let llm_type = LLMType::get_model(model_type.to_string(), model_name.to_string(),apikey); 
    llm_type
} 
fn get_default_history_result()->Vec<Message>{
    let files = get_files_in_dir(CHAT_HISTORY_DIR);
    let mut results:Vec<Message> = files.into_par_iter().map(|file|{
        let all_his = get_history(&file);
        let ret = all_his[all_his.len()-1].clone();
        ret
    }).collect();
    results.sort_by(|a, b| a.id.cmp(&b.id));
    results
}
  
fn get_history(filename:&str)->Vec<Message>{
    let text = read_string_from_txt(filename);
    let deserialized: Result<Vec<Message>, serde_json::Error> = serde_json::from_str(&text);
    if let Ok(deserialized) = deserialized{
        return deserialized;
    }
    vec![]
}
fn basename_to_history_path(fname:&str)->String{
    CHAT_HISTORY_DIR.to_owned() + fname + ".json"
} 
fn get_all_history()->Vec<Message>{
    let files = get_files_in_dir(CHAT_HISTORY_DIR);
    let results = files.into_par_iter().map(|file|get_history(&file)).flatten().collect();
    results
}
fn get_files_in_dir(dir:&str)->Vec<String>{
    let mut result = vec![];
    let paths = fs::read_dir(dir).unwrap();
    for path in paths{
        let path = path.unwrap().path();
        let filename = path.to_str().unwrap().to_string();
        result.push(filename);
    }
    result
}
fn save_chat_history(history:&Vec<Message>){
    let filename = CHAT_HISTORY_DIR.to_owned() + &get_today_string() + ".json";
    let json_string = serde_json::to_string(history).unwrap();
    write_string_to_txt(&filename, json_string);
}
 
 
fn content_token_limit(history:Vec<Message>,limit:usize )->Vec<Message>{
    let res:(usize,Vec<Message>) = (0,vec![]);
     
    let history_iter = history.into_iter().rev().map(|item|(token_count(&item.content),item)).fold(res,|mut acc, x| { 
        if acc.0 < limit{
            acc.0 = acc.0 + x.0;acc.1.insert(0,x.1);
        } 
        acc
    } ) ;
    
    history_iter.1
}
fn token_count(content:&str)->usize{
    let bpe = p50k_base().unwrap();
    let tokens = bpe.encode_with_special_tokens(content);
    tokens.len()
}
fn get_today_string() -> String {
    let local: DateTime<Local> = Local::now();
    local.format("%d-%m-%Y").to_string()
}

pub fn get_now_timestamp() -> u128 {
    let now = SystemTime::now();

  // Calculate the duration since Unix epoch
  let since_epoch = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();

  // Get the timestamp in seconds
  let timestamp_msecs = since_epoch.as_millis();

  timestamp_msecs
}  
 
fn get_content_chat_item(d: &Message) -> Vec<&str> {
    vec![d.content.as_str()]
}
fn get_data_doc_content(v:&VecResultSearch )->Vec<&str>{
    vec![&v.content]
}  
fn get_content_docs_search(v:&DocSearch )->Vec<&str>{
    vec![&v.content.as_str()]
}  

fn get_content_vector_search(res: &SearchResult) -> &str {   
    if let Metadata::Object(data) = &res.data {
        if let Some(Metadata::Text(s)) = data.get("content") {
            return s;
        }
    } 
    "" 
}
fn get_date_from_timestamp(timestamp: i64) -> String {
    let naive_datetime = NaiveDateTime::from_timestamp(timestamp, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
    datetime.format("%d-%m-%Y").to_string()
} 
   
fn is_eng(inp:&str)->bool{
    if inp.is_empty(){
        return true;
    }
    whichlang::detect_language(inp) == whichlang::Lang::Eng
}

fn is_eng_str(is_en:bool)->&'static str{
    if is_en{
        return "EN";
    }else{
        return "VI";
    }
}
fn truncate_text(text: String, limit: usize) -> String {
    if text.len() <= limit {
        return text;
    }
    
    let truncated:Vec<&str> = text.split(" ").into_iter().enumerate().filter(|item|item.0<limit).map(|item|item.1).collect();
     
    truncated.join(" ") + "..."
}

fn get_basename(path: &str) -> String {
    let path_buf = PathBuf::from(path);
    path_buf.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "".to_string())
}
 