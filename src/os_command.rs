use std::process::Command;
use tempfile::NamedTempFile;
use std::io::Write;

pub fn install_ollama() {
    if cfg!(target_os = "windows"){

    } else if cfg!(target_os = "macos") {

    }else{
        install_ollama_linux(); 
    }
}
fn install_ollama_linux(){
    if !check_ollama_linux(){
        let mut curl_command = Command::new("curl");
        curl_command.args(["-fsSL", "https://ollama.com/install.sh"]);

        let output = curl_command.output().expect("failed to execute curl");

        if output.status.success() {
            let script_content = String::from_utf8_lossy(&output.stdout);

            // Create a temporary file
            let mut temp_file = NamedTempFile::new().expect("failed to create temporary file");

            // Write the downloaded script to the temporary file
            temp_file.write_all(script_content.as_bytes()).expect("failed to write to temporary file");

            // Execute the script with safety checks
            let mut sh_command = Command::new("sh");
            sh_command.arg(temp_file.path());

            let sh_output = sh_command.status().expect("failed to execute sh");

            if sh_output.success() {
                println!("Script executed successfully");
            } else {
                println!("Error executing script: {}", sh_output);
            }
        } else {
            println!("Error downloading script: {}", output.status);
        }

    }
}
fn check_ollama_linux()->bool{
    let output = Command::new("ollama").arg("-v").status();
    match output {
        Ok(status) => {
            if !status.success() {
                return false;
            }
        }
        Err(_) => {
            return false;
        }
    }
    true
}

pub fn ollama_command(commands:&[&str])->Option<String>{
    let output = Command::new("ollama").args(commands).output();
    if let Ok(output) = output {
        if output.status.success(){
            return Some(String::from_utf8_lossy(&output.stdout).to_string());
        }
    };
    None
}
