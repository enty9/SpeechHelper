use std::process::Command;
use std::collections::HashMap;

pub fn run(text: &str){
    match text{
        "Открыть браузер" => open_browser(),
        _ => println!("I dont now"),
    }
}

pub fn open_browser(){
    Command::new("firefox").spawn();
}