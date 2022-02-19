import EventEmitter from "events"

export function stringifyAsKey(keyLike) {
  if (Array.isArray(keyLike)) {
    return keyLike.map((v) => v.toString()).join("_")
  }
  return keyLike.toString()
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
    invoke() {
      timer && clearTimeout(timer)
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
