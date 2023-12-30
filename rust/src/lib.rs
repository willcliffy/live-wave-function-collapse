use godot::prelude::*;

mod driver;
mod manager;
mod map;
mod models;
mod worker;

struct LiveWFC;

#[gdextension]
unsafe impl ExtensionLibrary for LiveWFC {}
