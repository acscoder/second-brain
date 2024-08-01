use crate::token_output_stream::TokenOutputStream;
use candle_core::quantized::gguf_file;
use candle_core::{DType, Device, IndexOp, Result, Tensor};
use candle_transformers::generation::{LogitsProcessor, Sampling};
use candle_transformers::models::quantized_llama;
use tokenizers::Tokenizer;

static EN_MODEL_DEFAULT: &str =
    "./model/QuantFactory/Meta-Llama-3-8B/Meta-Llama-3-8B-Instruct.Q5_K_S.gguf";
static VI_MODEL_DEFAULT: &str = "./model/vilm/vinallama-2.7b-chat/vinallama-2.7b-chat_q5_0.gguf";
static EN_MODEL_DEFAULT_FN_CALL:&str ="./model/QuantFactory/Meta-Llama-3-8B-Instruct-function-calling-json/Meta-Llama-3-8B-Instruct-function-calling-json-mode.Q6_K.gguf";

static EN_MODEL_DEFAULT_TOKENIZER: &str = "./model/QuantFactory/Meta-Llama-3-8B/tokenizer.json";
static VI_MODEL_DEFAULT_TOKENIZER: &str = "./model/vilm/vinallama-2.7b-chat/tokenizer.json";
static EN_MODEL_DEFAULT_FN_CALL_TOKENIZER: &str =
    "./model/QuantFactory/Meta-Llama-3-8B-Instruct-function-calling-json/tokenizer.json";

pub fn get_device() -> Device {
    let device = match Device::new_cuda(0) {
        Ok(v) => v,
        Err(_) => Device::Cpu,
    };
    device
}

pub fn load_default_model(model_type: &str) -> anyhow::Result<()> {
    let device = get_device();
    let model_filepath = match model_type {
        "EN" => (EN_MODEL_DEFAULT, EN_MODEL_DEFAULT_TOKENIZER),
        "VI" => (VI_MODEL_DEFAULT, VI_MODEL_DEFAULT_TOKENIZER),
        _ => (EN_MODEL_DEFAULT_FN_CALL, EN_MODEL_DEFAULT_FN_CALL_TOKENIZER),
    };

    let mut file = std::fs::File::open(&model_filepath.0)?;
    let model = gguf_file::Content::read(&mut file).map_err(|e| e.with_path(model_filepath.0))?;
    let mut llama_model = quantized_llama::ModelWeights::from_gguf(model, &mut file, &device)?;
    let tokenizer = Tokenizer::from_file(model_filepath.1).map_err(anyhow::Error::msg)?;
    let mut tos = TokenOutputStream::new(tokenizer);

    let prompt_str = "<|im_start|>system
Bạn là một trợ lí AI hữu ích. Hãy trả lời người dùng một cách chính xác.
<|im_end|>
<|im_start|>user
bài luận về du hành thời gian<|im_end|>
<|im_start|>assistant";

    let tokens = tos
        .tokenizer()
        .encode(prompt_str, true)
        .map_err(anyhow::Error::msg)?;

    let mut pre_prompt_tokens = vec![];
    let prompt_tokens = [&pre_prompt_tokens, tokens.get_ids()].concat();
    let to_sample = 2000;
    let mut all_tokens = vec![];
    let mut logits_processor: LogitsProcessor = LogitsProcessor::from_sampling(1, Sampling::ArgMax);
    let next_token = {
        let input = Tensor::new(prompt_tokens.as_slice(), &device)?.unsqueeze(0)?;
        let logits = llama_model.forward(&input, 0)?;
        let logits = logits.squeeze(0)?;
        logits_processor.sample(&logits)?
    };
     

    let eos_token_id = tos.get_token("").unwrap_or(2);
    let mut tokentostring = token_to_string(tos);
    let mut nexttoken = get_next_token(llama_model, logits_processor, device);

    let mut ntk = nexttoken((next_token, prompt_tokens.len()))?;
 
    loop {
        let t = tokentostring(ntk.0)?;
        println!("token {} - {} index {}", ntk.0,t,ntk.1);  
        
        ntk = nexttoken(ntk)?;
        if ntk.0 == eos_token_id || ntk.1 > to_sample {
            break;
        };   
        all_tokens.push(t); 
    }
    
    println!("{}", all_tokens.join("") );  

   

    Ok(())
}

fn get_next_token(
    mut llama_model: quantized_llama::ModelWeights,
    mut logits_processor: LogitsProcessor,
    device: Device,
) -> Box<dyn FnMut((u32, usize)) -> anyhow::Result<(u32, usize)>> {
    let f = move |tk: (u32, usize)| -> anyhow::Result<(u32, usize)> {
        let input = Tensor::new(&[tk.0], &device)?.unsqueeze(0)?;
        let logits = llama_model.forward(&input, tk.1)?;
        let logits = logits.squeeze(0)?;
        let index = tk.1 + 1;
        let next_token = logits_processor.sample(&logits)?;
        Ok((next_token, index))
    };
    Box::new(f)
}
fn token_to_string(mut tos:TokenOutputStream)->Box<dyn FnMut(u32) -> anyhow::Result<String>> {
    let f = move|next_token:u32|{
        if let Some(t) = tos.next_token(next_token)? {
            return Ok(t);
        }
        return Ok("".to_owned());
    };
    Box::new(f)
}
/* 
fn continuous_repeat_counter(mut counter:Vec<u32>,repeat:usize)->Box<dyn FnMut(u32)->bool >{
    let f = move |t|{
        if counter.len()>0 && counter[counter.len()-1] == t {
            counter.push(t);
        }else {
            counter = vec![t];
        }
        counter.len()>=repeat
    } ;
    Box::new(f)
}*/