import { nextTick } from "vue"

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
