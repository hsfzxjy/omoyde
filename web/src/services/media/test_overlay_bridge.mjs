import { Bridge } from "./overlay_bridge.mjs"

function eq(a, b) {
  let i = 0
  while (i < a.length || i < b.length) {
    if (a[i] !== b[i]) return false
    i++
  }
  return true
}

function test(bridge, testcases) {
  for (const [input, expected] of testcases) {
    const output = bridge.range_t2b(...input).flat()
    if (!eq(output, expected)) {
      console.warn(
        input,
        `failed`,
        `\noutput  :`,
        output,
        `\nexpected:`,
        expected
      )
      process.exit(1)
    } else {
      console.log(input, `passed`)
    }
  }
}

let bridge, testcases

// (-1) [a, b] 0 [] 1 [] 2 [c, d] 3 [e]
//   x                   x
bridge = Bridge(
  4,
  [
    [-1, ["a", "b"]],
    [2, ["c", "d"]],
    [3, ["e"]],
  ],
  [
    [-1, null],
    [2, null],
  ]
)

testcases = [
  [
    [0, 0],
    [0, -1, "a"],
  ],
  [
    [0, 1],
    [0, -1, "a", "b"],
  ],
  [
    [0, 2],
    [0, 0, "a", "b", 0],
  ],
  [
    [0, 3],
    [0, 1, "a", "b", 0, 1],
  ],
  [
    [0, 4],
    [0, 1, "a", "b", 0, 1, "c"],
  ],
  [
    [0, 5],
    [0, 1, "a", "b", 0, 1, "c", "d"],
  ],
  [
    [0, 6],
    [0, 3, "a", "b", 0, 1, "c", "d", 3],
  ],
  [
    [0, 7],
    [0, 3, "a", "b", 0, 1, "c", "d", 3, "e"],
  ],
  [
    [1, 1],
    [0, -1, "b"],
  ],
  [
    [1, 2],
    [0, 0, "b", 0],
  ],
  [
    [1, 3],
    [0, 1, "b", 0, 1],
  ],
  [
    [1, 4],
    [0, 1, "b", 0, 1, "c"],
  ],
  [
    [1, 5],
    [0, 1, "b", 0, 1, "c", "d"],
  ],
  [
    [1, 6],
    [0, 3, "b", 0, 1, "c", "d", 3],
  ],
  [
    [1, 7],
    [0, 3, "b", 0, 1, "c", "d", 3, "e"],
  ],
  [
    [2, 2],
    [0, 0, 0],
  ],
  [
    [2, 3],
    [0, 1, 0, 1],
  ],
  [
    [2, 4],
    [0, 1, 0, 1, "c"],
  ],
  [
    [2, 5],
    [0, 1, 0, 1, "c", "d"],
  ],
  [
    [2, 6],
    [0, 3, 0, 1, "c", "d", 3],
  ],
  [
    [2, 7],
    [0, 3, 0, 1, "c", "d", 3, "e"],
  ],
  [
    [3, 3],
    [1, 1, 0],
  ],
  [
    [3, 4],
    [1, 1, 0, "c"],
  ],
  [
    [3, 5],
    [1, 1, 0, "c", "d"],
  ],
  [
    [3, 6],
    [1, 3, 0, "c", "d", 2],
  ],
  [
    [3, 7],
    [1, 3, 0, "c", "d", 2, "e"],
  ],
  [
    [4, 4],
    [3, 2, "c"],
  ],
  [
    [4, 5],
    [3, 2, "c", "d"],
  ],
  [
    [4, 6],
    [3, 3, "c", "d", 0],
  ],
  [
    [4, 7],
    [3, 3, "c", "d", 0, "e"],
  ],
  [
    [5, 5],
    [3, 2, "d"],
  ],
  [
    [5, 6],
    [3, 3, "d", 0],
  ],
  [
    [5, 7],
    [3, 3, "d", 0, "e"],
  ],
  [
    [6, 6],
    [3, 3, 0],
  ],
  [
    [6, 7],
    [3, 3, 0, "e"],
  ],
  [
    [7, 7],
    [4, 3, "e"],
  ],
]

test(bridge, testcases)

// (-1) [] 0 [] 1 [] 2 [c, d] 3 [e]
//   x          x    x
bridge = Bridge(
  4,
  [
    [-1, []],
    [2, ["c", "d"]],
    [3, ["e"]],
  ],
  [
    [-1, null],
    [1, null],
    [2, null],
  ]
)

testcases = [
  [
    [0, 0],
    [0, 0, 0],
  ],
  [
    [0, 1],
    [0, 0, 0, "c"],
  ],
  [
    [0, 2],
    [0, 0, 0, "c", "d"],
  ],
  [
    [0, 3],
    [0, 3, 0, "c", "d", 3],
  ],
  [
    [0, 4],
    [0, 3, 0, "c", "d", 3, "e"],
  ],

  [
    [1, 1],
    [3, 2, "c"],
  ],
  [
    [1, 2],
    [3, 2, "c", "d"],
  ],
  [
    [1, 3],
    [3, 3, "c", "d", 0],
  ],
  [
    [1, 4],
    [3, 3, "c", "d", 0, "e"],
  ],
  [
    [2, 2],
    [3, 2, "d"],
  ],
  [
    [2, 3],
    [3, 3, "d", 0],
  ],
  [
    [2, 4],
    [3, 3, "d", 0, "e"],
  ],
  [
    [3, 3],
    [3, 3, 0],
  ],
  [
    [3, 4],
    [3, 3, 0, "e"],
  ],
  [
    [4, 4],
    [4, 3, "e"],
  ],
]

test(bridge, testcases)

// bridge = Bridge(4, [[3, []]], [-1])
// bridge.insert(3, "a")
// console.log(bridge.range_t2b(0, -1))
// console.log(bridge._internal())
// bridge.insert(3, "b")
// console.log(bridge._internal())
// console.log(bridge.range_t2b(0, +Infinity))
// bridge.remove(4, 4)
// console.log(bridge.range_t2b(0, +Infinity))
// bridge.remove(0, 3)
// console.log(bridge.range_t2b(0, +Infinity))
