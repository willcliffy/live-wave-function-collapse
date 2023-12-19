#[cfg(test)]
mod tests {
    use godot::builtin::Vector3i;

    use crate::worker::chunk::Chunk;

    #[test]
    fn test_get_overlapping() {
        struct GetOverlappingTest {
            name: String,
            chunk1: Chunk,
            chunk2: Chunk,
            expected: Vec<Vector3i>,
        }

        let runner = |mut tests: Vec<GetOverlappingTest>| {
            for test in tests.iter_mut() {
                let mut overlapping = test.chunk1.get_overlapping(&test.chunk2);
                overlapping.sort();
                test.expected.sort();
                assert_eq!(test.expected, overlapping, "Test Failed: {}", test.name);
            }
        };

        let tests: Vec<GetOverlappingTest> = vec![
            GetOverlappingTest {
                name: "no overlap".into(),
                chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 1 }),
                chunk2: Chunk::new(Vector3i { x: 5, y: 5, z: 5 }, Vector3i { x: 1, y: 1, z: 1 }),
                expected: vec![],
            },
            GetOverlappingTest {
                name: "self-contained".into(),
                chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 1 }),
                chunk2: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 1 }),
                expected: vec![Vector3i { x: 0, y: 0, z: 0 }],
            },
            GetOverlappingTest {
                name: "subset".into(),
                chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 3, y: 3, z: 3 }),
                chunk2: Chunk::new(Vector3i { x: 1, y: 1, z: 1 }, Vector3i { x: 3, y: 3, z: 3 }),
                expected: vec![
                    Vector3i { x: 1, y: 1, z: 1 },
                    Vector3i { x: 1, y: 1, z: 2 },
                    Vector3i { x: 1, y: 2, z: 1 },
                    Vector3i { x: 1, y: 2, z: 2 },
                    Vector3i { x: 2, y: 1, z: 1 },
                    Vector3i { x: 2, y: 1, z: 2 },
                    Vector3i { x: 2, y: 2, z: 1 },
                    Vector3i { x: 2, y: 2, z: 2 },
                ],
            },
            GetOverlappingTest {
                name: "true overlap (y-direction)".into(),
                chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 2, y: 2, z: 2 }),
                chunk2: Chunk::new(Vector3i { x: 0, y: 1, z: 0 }, Vector3i { x: 2, y: 3, z: 2 }),
                expected: vec![
                    Vector3i { x: 0, y: 1, z: 0 },
                    Vector3i { x: 0, y: 1, z: 1 },
                    Vector3i { x: 1, y: 1, z: 0 },
                    Vector3i { x: 1, y: 1, z: 1 },
                ],
            },
            GetOverlappingTest {
                name: "true overlap (x-direction)".into(),
                chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 2, y: 2, z: 2 }),
                chunk2: Chunk::new(Vector3i { x: 1, y: 0, z: 0 }, Vector3i { x: 3, y: 2, z: 2 }),
                expected: vec![
                    Vector3i { x: 1, y: 0, z: 0 },
                    Vector3i { x: 1, y: 0, z: 1 },
                    Vector3i { x: 1, y: 1, z: 0 },
                    Vector3i { x: 1, y: 1, z: 1 },
                ],
            },
            GetOverlappingTest {
                name: "true overlap (z-direction)".into(),
                chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 2, y: 2, z: 2 }),
                chunk2: Chunk::new(Vector3i { x: 0, y: 0, z: 1 }, Vector3i { x: 2, y: 2, z: 3 }),
                expected: vec![
                    Vector3i { x: 0, y: 0, z: 1 },
                    Vector3i { x: 0, y: 1, z: 1 },
                    Vector3i { x: 1, y: 0, z: 1 },
                    Vector3i { x: 1, y: 1, z: 1 },
                ],
            },
        ];

        runner(tests);
    }

    #[test]
    fn test_get_neighbors() {
        struct GetNeighborsTest {
            name: String,
            chunk1: Chunk,
            chunk2: Chunk,
            chunk_overlap: i32,
            expected: Vec<Vector3i>,
        }

        let runner = |mut tests: Vec<GetNeighborsTest>| {
            for test in tests.iter_mut() {
                let mut overlapping = test.chunk1.get_neighbors(&test.chunk2, test.chunk_overlap);
                overlapping.sort();
                test.expected.sort();
                assert_eq!(test.expected, overlapping, "Test Failed: {}", test.name);
            }
        };

        let tests: Vec<GetNeighborsTest> = vec![
            GetNeighborsTest {
                name: "no neighbors".into(),
                chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 1 }),
                chunk2: Chunk::new(Vector3i { x: 5, y: 5, z: 5 }, Vector3i { x: 1, y: 1, z: 1 }),
                chunk_overlap: 1,
                expected: vec![],
            },
            GetNeighborsTest {
                name: "same chunk - no neighbors".into(),
                chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 1 }),
                chunk2: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 1 }),
                chunk_overlap: 1,
                expected: vec![],
            },
            GetNeighborsTest {
                name: "one neighbor, +x".into(),
                chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 2, y: 1, z: 1 }),
                chunk2: Chunk::new(Vector3i { x: 1, y: 0, z: 0 }, Vector3i { x: 2, y: 1, z: 1 }),
                chunk_overlap: 1,
                expected: vec![Vector3i { x: 2, y: 0, z: 0 }],
            },
            GetNeighborsTest {
                name: "one neighbor, +y".into(),
                chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 2, z: 1 }),
                chunk2: Chunk::new(Vector3i { x: 0, y: 1, z: 0 }, Vector3i { x: 1, y: 2, z: 1 }),
                chunk_overlap: 1,
                expected: vec![Vector3i { x: 0, y: 2, z: 0 }],
            },
            GetNeighborsTest {
                name: "one neighbor, +z".into(),
                chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 2 }),
                chunk2: Chunk::new(Vector3i { x: 0, y: 0, z: 1 }, Vector3i { x: 1, y: 1, z: 2 }),
                chunk_overlap: 1,
                expected: vec![Vector3i { x: 0, y: 0, z: 2 }],
            },
            GetNeighborsTest {
                name: "one neighbor, -x".into(),
                chunk1: Chunk::new(Vector3i { x: 1, y: 0, z: 0 }, Vector3i { x: 2, y: 1, z: 1 }),
                chunk2: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 2, y: 1, z: 1 }),
                chunk_overlap: 1,
                expected: vec![Vector3i { x: 0, y: 0, z: 0 }],
            },
            GetNeighborsTest {
                name: "one neighbor, -y".into(),
                chunk1: Chunk::new(Vector3i { x: 0, y: 1, z: 0 }, Vector3i { x: 1, y: 2, z: 1 }),
                chunk2: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 2, z: 1 }),
                chunk_overlap: 1,
                expected: vec![Vector3i { x: 0, y: 0, z: 0 }],
            },
            GetNeighborsTest {
                name: "one neighbor, -z".into(),
                chunk1: Chunk::new(Vector3i { x: 0, y: 0, z: 1 }, Vector3i { x: 1, y: 1, z: 2 }),
                chunk2: Chunk::new(Vector3i { x: 0, y: 0, z: 0 }, Vector3i { x: 1, y: 1, z: 2 }),
                chunk_overlap: 1,
                expected: vec![Vector3i { x: 0, y: 0, z: 0 }],
            },
        ];

        runner(tests);
    }
}
