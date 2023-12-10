use godot::prelude::*;

mod collapser;
mod models;
// mod worker; // TODO - remove

struct LiveWFC;

#[gdextension]
unsafe impl ExtensionLibrary for LiveWFC {}

// use godot::engine::ISprite2D;
// use godot::engine::Sprite2D;

// #[derive(GodotClass)]
// #[class(base=Sprite2D)]
// struct Player {
//     speed: f64,
//     angular_speed: f64,

//     #[base]
//     sprite: Base<Sprite2D>,
// }

// #[godot_api]
// impl ISprite2D for Player {
//     fn init(sprite: Base<Sprite2D>) -> Self {
//         godot_print!("Hello, world!"); // Prints to the Godot console

//         Self {
//             speed: 400.0,
//             angular_speed: std::f64::consts::PI,
//             sprite,
//         }
//     }

//     fn physics_process(&mut self, delta: f64) {
//         self.sprite.rotate((self.angular_speed * delta) as f32);

//         let rotation = self.sprite.get_rotation();
//         let velocity = Vector2::UP.rotated(rotation) * self.speed as f32;
//         self.sprite.translate(velocity * delta as f32);
//     }
// }

// #[godot_api]
// impl Player {
//     #[func]
//     fn increase_speed(&mut self, amount: f64) {
//         self.speed += amount;
//         self.sprite.emit_signal("speed_increased".into(), &[]);
//     }

//     #[signal]
//     fn speed_increased();
// }
