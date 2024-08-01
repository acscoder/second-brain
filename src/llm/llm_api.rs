use std::future::Future;
use std::pin::Pin;
use std::vec;
use std::{boxed::Box, rc::Rc};
use crate::read_lines;
use anthropic;
use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};
use openai_api_rs;

use anthropic::{AI_PROMPT, HUMAN_PROMPT};

static MODELS_LIST_DIR: &str = "./db/";
#[derive(Debug)]
pub enum LLMType {
    Ollama(String, String),
    OpenAI(String, String),
    Claude(String, String),
    Gemini(String, String),
}
impl LLMType {
    pub fn get_model(model_type:String,model_name:String,apikey:String) -> Self {
            match model_type.as_str() {
                "OpenAI" => LLMType::OpenAI(model_name, apikey),
               "Claude" => LLMType::Claude(model_name, apikey),
                "Gemini" => LLMType::Gemini(model_name, apikey),
                _ => LLMType::Ollama(model_name , apikey),
            }
        
    }
    pub fn get_model_type_string(&self) -> String {
        match self {
            LLMType::Ollama(_, _) => "Ollama".to_owned(),
            LLMType::OpenAI(_, _) => "OpenAI".to_owned(),
            LLMType::Claude(_, _) => "Claude".to_owned(),
            LLMType::Gemini(_, _) => "Gemini".to_owned(),
            _ => "".to_owned()
        }
    }
   
    pub fn to_string(&self) -> String {
        match self {
            LLMType::Ollama(m, _) => format!("{}::{}","Ollama",m),
            LLMType::OpenAI(m, _) => format!("{}::{}","OpenAI",m),
            LLMType::Claude(m, _) => format!("{}::{}","Claude",m),
            LLMType::Gemini(m, _) => format!("{}::{}","Gemini",m),
            _ => "".to_owned()
        }
    }
}

pub fn get_models_llm_list(llm_type:&str)->Vec<String>{
    let file_path = format!("{}{}_llm_list.txt", MODELS_LIST_DIR, llm_type.to_lowercase());
    read_lines(&file_path)
}
pub fn get_models_embedding_list(llm_type:&str)->Vec<String>{
    let file_path = format!("{}{}_embedding_list.txt", MODELS_LIST_DIR, llm_type.to_lowercase());
    read_lines(&file_path)
}

pub fn LLM_chat_config(cf: LLMType) -> impl Fn(String) -> Pin<Box<dyn Future<Output = String>>> {
    match cf {
        LLMType::Ollama(modal_name, apikey) => chatOllama(modal_name, apikey),
        LLMType::OpenAI(modal_name, apikey) => chatOpenAI(modal_name, apikey),
        LLMType::Claude(modal_name, apikey) => chatClaude(modal_name, apikey),
        LLMType::Gemini(modal_name, apikey) => chatGemini(modal_name, apikey),
    }
}
pub fn LLM_embeding_config(
    cf: LLMType,
) -> impl Fn(String) -> Pin<Box<dyn Future<Output = Vec<f32>>>> {
    match cf {
        LLMType::Ollama(modal_name, apikey) => embedingOllama(modal_name, apikey),
        LLMType::OpenAI(modal_name, apikey) => embedingOpenAI(modal_name, apikey),
        LLMType::Claude(modal_name, apikey) => embedingClaude(modal_name, apikey),
        LLMType::Gemini(modal_name, apikey) => embedingGemini(modal_name, apikey),
    }
}

fn chatOllama(
    model_name: String,
    _apikey: String,
) -> Box<dyn Fn(String) -> Pin<Box<dyn Future<Output = String>>>> {
    let mname = Rc::new(model_name);
    let f = move |input_string: String| -> Pin<Box<dyn Future<Output = String>>> {
        let modelname = mname.clone();
        let ollama = Ollama::default();
        let c = async move {
            let res = ollama.generate(GenerationRequest::new(modelname.to_string(), input_string));
            if let Ok(r) = res.await {
                r.response
            } else {
                "".to_owned()
            }
        };
        Box::pin(c)
    };
    Box::new(f)
}

fn embedingOllama(
    model_name: String,
    _apikey: String,
) -> Box<dyn Fn(String) -> Pin<Box<dyn Future<Output = Vec<f32>>>>> {
    let mname = Rc::new(model_name);
    let f = move |input_string: String| -> Pin<Box<dyn Future<Output = Vec<f32>>>> {
        let modelname = mname.clone();
        let ollama = Ollama::default();

        let c = async move {
            let res = ollama.generate_embeddings(modelname.to_string(), input_string, None);
            if let Ok(r) = res.await {
                r.embeddings.into_iter().map(|x| x as f32).collect()
            } else {
                vec![]
            }
        };
        Box::pin(c)
    };
    Box::new(f)
}

fn chatOpenAI(
    model_name: String,
    apikey: String,
) -> Box<dyn Fn(String) -> Pin<Box<dyn Future<Output = String>>>> {
    let client = openai_api_rs::v1::api::Client::new(apikey);
    let mname = Rc::new(model_name);
    let f = move |input_string: String| -> Pin<Box<dyn Future<Output = String>>> {
        let req = openai_api_rs::v1::chat_completion::ChatCompletionRequest::new(
            mname.to_string(),
            vec![openai_api_rs::v1::chat_completion::ChatCompletionMessage {
                role: openai_api_rs::v1::chat_completion::MessageRole::user,
                content: openai_api_rs::v1::chat_completion::Content::Text(input_string),
                name: None,
            }],
        )
        .max_tokens(3000)
        .temperature(0.9)
        .top_p(1.0)
        .stop(vec![String::from(" Human:"), String::from(" AI:")])
        .presence_penalty(0.6)
        .frequency_penalty(0.0);

        let result = client.chat_completion(req).unwrap();

        let c = result.choices[0]
            .message
            .content
            .clone()
            .unwrap_or("".to_owned());
        Box::pin(to_future(c))
    };
    Box::new(f)
}
fn to_future<T>(data: T) -> impl Future<Output = T> {
    async { data }
}
fn embedingOpenAI(
    model_name: String,
    apikey: String,
) -> Box<dyn Fn(String) -> Pin<Box<dyn Future<Output = Vec<f32>>>>> {
    let api_key = Rc::new(apikey);
    let mname = Rc::new(model_name);
    let f = move |input_string: String| -> Pin<Box<dyn Future<Output = Vec<f32>>>> {
        let modelname = mname.clone();
        let key = api_key.clone();
        let client = openai_api_rs::v1::api::Client::new(key.to_string());
        let req = openai_api_rs::v1::embedding::EmbeddingRequest::new(
            modelname.to_string(),
            input_string,
        );
        let result = client.embedding(req).unwrap();
        let c = result.data[0].embedding.clone();
        Box::pin(to_future(c))
    };
    Box::new(f)
}

fn chatClaude(
    model_name: String,
    apikey: String,
) -> Box<dyn Fn(String) -> Pin<Box<dyn Future<Output = String>>>> {
    let mname = Rc::new(model_name);
    let api_key = Rc::new(apikey);
    let f = move |input_string: String| -> Pin<Box<dyn Future<Output = String>>> {
        let modelname = mname.clone();
        let key = api_key.clone();
        let cfg = anthropic::config::AnthropicConfig {
            api_key: key.to_string(),
            default_model: Some(modelname.to_string()),
            api_base: Some(anthropic::DEFAULT_API_BASE.to_string()),
        };

        let c = async move {
            let prompt = format!("{HUMAN_PROMPT}{input_string}{AI_PROMPT}");
            let client = anthropic::client::Client::try_from(cfg).unwrap();
            let complete_request = anthropic::types::CompleteRequestBuilder::default()
                .prompt(prompt)
                .max_tokens_to_sample(256usize)
                .stream(false)
                .stop_sequences(vec![HUMAN_PROMPT.to_string()])
                .build()
                .unwrap();
            let complete_response = client.complete(complete_request);
            if let Ok(r) = complete_response.await {
                r.completion
            } else {
                "".to_owned()
            }
        };
        Box::pin(c)
    };
    Box::new(f)
}

fn embedingClaude(
    model_name: String,
    apikey: String,
) -> Box<dyn Fn(String) -> Pin<Box<dyn Future<Output = Vec<f32>>>>> {
    let f = move |input_string: String| -> Pin<Box<dyn Future<Output = Vec<f32>>>> { todo!() };
    Box::new(f)
}
fn chatGemini(
    model_name: String,
    apikey: String,
) -> Box<dyn Fn(String) -> Pin<Box<dyn Future<Output = String>>>> {
    let mname = Rc::new(model_name);
    let api_key = Rc::new(apikey);
    //let client = JeminiClient::new(apikey);
    let f = move |input_string: String| -> Pin<Box<dyn Future<Output = String>>> {
        let model_name = mname.clone();
        let key = api_key.clone();
        todo!()
    };
    Box::new(f)
}
fn embedingGemini(
    model_name: String,
    apikey: String,
) -> Box<dyn Fn(String) -> Pin<Box<dyn Future<Output = Vec<f32>>>>> {
    let mname = Rc::new(model_name);
    let api_key = Rc::new(apikey);
    let f = move |input_string: String| -> Pin<Box<dyn Future<Output = Vec<f32>>>> {
        let model_name = mname.clone();
        let key = api_key.clone();
        todo!()
    };
    Box::new(f)
}

pub async fn list_ollama_model() -> Vec<String> {
    let ollama = Ollama::default();
    let res = ollama.list_local_models().await;
    if let Ok(r) = res {
        r.into_iter().map(|x| x.name).collect()
    } else {
        vec![]
    }
}
