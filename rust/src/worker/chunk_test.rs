#[cfg(test)]
mod tests {
    use godot::builtin::Vector3i;

    use crate::worker::chunk::Chunk;

    // #[test]
    // fn test_get_neighboring_cells() {
    //     struct GetNeighboringCellsTest {
    //         name: String,
    //         chunk1: Chunk,
    //         chunk2: Chunk,
    //         map_size: Vector3i,
    //         chunk_overlap: i32,
    //         expected: Vec<Vector3i>,
    //     }

    //     let runner = |mut tests: Vec<GetNeighboringCellsTest>| {
    //         for test in tests.iter_mut() {
    //             let mut overlapping = test.chunk1.get_neighboring_cells(
    //                 &test.chunk2,
    //                 test.map_size,
    //                 test.chunk_overlap,
    //             );
    //             overlapping.sort();
    //             test.expected.sort();
    //             assert_eq!(test.expected, overlapping, "Test Failed: {}", test.name);
    //         }
    //     };

    //     let tests: Vec<GetNeighboringCellsTest> = vec![
    //         GetNeighboringCellsTest {
    //             name: "no neighbors".into(),
    //             chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 1 }),
    //             chunk2: Chunk::new(Vector3i { x: 5, y: 5, z: 5 }, Vector3i { x: 1, y: 1, z: 1 }),
    //             map_size: Vector3i {
    //                 x: 10,
    //                 y: 10,
    //                 z: 10,
    //             },
    //             chunk_overlap: 1,
    //             expected: vec![],
    //         },
    //         GetNeighboringCellsTest {
    //             name: "same chunk - no neighbors".into(),
    //             chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 1 }),
    //             chunk2: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 1 }),
    //             map_size: Vector3i {
    //                 x: 10,
    //                 y: 10,
    //                 z: 10,
    //             },
    //             chunk_overlap: 1,
    //             expected: vec![],
    //         },
    //         GetNeighboringCellsTest {
    //             name: "one neighbor, +x".into(),
    //             chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 2, y: 1, z: 1 }),
    //             chunk2: Chunk::new(Vector3i { x: 1, y: 0, z: 0 }, Vector3i { x: 2, y: 1, z: 1 }),
    //             map_size: Vector3i {
    //                 x: 10,
    //                 y: 10,
    //                 z: 10,
    //             },
    //             chunk_overlap: 1,
    //             expected: vec![Vector3i { x: 2, y: 0, z: 0 }],
    //         },
    //         GetNeighboringCellsTest {
    //             name: "one neighbor, +y".into(),
    //             chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 2, z: 1 }),
    //             chunk2: Chunk::new(Vector3i { x: 0, y: 1, z: 0 }, Vector3i { x: 1, y: 2, z: 1 }),
    //             map_size: Vector3i {
    //                 x: 10,
    //                 y: 10,
    //                 z: 10,
    //             },
    //             chunk_overlap: 1,
    //             expected: vec![Vector3i { x: 0, y: 2, z: 0 }],
    //         },
    //         GetNeighboringCellsTest {
    //             name: "one neighbor, +z".into(),
    //             chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 2 }),
    //             chunk2: Chunk::new(Vector3i { x: 0, y: 0, z: 1 }, Vector3i { x: 1, y: 1, z: 2 }),
    //             map_size: Vector3i {
    //                 x: 10,
    //                 y: 10,
    //                 z: 10,
    //             },
    //             chunk_overlap: 1,
    //             expected: vec![Vector3i { x: 0, y: 0, z: 2 }],
    //         },
    //         GetNeighboringCellsTest {
    //             name: "one neighbor, -x".into(),
    //             chunk1: Chunk::new(Vector3i { x: 1, y: 0, z: 0 }, Vector3i { x: 2, y: 1, z: 1 }),
    //             chunk2: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 2, y: 1, z: 1 }),
    //             map_size: Vector3i {
    //                 x: 10,
    //                 y: 10,
    //                 z: 10,
    //             },
    //             chunk_overlap: 1,
    //             expected: vec![Vector3i { x: 0, y: 0, z: 0 }],
    //         },
    //         GetNeighboringCellsTest {
    //             name: "one neighbor, -y".into(),
    //             chunk1: Chunk::new(Vector3i { x: 0, y: 1, z: 0 }, Vector3i { x: 1, y: 2, z: 1 }),
    //             chunk2: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 2, z: 1 }),
    //             map_size: Vector3i {
    //                 x: 10,
    //                 y: 10,
    //                 z: 10,
    //             },
    //             chunk_overlap: 1,
    //             expected: vec![Vector3i { x: 0, y: 0, z: 0 }],
    //         },
    //         GetNeighboringCellsTest {
    //             name: "one neighbor, -z".into(),
    //             chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 1 }, Vector3i { x: 1, y: 1, z: 2 }),
    //             chunk2: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 2 }),
    //             map_size: Vector3i {
    //                 x: 10,
    //                 y: 10,
    //                 z: 10,
    //             },
    //             chunk_overlap: 1,
    //             expected: vec![Vector3i { x: 0, y: 0, z: 0 }],
    //         },
    //         GetNeighboringCellsTest {
    //             name: "chunk 2 overlaps map boundary".into(),
    //             chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 1 }, Vector3i { x: 1, y: 1, z: 2 }),
    //             chunk2: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 2 }),
    //             map_size: Vector3i { x: 1, y: 1, z: 1 },
    //             chunk_overlap: 1,
    //             expected: vec![Vector3i { x: 0, y: 0, z: 0 }],
    //         },
    //     ];

    //     runner(tests);
    // }

    #[test]
    fn test_contains() {
        struct ContainsTest {
            name: String,
            chunk: Chunk,
            position: Vector3i,
            expected: bool,
        }

        let runner = |mut tests: Vec<ContainsTest>| {
            for test in tests.iter_mut() {
                assert_eq!(
                    test.expected,
                    test.chunk.contains(test.position),
                    "Test Failed: {}",
                    test.name
                );
            }
        };

        let tests: Vec<ContainsTest> = vec![
            ContainsTest {
                name: "chunk contains chunk position".into(),
                chunk: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 1 }),
                position: Vector3i { x: 0, y: 0, z: 0 },
                expected: true,
            },
            ContainsTest {
                name: "chunk does not contain chunk end".into(),
                chunk: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 1 }),
                position: Vector3i { x: 1, y: 1, z: 1 },
                expected: false,
            },
            ContainsTest {
                name: "chunk does not contain 1 + x".into(),
                chunk: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 1 }),
                position: Vector3i { x: 1, y: 0, z: 0 },
                expected: false,
            },
            ContainsTest {
                name: "chunk does not contain 1 + y".into(),
                chunk: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 1 }),
                position: Vector3i { x: 0, y: 1, z: 0 },
                expected: false,
            },
            ContainsTest {
                name: "chunk does not contain 1 + z".into(),
                chunk: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 1 }),
                position: Vector3i { x: 0, y: 0, z: 1 },
                expected: false,
            },
        ];

        runner(tests);
    }
}
