use godot::prelude::*;

mod driver;
mod models;
mod worker;

struct LiveWFC;

#[gdextension]
unsafe impl ExtensionLibrary for LiveWFC {}
