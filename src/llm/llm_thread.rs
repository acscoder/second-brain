use std::thread;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::JoinHandle;

 
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::boxed::Box;
  

use crate::{set_arc_mutex_var,coddition_arc_mutex_var}; 

pub fn get_llm_thread()->(Box<dyn Fn(String)>,Receiver<String>,Arc<Mutex<bool>>){
    let input =   Arc::new(Mutex::new("".to_owned()));
    let kill_llm_terminal =   Arc::new(Mutex::new(false));
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let llm  = setup_llm(input.clone(),tx,kill_llm_terminal.clone());
    let llm_thread = llm_thread_unpark(llm,input.clone());
    (llm_thread,rx,kill_llm_terminal)    
}
fn setup_llm(input:Arc<Mutex<String>>,thread_tx:Sender<String>,kill:Arc<Mutex<bool>>)->JoinHandle<()> {
    thread::spawn(move || {
        let back = input.clone();
        //llm load here
         
        set_arc_mutex_var!(kill,false);

        loop{
          
            coddition_arc_mutex_var!(kill,true,break);

            let input_lock = back.clone();
            let mut inp = input_lock.lock().unwrap();
            
            if *inp != "" {
                dbg!(&inp);
                thread_tx.send("llm here".to_string()).unwrap();
                *inp = "".to_owned();
               
            }else{
                println!("waiting");
                drop(inp);
                drop(input_lock);
                thread::park();
            }
        }
    }) 
}

fn llm_thread_unpark(handle: JoinHandle<()>,oinput:Arc<Mutex<String>>) ->Box<dyn Fn(String)>{
    Box::new(move |text| {
        let input = oinput.clone();
        let mut inp = input.lock().unwrap();
        *inp = text;
        _ = &handle.thread().unpark();
    })
} 

/*
 let (llm_thread,rx,kill_llm_terminal)   = get_llm_thread();
    for i in 0..10 {
        if i == 5 {
            set_arc_mutex_var!(kill_llm_terminal,true);
            llm_thread("".to_owned());
        }
        let formatted_string = format!("Hello, {}!", i);
        llm_thread(formatted_string);
        if let Ok(x) = rx.recv(){
            println!("{}", x); 
        }
    }    
*/