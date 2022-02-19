import { nextTick } from "vue"
import { Mutex } from "./aio"
import { debounce, noop, stringifyAsKey } from "./misc"

export class ScrollTopRecoverer {
  constructor(scrollableGetter, anchorGetter) {
    this._scrollableGetter = scrollableGetter
    this._anchorGetter = anchorGetter
  }
  dictate() {
    const $scrollable = this._scrollableGetter()
    const $anchor = this._anchorGetter()
    const oldAnchorOffsetTop = $anchor.offsetTop
    const oldScrollTop = $scrollable.scrollTop
    return async () => {
      await nextTick()
      const $scrollable = this._scrollableGetter()
      const $anchor = this._anchorGetter()
      const newAnchorOffsetTop = $anchor.offsetTop
      $scrollable.scrollTop =
        oldScrollTop - oldAnchorOffsetTop + newAnchorOffsetTop
    }
  }
}

export class DebouncedIntersectionObserver extends IntersectionObserver {
  constructor({ onEvent, handle, setupCtx, options }) {
    const { timeout = 150 } = options
    const mutex = new Mutex()
    const deb = debounce(
      (...rest) => mutex.guard(() => handle(...rest)),
      timeout,
      setupCtx
    )
    const cb = (entries) =>
      mutex.guard(() => {
        onEvent(entries, deb.ctx)
        deb.invoke()
      })

    super(cb, options)
    this._deb = deb
  }
  nextTick() {
    return this._deb.nextTick()
  }
}

export const LS = {
  setItem(key, value) {
    key = stringifyAsKey(key)
    return window.localStorage.setItem(key, JSON.stringify(value))
  },
  getItem(key) {
    key = stringifyAsKey(key)
    const value = window.localStorage.getItem(key)
    if (value === null) return null
    return JSON.parse(value)
  },
  hasItem(key) {
    key = stringifyAsKey(key)
    return window.localStorage.hasItem(key)
  },
  removeItem(key) {
    key = stringifyAsKey(key)
    return window.localStorage.removeItem(key)
  },
}
