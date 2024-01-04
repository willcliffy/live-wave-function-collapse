# Live Wave Function Collapse - Rust

Currently:
Driver interacts with Godot and manages the single Collapser thread.
The Collapser thread contains the map. The map contains cells and chunks. The chunks are allowed to operate on cells through mutable reference to map.

Ideally:
Driver stays as is.
Introduce "Manager" which takes some of the current responsibilities of the Collapser. The manager holds the map.
The Collapser becomes a pool of Collapsers which can operate on a chunk at a time. This combines some of the logic of chunk and collapser.

Chunks that are not adjacent to or overlapping one another can be run in parallel :noice:
