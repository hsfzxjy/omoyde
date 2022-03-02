import { store } from "../states"
import { Mutex } from "../utils/aio"
import { mediaDB } from "./media/db"
import { OverlayDS } from "./media/overlay.mjs"

function _getDataSource() {
  return mediaDB
}

const overlay = {
  _ds: null,
  _mutex: new Mutex(),
  _init() {
    return this._mutex.guardOrWait(async () => {
      const ds = _getDataSource()
      const bottomSize = await ds.countAll()
      this._ds = new OverlayDS(ds, bottomSize)
    })
  },
  async get() {
    if (this._ds === null) await this._init()
    return this._ds
  },
  reset() {
    return this._mutex.guardOrWait(async () => {
      this._ds = null
    })
  },
}

export async function getDataSource() {
  while (true) {
    if (store.fragment.editting) {
      const ds = await overlay.get()
      if (!store.fragment.editting) continue
      return ds
    } else {
      await overlay.reset()
      if (store.fragment.editting) continue
      return _getDataSource()
    }
  }
}
