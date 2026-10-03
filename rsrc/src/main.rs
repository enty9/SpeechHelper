use rdev::{listen, Event, EventType, Key};
use cpal::{StreamConfig, BufferSize};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}, mpsc};
use std::{thread, process};
use std::time::Duration;
use whisper_rs::{WhisperContext, WhisperContextParameters};
mod audio;

fn main() {
   let recording = Arc::new(AtomicBool::new(false));
   let samples = Arc::new(Mutex::new(Vec::<f32>::new()));

   let recording_keyboard = Arc::clone(&recording);
   let samples_keyboard = Arc::clone(&samples);

   let (tx, rx) = mpsc::channel();


   let record_audio = thread::spawn(move || {
      let callback = move |event: Event| {
         match event.event_type {
            EventType::KeyPress(Key::MetaLeft) => {
               // Уже записываем — ничего не делаем
               if recording_keyboard.load(Ordering::Relaxed) {
                  return;
               }

               println!("Начало записи");

               // Очищаем старую запись
               samples_keyboard.lock().unwrap().clear();

               recording_keyboard.store(true, Ordering::Relaxed);

               let recording = Arc::clone(&recording_keyboard);
               let samples = Arc::clone(&samples_keyboard);

               thread::spawn(move || {
                  if let Err(e) = audio::getAudio(
                            recording,
                            samples,
                        ) {
                  eprintln!("Ошибка записи: {}", e);
               }});
            }

            EventType::KeyRelease(Key::MetaLeft) => {
               recording_keyboard.store(false, Ordering::Relaxed);

               let samples = {
                  let samples = samples_keyboard.lock().unwrap();

                  println!(
                        "Запись закончена: {} samples",
                        samples.len()
                  );

                  samples.clone()
               };

               if tx.send(samples).is_err() {
                  eprintln!("Поток обработки аудио завершён");
               }
            }

            EventType::KeyPress(Key::Escape) => { // Временный костыль
               println!("Программа завершена");
               process::exit(process::id().try_into().unwrap());
            }

            _ => {}
      }};
      listen(callback).expect("Error");
   });


   let speech = thread::spawn(move || {
      let ctx = WhisperContext::new_with_params(
        "/home/enty/Projects/SpeechHelper/rsrc/src/ggml-medium-q8_0.bin",
        WhisperContextParameters::default(),
      ).expect("error");
      while let Ok(samples) = rx.recv() {
         let mut cursamples = Vec::<f32>::new();
         let mut ressamples = Vec::<f32>::new();

         for s in samples {
            cursamples.push(s);
            if cursamples.len() == 480 {
               let res = audio::isSpeech(&cursamples);
               if res {
                  ressamples.extend_from_slice(&cursamples);
               }
               cursamples.clear();
            }
         }
         if !ressamples.is_empty(){
            println!("{}", ressamples.len());
            let text = audio::textToSpeech(ressamples, &ctx);
            println!("{}", text);
         } else {
            println!("Please say something");
         }
      }
   });

   println!("Программа запущена.");
   loop {
        thread::sleep(Duration::from_secs(1));
   }
}