// a TrashBin holds references to creation-only anonymous objects
export class TrashBin {
  constructor() {
    this._holder = []
  }
  collect(v) {
    this._holder.push(v)
  }
}
