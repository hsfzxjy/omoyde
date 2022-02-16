import { isReactive, isRef, toRaw, watch } from "vue"
import { debounce, patch, stringifyAsKey } from "./misc"

class CachedValue {
  constructor(source, backend) {
    this._source = source
    this._backend = backend
    this._backend.loadValue((v) => this._source.set(v))
    this._onChangedHandler = debounce(
      () => this._backend.storeValue(this._source.getRaw()),
      150
    )
    this._source.onChanged(() => this._onChangedHandler.invoke())
  }
  get() {
    return this._source.get()
  }
  getRaw() {
    return this._source.getRaw()
  }
  set(v) {
    return this._source.set(v)
  }
}

class BaseSource {
  constructor() {
    this._onChanged = null
  }
  onChanged(cb) {
    this._onChanged = cb
  }
}

class RefSource extends BaseSource {
  constructor(value) {
    super()
    if (!isRef(value) && !isReactive(value))
      throw new Error("value should be a ref or reactive")
    this._inner = value
    watch(
      this._inner,
      (value) => {
        this._onChanged && this._onChanged(value)
      },
      { deep: true }
    )
  }
  get() {
    return this._inner
  }
  set(v) {
    if (isRef(this._inner)) {
      this._inner.value = v
    } else {
      patch(this._inner, v)
    }
  }
  getRaw() {
    return toRaw(this._inner)
  }
}

class LSBackend {
  constructor(keyLike) {
    this._key = stringifyAsKey(keyLike)
  }
  loadValue(cb) {
    const loaded = window.localStorage.getItem(this._key)
    if (loaded === null) return
    cb(JSON.parse(loaded))
  }
  storeValue(v) {
    window.localStorage.setItem(this._key, JSON.stringify(v))
  }
}

export function LSRefValue(key, value) {
  return new CachedValue(new RefSource(value), new LSBackend(key))
}
