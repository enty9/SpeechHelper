use rdev::{listen, Event, EventType, Key};

fn main() {
   let callback = |e: Event| { // Создаем каллбек который реагирует на нажатие клавищ
      match e.event_type {
         EventType::KeyPress(key) => {
            if key == Key::MetaLeft{ // Вызываем функцию по нажатию на клавишу
               println!("Ai start");
            }
         }
         _ => {}
      }
   };

   listen(callback).expect("Error"); // Начинаем прослушивать клавиатуру с вызовым этого каллбека и отловом ошибки     
}