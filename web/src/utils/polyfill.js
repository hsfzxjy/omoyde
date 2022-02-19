function polyfillArray() {
  Array.prototype.aFilter = async function (cb) {
    const results = await this.aMap(cb)
    return this.filter((_, index) => results[index])
  }

  Array.prototype.aMap = function (cb) {
    return Promise.all(this.map(cb))
  }

  Array.prototype.extend = function (arr) {
    return this.splice.apply(this, [+Infinity, 0].concat(arr))
  }

  Array.prototype.replaceAll = function (arr) {
    return this.splice.apply(this, [0, +Infinity].concat(arr))
  }

  Array.prototype.zip = function (arr) {
    if (typeof arr[Symbol.iterator] !== "function")
      throw new Error(`expect arr to be iterable, got ${arr}`)
    if (arr.length !== this.length)
      throw new Error(
        `arr.length !== this.length (${arr.length} !== ${this.length})`
      )
    const result = []
    let index = 0
    for (const second of arr) {
      result.push([this[index], second])
    }
    return result
  }
}

function polyfillNumber() {
  Number.clamp = function (val, min, max) {
    if (val < min) {
      return min
    } else if (val > max) {
      return max
    } else {
      return val
    }
  }
}

export function polyfill() {
  polyfillArray()
  polyfillNumber()
}
