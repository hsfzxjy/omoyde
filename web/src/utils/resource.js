import EventEmitter from "events"
import { LS } from "./dom"
import { stringifyAsKey } from "./misc"

const STATE_CREATED = Symbol("STATE_CREATED")
const STATE_UNINIT = Symbol("STATE_UNINIT")
const STATE_INVALID = Symbol("STATE_INVALID")
const STATE_PANICKED = Symbol("STATE_PANICKED")
const STATE_CALLBACK_RUNNING = Symbol("STATE_CALLBACK_RUNNING")
const STATE_EXPECTING = Symbol("STATE_EXPECTING")
const STATE_REQUEST_INCOMING = Symbol("STATE_REQUEST_INCOMING")
const STATE_EXPIRED = Symbol("STATE_EXPIRED")
const STATE_READY = Symbol("STATE_READY")

function _assertErgodic(s) {
  if (
    s !== STATE_CALLBACK_RUNNING &&
    s !== STATE_READY &&
    s !== STATE_EXPECTING
  )
    throw new Error(`expected ergodic state, got ${s.toString()}`)
}

class StateRequest {
  constructor(state, value) {
    this.targetState = state
    this.value = value
  }
}

function makeHandle(res) {
  return {
    panic(value) {
      throw new StateRequest(STATE_PANICKED, value)
    },
    expire(value) {
      throw new StateRequest(STATE_EXPIRED, value)
    },
    ready(value) {
      throw new StateRequest(STATE_READY, value)
    },
    reset() {
      throw new StateRequest(STATE_UNINIT)
    },
    expect(phrase) {
      console.debug(res._name, "expecting", phrase)
      res._to(STATE_EXPECTING)
      return new Promise((resolve, reject) => {
        res._events.once("SEND", (phrase_, val) => {
          res._to(STATE_CALLBACK_RUNNING)
          if (phrase === phrase_) resolve(val)
          else reject(`unexpected phrase ${phrase_}`)
        })
      })
    },
  }
}

function makeRequestBuffer(res) {
  return {
    internal: null,
    external: [],
    get() {
      let r = this.external.length > 0 ? this.external.at(-1) : this.internal
      this.internal = null
      this.external.length = 0
      return r
    },
    setInternal(e) {
      if (!(e instanceof StateRequest)) {
        e = new StateRequest(STATE_PANICKED, e)
      }
      this.internal = e
    },
  }
}

export class Resource {
  onInit(cb) {
    this._callbacks.set(STATE_UNINIT, cb)
    return this
  }
  onPanicked(cb) {
    this._callbacks.set(STATE_PANICKED, cb)
    return this
  }
  onExpired(cb) {
    this._callbacks.set(STATE_EXPIRED, cb)
    return this
  }
  onExpiredOrPanicked(cb) {
    return this.onPanicked(cb).onExpired(cb)
  }
  autoReset() {
    return this.onExpiredOrPanicked((h) => h.reset())
  }
  drive() {
    if (this._state !== STATE_CREATED)
      throw new Error(`Resource ${this.name} has already driven`)
    this.val()
    return this
  }
  afterReady(cb) {
    this._events.on(STATE_READY, cb)
    return this
  }
  extend(methods) {
    for (const [name, fn] of Object.entries(methods)) {
      this[name] = async function (...rest) {
        const val = await this.val()
        if (fn === Proxy) {
          return val[name].apply(val, rest)
        } else {
          return fn.apply(val, rest)
        }
      }
    }
    return this
  }
  isExpecting() {
    return this._state === STATE_EXPECTING
  }
  send(phrase, val) {
    this._events.emit("SEND", phrase, val)
  }
  expire(version) {
    this._externalStateRequest(STATE_EXPIRED, version)
  }
  forceExpire() {
    this.expire(this.version())
  }
  val() {
    switch (this._state) {
      case STATE_CREATED:
        this._poll()
        return this.val()

      case STATE_READY:
        return Promise.resolve(this._value)

      case STATE_EXPECTING:
      case STATE_CALLBACK_RUNNING:
        return new Promise((resolve, reject) => {
          this._events.once(STATE_REQUEST_INCOMING, (targetState, value) => {
            if (targetState === STATE_READY) {
              resolve(value)
            } else {
              reject(value)
            }
          })
        })
      default:
        throw new Error(`unexpected state ${this._state.toString()}`)
    }
  }
  readyVal() {
    const val = this.val()
    if (!(val instanceof Promise)) return val
    return val.catch(() => this.readyVal())
  }
  version() {
    return this._version
  }

  constructor(name) {
    this._name = name
    this._handle = makeHandle(this)
    this._callbacks = new Map()

    this._value = null
    this._version = 0
    this._state = STATE_CREATED
    this._events = new EventEmitter()
    this._requestBuffer = makeRequestBuffer(this)

    this.onInit((h) => h.ready())
  }
  _to(state) {
    console.debug(this._name, this._state, "=>", state, this._value)
    this._state = state
  }
  _schedule(cb) {
    this._to(STATE_CALLBACK_RUNNING)
    ;(async () => {
      try {
        await cb(this._handle, this._value)
        this._to(STATE_INVALID)
      } catch (e) {
        this._requestBuffer.setInternal(e)
        this._to(STATE_REQUEST_INCOMING)
      }
      this._poll()
    })()
  }
  _poll() {
    switch (this._state) {
      // start states
      case STATE_CREATED:
        this._to(STATE_UNINIT)
        this._poll()
        return

      // dead states
      case STATE_INVALID:
        throw new Error("trapped at STATE_INVALID")

      // ergodic states
      case STATE_EXPECTING:
      case STATE_CALLBACK_RUNNING:
        return

      case STATE_READY:
        this._events.emit(STATE_READY, this._value)
        this._events.once("AWAKE", () => {
          this._to(STATE_REQUEST_INCOMING)
          this._poll()
        })
        return

      // transient states
      case STATE_REQUEST_INCOMING:
        const { targetState, value } = this._requestBuffer.get()
        this._events.emit(STATE_REQUEST_INCOMING, targetState, value)
        this._value = value
        this._to(targetState)
        this._version++
        this._poll()
        return

      case STATE_UNINIT:
      case STATE_EXPIRED:
      case STATE_PANICKED:
        const cb = this._callbacks.get(this._state)
        this._events.emit(this._state)
        if (!cb && this._state === STATE_PANICKED) throw this._value
        this._schedule(cb)
        return
    }
  }
  _externalStateRequest(targetState, version) {
    _assertErgodic(this._state)
    if (version !== this._version) return
    const r = new StateRequest(targetState)
    this._requestBuffer.external.push(r)
    if (this._state === STATE_READY) this._events.emit("AWAKE")
  }
}

export class LSResource extends Resource {
  constructor(key) {
    const _key = stringifyAsKey(key)
    super(`LSResource[${_key}]`)
    this._key = _key
    this._events
      .on(STATE_READY, this._saveValue.bind(this))
      .on(STATE_PANICKED, this._eraseValue.bind(this))
      .on(STATE_EXPIRED, this._eraseValue.bind(this))
  }

  drive() {
    this._loadValue()
    if (this._state === STATE_CREATED) super.drive()
    return this
  }

  _loadValue() {
    const value = LS.getItem(this._key)
    if (value !== null) {
      this._value = value
      this._to(STATE_READY)
      this._poll()
    }
  }

  _saveValue() {
    LS.setItem(this._key, this._value)
  }

  _eraseValue() {
    LS.removeItem(this._key)
  }
}
