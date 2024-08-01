use chat_templates::{apply_template,Message,ChatTemplate};
use regex::Regex;
pub fn system_prompt_template(template_name: &str, var: Vec<String>) -> String {
    let template = match template_name {
        "chatting" => {
            system_prompt_template_chatting(var[0].clone())
        }
        "simple_system_prompt"=>simple_system_prompt(),
        "get_title_description" => prompt_template_get_title_description(),
       
        _ => "".to_string(),
    };
    template
}
fn simple_system_prompt() -> String {
    let template = format!("You are a helpful assistant.");
    template
}
fn system_prompt_template_chatting(
    context: String,
) -> String {
    let template = format!("You are an AI Assistant that help everyone by answering questions, and improve your answers from previous answers and CONTEXT information below.
Answer in the same language the question was asked.Just reply directly, don't say base on history or mention about chat history.  
If you don't know the answer, just say that you don't know.
----------------------------------------
CONTEXT: {context}
----------------------------------------
");
    template
}
fn prompt_template_get_title_description() -> String {
    let template = format!(" 
Base on Document below, give me back title, description and 2 closest categories of the document.
The Answer should on format like this:
<title>your title here</title>
<description>your short description here</description>
<categories>category 1,category 2</categories>
More example Answer output:
<title>Rust programing technic for beginner</title>
<description>This document explores the evolving landscape of programming languages, focusing on the Rust programming language. It discusses the unique features of Rust, including its performance, type system, memory safety, concurrency model, and interoperability with C. The document also addresses the philosophical and practical reasons for learning Rust and its impact on programming practices.</description>
<categories>Rust,System programing</categories>
----
<title>Organizing Principles of Knowledge Graphs</title>
<description> This document introduces the concept of knowledge graphs, explaining how they transform regular graphs by applying organizing principles that make data smarter. It explores various methods to organize data in graphs to solve complex problems efficiently and discusses the advantages of encoding intelligent behavior directly into the data.</description>
<categories>knowledge graphs,NeoJ</categories>
---------------------------------------------------------------
");
    template
}

pub fn extract_title_description_category(context: String) -> (String, String, String) {
    let mut res = ("".to_owned(), "".to_owned(), "".to_owned());
    let re_title = Regex::new(r"<title>(.*?)</title>").unwrap();

    if let Some(caps) = re_title.captures(&context) {
        if let Some(cap) = caps.get(1) {
            println!("title: {}", cap.as_str());
            res.0 = cap.as_str().to_string();
        }
    }

    let re_description = Regex::new(r"<description>(.*?)</description>").unwrap();
    if let Some(caps) = re_description.captures(&context) {
        if let Some(cap) = caps.get(1) {
            println!("description: {}", cap.as_str());
            res.1 = cap.as_str().to_string();
        }
    }
    let re_categories = Regex::new(r"<categories>(.*?)</categories>").unwrap();
    if let Some(caps) = re_categories.captures(&context) {
        if let Some(cap) = caps.get(1) {
            println!("categories: {}", cap.as_str());
            res.2 = cap.as_str().to_string();
        }
    }
    res
}

 