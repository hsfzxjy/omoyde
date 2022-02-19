import { LS } from "./dom"

export class LSCache {
  constructor() {
    this._store = new Map()
  }
  expireAll() {
    for (const key of this._store.keys()) {
      LS.removeItem(key)
    }
    this._store.clear()
  }
  set(key, value) {
    this._store.set(key, value)
    LS.setItem(key, value)
  }
  get(key) {
    if (this._store.has(key)) return this._store.get(key)
    const value = LS.getItem(key)
    this._store.set(key, value)
    return value
  }
}
