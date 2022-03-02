import { patch } from "../../utils/misc"

export const ITEM_NULL = Symbol("ITEM_NULL")
export const ITEM_UNKNOWN = Symbol("ITEM_UNKNOWN")

class NeighborAnnotator {
  constructor({ items, tracker }) {
    this._items = items
    this._tracker = tracker
  }

  preproc(items) {
    const unknown = () => ({ dt: null, kind: ITEM_UNKNOWN })
    items.forEach((item) => {
      item.prev = [unknown(), unknown()]
      item.next = [unknown(), unknown()]
    })
  }

  postproc() {
    const items = this._items
    const tracker = this._tracker

    const L = items.length
    if (!L) return

    const ITEM_MARKER_PREV = tracker.atStart ? ITEM_NULL : ITEM_UNKNOWN
    const ITEM_MARKER_NEXT = tracker.atEnd ? ITEM_NULL : ITEM_UNKNOWN
    const fillExcerpt = (excerpt, i, marker) => {
      if (i < 0 || i >= L) {
        excerpt.kind = marker
        return
      }
      const { dt, kind } = items[i]
      patch(excerpt, { dt, kind })
    }

    for (let i = 0; i < L; i++) {
      const { prev, next } = items[i]
      if (prev[0].kind !== ITEM_UNKNOWN && prev[1].kind !== ITEM_UNKNOWN) break

      fillExcerpt(prev[0], i - 1, ITEM_MARKER_PREV)
      fillExcerpt(prev[1], i - 2, ITEM_MARKER_PREV)
      fillExcerpt(next[0], i + 1, ITEM_MARKER_NEXT)
      fillExcerpt(next[1], i + 2, ITEM_MARKER_NEXT)
    }
    for (let i = L - 1; i >= 0; --i) {
      const { prev, next } = items[i]
      if (next[0].kind !== ITEM_UNKNOWN && next[1].kind !== ITEM_UNKNOWN) break

      fillExcerpt(prev[0], i - 1, ITEM_MARKER_PREV)
      fillExcerpt(prev[1], i - 2, ITEM_MARKER_PREV)
      fillExcerpt(next[0], i + 1, ITEM_MARKER_NEXT)
      fillExcerpt(next[1], i + 2, ITEM_MARKER_NEXT)
    }
  }
}

export class MediaLocalView {
  constructor({ dataSource, items, tracker, limit }) {
    this._ds = dataSource
    this._items = items
    this._tracker = tracker
    this._limit = limit
    this._annotator = new NeighborAnnotator({ items, tracker })
  }

  async jumpTo({ loadForward, targetIndex }) {
    const nForward = loadForward ? Math.min(this._limit, targetIndex) : 0
    const [globalIndex, items] = await this._ds.after({
      index: targetIndex - nForward,
      limit: this._limit + 1 + nForward,
      includes: true,
      withFirstIndex: true,
    })

    this._annotator.preproc(items)
    this._items.replaceAll(items)
    this._annotator.postproc()
    if (items.length) {
      const first = items[0]
      patch(this._tracker, {
        date: +first.dt,
        pid: first.pid,
        offset: 0,
        localIndex: nForward,
      })
    }
    patch(this._tracker, {
      globalIndex: globalIndex + nForward,
      atStart: targetIndex === nForward,
      atEnd: items.length < this._limit + 1 + nForward,
    })
  }

  async loadForward() {
    const tracker = this._tracker

    // no more items to pull
    if (tracker.atStart) return false

    const breach = this._limit - tracker.localIndex
    // items are adequate, no need to pull
    if (breach <= 0) return false

    const items = await this._ds.before({
      index: tracker.globalIndex - tracker.localIndex,
      limit: breach,
    })
    this._annotator.preproc(items)

    // if no new items, we reach the very beginning
    if (items.length < breach) tracker.atStart = true
    // new items would be prepended, so we increase localIndex
    tracker.localIndex += items.length
    // prepend new items
    this._items.unshift.apply(this._items, items)
    // drop redundant items at the tail
    const numToSplice =
      this._items.length - 1 - tracker.localIndex - this._limit
    if (numToSplice > 0) {
      this._items.splice(-numToSplice, numToSplice)
      tracker.atEnd = false
    }
    this._annotator.postproc()

    return true
  }

  async loadBackward() {
    const tracker = this._tracker

    // no more items to pull
    if (tracker.atEnd) return false

    const breach = this._limit - (this._items.length - 1 - tracker.localIndex)
    // items are adequate, no need to pull
    if (breach <= 0) return false

    const items = await this._ds.after({
      index: tracker.globalIndex + this._items.length - 1 - tracker.localIndex,
      limit: breach,
    })
    this._annotator.preproc(items)

    // if no new items, we reach the very end
    if (items.length < breach) tracker.atEnd = true
    // since new items would be appended, we don't have to mutate localIndex
    // append new items
    this._items.extend(items)
    // drop excess items at the front
    const numToSplice = tracker.localIndex - this._limit
    if (numToSplice > 0) {
      this._items.splice(0, numToSplice)
      tracker.localIndex -= numToSplice
      tracker.atStart = false
    }
    this._annotator.postproc()

    return true
  }
}
