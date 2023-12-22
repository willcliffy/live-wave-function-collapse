# Live Wave Function Collapse - Rust

Currently:
Driver interacts with Godot and manages the single Worker thread.
The Worker thread contains the map. The map contains cells and chunks. The chunks are allowed to operate on cells through mutable reference to map.

Ideally:
Driver stays as is.
Introduce "Manager" which takes some of the current responsibilities of the Worker. The manager holds the map.
The Worker becomes a pool of Workers which can operate on a chunk at a time. This combines some of the logic of chunk and worker.

Chunks that are not adjacent to or overlapping one another can be run in parallel :noice:
