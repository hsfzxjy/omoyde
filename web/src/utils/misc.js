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

export const patch = Object.assign
export const noop = () => {}
