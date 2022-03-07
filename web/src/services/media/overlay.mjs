// This module provides utility to create an overlay data source.
//
// A data source (DS) consists of a sequence of items. Given a range
// [start, end] (both inclusive, 0-based), we can query DS and obtain
// a sub-sequence of items.
//
// An overlay DS is built on top of an existing DS. For clarity, we call the
// overlay one <top DS>, and the underlying one <bottom DS>.
// One can perform modifications on <top DS>. All modifications applied on
// <top DS> will be dictated and not committed to <bottom DS>.
// One can also query from <top DS>, as if it is a regular DS.
//
// For simplicity, an overlay DS supports two kinds of modifications
// -- additions and deletions. They are maintained in two arrays
// `adds: [[int, [Item]]]` and `dels: [int, any]`. The modifications are applied
// regarding the rules as follows:
//
// 1. Create exactly one "slot" after each item in the sequence of <bottom DS>.
//    The slot after i-th item is named as <slot i>. Specifically, there's a
//    "slot -1" at the very front of the sequence.
// 2. For each addition entry `add`, `add[0]` is a number and `add[1]` is an
//    array of items. All items of `add[1]` will be inserted without changing
//    their order at the place of <slot `add[0]`>.
// 3. For each deletion entry `del`, `del[0]` is a number. The i-th item of the
//    **original** sequence be deleted. `del[1]` is extra information of the
//    deleted item, which is bound to be passed over to the server.
//
// As of concrete examples, check test_overlay.mjs.

import { dispatch, patch } from "../../utils/misc"

export function Bridge(bottom_size, adds, dels) {
  let size =
    bottom_size -
    (dels.length - 1) +
    adds.reduce((s, [_, addx]) => s + addx.length, 0)
  return {
    _internal() {
      return { adds, dels, size }
    },
    // remove range [tstart, tend]
    remove(tstart, tend, extras = []) {
      const [bstart, _, range] = this.range_t2b(tstart, tend, true)
      let id = dels.length - 1
      for (let i = range.length - 1; i >= 0; i--) {
        let x = range[i]
        if (typeof x === "number") {
          x += bstart
          while (dels[id][0] > x) id--
          dels.splice(id + 1, 0, [x, extras[i]])
        } else {
          const {
            handle: { ia, i },
          } = x
          adds[ia][1].splice(i, 1)
          if (!adds[ia][1].length && ia !== adds.length - 1) adds.splice(ia, 1)
        }
      }
      size -= tend - tstart + 1
    },
    // insert items after tstart-th item
    insert(tstart, items) {
      if (tstart === -1) {
        const first = adds[0]
        if (first[0] !== -1) {
          adds.splice(0, 0, [-1, items])
        } else {
          first[1].splice(0, 0, ...items)
        }
      } else if (tstart === size - 1) {
        adds[adds.length - 1][1].splice(+Infinity, 0, ...items)
      } else {
        const [bstart, _, [x, xnext]] = this.range_t2b(tstart, tstart + 1, true)
        let ia, i
        if (typeof x !== "number") {
          const { handle } = x
          ia = handle.ia
          i = handle.i + 1
        } else if (typeof xnext !== "number") {
          const { handle } = xnext
          ia = handle.ia
          i = handle.i
        } else {
          ia = 0
          while (bstart > adds[ia][0]) ia++
          adds.splice(ia, 0, [bstart, []])
          i = 0
        }
        adds[ia][1].splice(i, 0, ...items)
      }
      size += items.length
    },
    size() {
      return size
    },
    range_t2b(tstart, tend, with_handle = false) {
      if (tstart < 0) tstart = 0
      if (tend >= size) tend = size - 1
      if (tstart > tend) return [tstart, tend, []]

      function make_ret() {
        let i = ret.length - 1
        while (i >= 0 && typeof ret[i] !== "number") i--
        return [bstart, bstart + (i === -1 ? -1 : ret[i]), ret]
      }

      let ia = 0
      let id = 0
      let la = adds.length
      let ld = dels.length
      let bi = -2
      let ti = -1
      let ri = -1
      let found_start = false
      let bstart = 0
      const tn = tend - tstart + 1
      const ret = new Array(tn)
      while (ia < la || id < ld) {
        function wrap_addx(i) {
          if (!with_handle) return addx[i]
          else return { handle: { adds, ia, i }, item: addx[i] }
        }

        let ndel = 0
        const deli = (dels[id] || [])[0]
        const [addi, addx] = adds[ia] || []
        const addxl = addx && addx.length
        if (ia == la || (id < ld && deli <= addi)) {
          const defer = ia < la && deli === addi
          id += 1
          if (defer) {
            ndel = 1
          } else {
            if (!found_start && tstart <= ti + deli - bi - 1) {
              found_start = true
              bstart = bi + tstart - ti
              bi = bstart - 1
              ti = tstart - 1
            }
            if (!found_start) {
              ti += deli - bi - 1
              bi = deli
            } else {
              while (ri < tn - 1 && bi < deli - 1) {
                ti++
                bi++
                ri++
                ret[ri] = bi - bstart
              }
              bi += 1
              if (ri === tn - 1) return make_ret()
            }
            continue
          }
        }

        if (!found_start) {
          if (tstart <= addi - bi + ti - ndel) {
            bstart = bi + tstart - ti
            found_start = true
            bi = bstart - 1
            ti = tstart - 1
          } else if (tstart <= addi - bi + ti - ndel + addxl) {
            found_start = true
            bstart = addi + 1
            let i = tstart - (addi - bi + ti - ndel) - 2
            ti += i + 1 + addi - bi - 1
            do {
              ri++
              ti++
              i++
              ret[ri] = wrap_addx(i)
            } while (ri < tn - 1 && i < addxl - 1)
            bi = bstart - 1
            if (ri === tn - 1) return make_ret()
            else {
              bi = addi
              ia++
              continue
            }
          }
        }

        if (!found_start) {
          ti += addi - bi + addxl - ndel
          bi = addi
        } else {
          if (tend <= addi - bi + ti - ndel) {
            while (ri < tn - 1) {
              bi++
              ti++
              ri++
              ret[ri] = bi - bstart
            }
            return make_ret()
          } else {
            while (bi < addi - ndel) {
              bi++
              ti++
              ri++
              ret[ri] = bi - bstart
            }
            bi += ndel
            let i = -1
            while (ri < tn - 1 && i < addxl - 1) {
              i++
              ri++
              ti++
              ret[ri] = wrap_addx(i)
            }
            if (ri === tn - 1) {
              return make_ret()
            }
          }
        }
        ia += 1
      }
    },
  }
}

export class OverlayDS {
  constructor(bottom, bottomSize) {
    this._bottom = bottom
    this._bottomSize = bottomSize
    this._adds = [[bottomSize - 1, []]]
    this._dels = [[-1, null]]
    this._bridge = Bridge(bottomSize, this._adds, this._dels)
  }
  async _query(tstart, tend) {
    const [bstart, bend, range] = this._bridge.range_t2b(tstart, tend)
    if (bstart > bend) return range
    const items = await this._bottom.afterIndex({
      index: bstart,
      limit: bend - bstart + 1,
      includes: true,
    })
    return range.map((x) => (typeof x === "number" ? items[x] : x))
  }
  collect() {
    const { adds, dels } = this._bridge._internal()
    const flatAdds = []
    for (const [_, add] of adds) flatAdds.extend(add)
    const flatDels = dels.slice(1).map(([_, del]) => del)
    return [flatAdds, flatDels]
  }
  remove(tstart, tend, extras = []) {
    return this._bridge.remove(tstart, tend, extras)
  }
  insert(tstart, items) {
    items.forEach((item) => {
      item.id = Symbol()
    })
    return this._bridge.insert(tstart, items)
  }
  // caller should also provide item, so that we don't have to query bottom DS
  moveForward(index, item, extras = []) {
    this.remove(index, index, extras)
    this.insert(index - 2, [item])
  }
  moveBackward(index, item, extras = []) {
    this.remove(index, index, extras)
    this.insert(index, [item])
  }
  inplaceMutate(index, item, extras = []) {
    const x = this._bridge.range_t2b(index, index, true)[2][0]
    if (x.handle) {
      const { adds, ia, i } = x.handle
      item.id = Symbol()
      adds[ia][1][i] = item
    } else {
      this.remove(index, index, extras)
      this.insert(index - 1, [item])
    }
  }
  countAll() {
    return this._bridge.size()
  }
  before(...args) {
    return dispatch({ dt: "beforeDt", index: "beforeIndex" }, this)(...args)
  }
  async beforeIndex({ index, limit = 10, includes = false }) {
    if (!includes) index--
    return await this._query(index - limit + 1, index)
  }
  beforeDt() {
    throw new Error("`.beforeDt()` is not implemented on OverlayDS")
  }
  after(...args) {
    return dispatch({ dt: "afterDt", index: "afterIndex" }, this)(...args)
  }
  async afterIndex({
    index,
    limit = 10,
    includes = false,
    withFirstIndex = false,
  }) {
    if (!includes) index++
    const items = await this._query(index, index + limit - 1)
    return withFirstIndex ? [index, items] : items
  }
  afterDt() {
    throw new Error("`.afterDt()` is not implemented on OverlayDS")
  }
  async at(index) {
    return (await this._query(index, index))[0]
  }
  getHighlightedIndices() {
    // TODO
    return this._bottom.getHighlightedIndices()
  }
}
