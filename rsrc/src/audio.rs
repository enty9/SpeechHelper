use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Data, SampleRate, SampleFormat, StreamConfig, BufferSize};
use std::sync::{Arc, Mutex};
use std::time::Duration;

let host = cpal::default_host();
let indevice = host.default_input_device().expect("error");
let outdevice = host.default_output_device().expect("error");

pub fn getAudio(){
    
}

pub fn playAudio(){

}