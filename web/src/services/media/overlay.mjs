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
import { Bridge } from "./overlay_bridge.mjs"

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
    const ret = range.map((x, idx) => {
      const isAdded = typeof x !== "number"
      const item = isAdded ? patch(x, { id: Symbol() }) : items[x]
      patch(item, {
        isAdded,
        delBefore: false,
        delAfter: !!das[idx],
      })
      return item
    })
    if (ret.length && tstart <= 0 && this._bridge.has_del_at_front())
      ret[0].delBefore = true
    return ret
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
