import EventEmitter from "events"

export function stringifyAsKey(keyLike) {
  if (typeof keyLike === "symbol") keyLike = keyLike.description
  if (Array.isArray(keyLike)) {
    return keyLike.map((v) => v.toString()).join("_")
  }
  return keyLike.toString()
}

export function keyBuilder(prefix, type) {
  if (!Array.isArray(prefix)) prefix = [prefix]
  return function (...rest) {
    const key = stringifyAsKey(prefix.concat(rest))
    switch (type) {
      case String:
        return key
      case Symbol:
        return Symbol(key)
      default:
        throw new Error(`unsupport key type: ${type}`)
    }
  }
}

export function randomRange(low, high) {
  const v = Math.random()
  return low + (high - low) * v
}

export function debounce(cb, timeout, setupCtx) {
  setupCtx = setupCtx || (() => ({}))
  let timer = null
  const events = new EventEmitter()
  const handler = {
    ctx: setupCtx(),
    canceled() {
      return timer === null
    },
    cancel() {
      timer && clearTimeout(timer)
    },
    invoke() {
      this.cancel()
      timer = setTimeout(async () => {
        try {
          await cb(handler.ctx)
        } finally {
          timer = null
          this.ctx = setupCtx()
          events.emit("ticked")
        }
      }, timeout)
    },
    nextTick() {
      if (!timer) return
      return new Promise((resolve) => {
        events.once("ticked", resolve)
      })
    },
  }
  return handler
}

export class Tumbler {
  constructor({ init, timeout }) {
    this._deb = debounce(() => {
      this._value = init()
    }, timeout)
    this._init = init
    this._value = init()
  }
  value() {
    return this._value
  }
  mutate(newValue) {
    this._value = newValue
    this._deb.invoke()
  }
}

export function dispatch(routes, thisArg) {
  const keys = Object.keys(routes)
  return function (options) {
    let that
    switch (typeof thisArg) {
      case "function":
        that = thisArg()
        break
      case "undefined":
        that = this
        break
      default:
        that = thisArg
    }
    for (const [key, method] of Object.entries(routes)) {
      if (options[key] !== undefined) return that[method](options)
    }
    throw new Error(`should specify one of ${keys}`)
  }
}

export const patch = Object.assign
export const noop = () => {}
