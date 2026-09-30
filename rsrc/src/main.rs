use rdev::{listen, Event, EventType, Key};
use cpal::{StreamConfig, BufferSize};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}, mpsc};
use std::{thread, process};
use std::time::Duration;
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

               let samples = samples_keyboard.lock().unwrap();

               println!("Запись закончена: {} samples", samples.len());
               tx.send(samples.clone()).unwrap();
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
      let samples = rx.recv().unwrap();
      let mut cursamples = Vec::<f32>::new();
      let mut ressamples = Vec::<f32>::new();

      for s in samples {
         cursamples.push(s);
         if cursamples.len() == 480 {
            let res = audio::isSpeech(&cursamples);
            if res {
               ressamples.extend_from_slice(&cursamples);
               println!("Yay")
            }
            cursamples.clear();
         }
      }
   });

   println!("Программа запущена.");
   loop {
        thread::sleep(Duration::from_secs(1));
   }
}