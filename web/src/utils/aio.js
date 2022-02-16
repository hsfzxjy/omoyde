export const sleep = (milliseconds) =>
  new Promise((resolve) => {
    setTimeout(resolve, milliseconds)
  })

export const setImmediate = (cb) => setTimeout(cb, 0)

export class Deferred {
  constructor() {
    this._resolve = null
    this._promise = null
  }
  pending() {
    return this._promise !== null
  }
  wait() {
    if (this._promise === null) return
    return this._promise
  }
  reset() {
    if (this._promise !== null)
      throw new Error("cannot reset a pending Deferred")
    this._promise = new Promise((resolve) => (this._resolve = resolve))
  }
  resolve() {
    if (this._promise === null)
      throw new Error("cannot resolve a resolved Deferred")
    this._resolve()
    this._promise = this._resolve = null
  }
}

export class Mutex {
  constructor() {
    this._deferred = new Deferred()
  }
  async lock() {
    while (this._deferred.pending()) {
      await this._deferred.wait()
    }
    this._deferred.reset()
  }
  tryLock(throws = true) {
    if (this._deferred.pending())
      if (throws) {
        throw new Error("Mutex has been locked")
      } else return false
    this._deferred.reset()
    return true
  }
  unlock() {
    this._deferred.resolve()
  }
  async guard(cb) {
    await this.lock()
    try {
      return await cb()
    } finally {
      this.unlock()
    }
  }
  async guardOrSkip(cb) {
    const success = this.tryLock(false)
    if (!success) return
    try {
      return await cb()
    } finally {
      this.unlock()
    }
  }
}
