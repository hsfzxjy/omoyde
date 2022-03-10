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

import { reactive, ref, toRaw } from "vue"
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
    range_t2b(tstart, tend, with_handle = false, with_da = false) {
      if (tstart < 0) tstart = 0
      if (tend >= size) tend = size - 1
      if (tstart > tend) return [tstart, tend, []]

      function make_ret() {
        let i = ret.length - 1
        while (i >= 0 && typeof ret[i] !== "number") i--
        const new_r = [bstart, bstart + (i === -1 ? -1 : ret[i]), ret]
        return with_da ? [new_r, das] : new_r
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
      const das = with_da ? new Array(tn) : []
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
              if (with_da && bi === deli - 1 && ri >= 0) das[ri] = true
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
            if (with_da && i === -1 && ri >= 0 && ndel) das[ri] = true
            do {
              ri++
              ti++
              i++
              ret[ri] = wrap_addx(i)
            } while (ri < tn - 1 && i < addxl - 1)
            if (
              with_da &&
              i === addxl - 1 &&
              id < ld &&
              dels[id][0] === addi + 1
            )
              das[ri] = true
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
            if (with_da && tend <= addi - bi + ti - ndel && ndel && ri >= 0)
              das[ri] = true
            return make_ret()
          } else {
            while (bi < addi - ndel) {
              bi++
              ti++
              ri++
              ret[ri] = bi - bstart
            }
            if (with_da && ndel && ri >= 0) das[ri] = true
            bi += ndel
            let i = -1
            while (ri < tn - 1 && i < addxl - 1) {
              i++
              ri++
              ti++
              ret[ri] = wrap_addx(i)
            }
            if (
              with_da &&
              i === addxl - 1 &&
              id < ld &&
              dels[id][0] === addi + 1
            )
              das[ri] = true
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
    this._size = ref(bottomSize)
    this._hls = reactive([])
    this._bottomHls = null
  }
  _sync() {
    if (this._bottomHls === null) {
      throw new Error("`_hls` not initialized before mods are made")
    }
    this._size.value = this._bridge.size()
    const adds = this._adds
    const dels = this._dels
    const hls = this._bottomHls
    const la = adds.length
    const ld = dels.length
    const lh = hls.length
    let delta = 0
    let idx
    let ia = 0
    let id = 1
    let ih = 0
    let ret = []
    let prev = []
    while (ia < la || id < ld || ih < lh) {
      const [addi, addx] = adds[ia] || []
      const deli = (dels[id] || [])[0]
      const [hlsi, hlsx] = hls[ih] || []
      let mask =
        ((addi !== undefined) << 2) |
        ((deli !== undefined) << 1) |
        (hlsi !== undefined)
      inner: while (true)
        switch (mask) {
          case 0b000:
            throw new Error("unreachable")
          case 0b001:
            ret.push((prev = [hlsi + delta, hlsx]))
            ih++
            break inner
          case 0b010:
            idx = deli + delta
            if (prev[1] !== "del" || idx !== prev[0]) {
              ret.push((prev = [idx, "del"]))
            }
            delta--
            id++
            break inner
          case 0b011:
            if (deli <= hlsi) {
              mask = 0b010
              if (deli === hlsi) ih++
            } else {
              mask = 0b001
            }
            continue inner
          default:
            if (
              (mask & 0b010 && addi >= deli) ||
              (mask & 0b001 && addi >= hlsi)
            ) {
              mask &= 0b011
              continue inner
            }
            for (let i = 0; i < addx.length; i++) {
              delta++
              ret.push((prev = [delta + addi, "add"]))
            }
            ia++
            break inner
        }
    }
    this._hls.replaceAll(ret)
  }
  async _query(tstart, tend) {
    const [[bstart, bend, range], das] = this._bridge.range_t2b(
      tstart,
      tend,
      false,
      true
    )
    if (bstart > bend) return range
    const items = await this._bottom.afterIndex({
      index: bstart,
      limit: bend - bstart + 1,
      includes: true,
    })
    return range.map((x, idx) => {
      const isAdded = typeof x !== "number"
      const item = isAdded ? patch(x, { id: Symbol() }) : items[x]
      patch(item, {
        isAdded,
        delAfter: !!das[idx],
      })
      return item
    })
  }
  collect() {
    const { adds, dels } = this._bridge._internal()
    const flatAdds = []
    for (const [_, add] of adds) flatAdds.extend(add)
    const flatDels = dels.slice(1).map(([_, del]) => del)
    return [flatAdds, flatDels]
  }
  remove(tstart, tend, extras = []) {
    this._bridge.remove(tstart, tend, extras)
    this._sync()
  }
  insert(tstart, items) {
    items.forEach((item) => {
      item.id = Symbol()
    })
    this._bridge.insert(tstart, items)
    this._sync()
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
    return this._size
  }
  before(...args) {
    return dispatch({ index: "beforeIndex" }, this)(...args)
  }
  async beforeIndex({ index, limit = 10, includes = false }) {
    if (!includes) index--
    return await this._query(index - limit + 1, index)
  }
  after(...args) {
    return dispatch({ index: "afterIndex" }, this)(...args)
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

  async at(index) {
    return (await this._query(index, index))[0]
  }
  async getHlItems() {
    if (this._bottomHls === null) {
      const bottomHls = await this._bottom.getHlItems()
      this._bottomHls = toRaw(bottomHls)
      this._sync()
    }
    return this._hls
  }
}
