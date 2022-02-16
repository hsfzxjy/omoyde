import Dexie from "dexie"

const DB_VERSION = 1

function getDexieInstance(dbName, tblSchema) {
  let dexie = new Dexie(dbName)
  dexie.version(DB_VERSION).stores({ data: tblSchema, log: "time" })
  dexie.open()
  return dexie
}

export class Database {
  constructor({ name, schema }) {
    this._name = name
    this._dexie = getDexieInstance(name, schema)
  }
  onPull(cb) {
    this._onPull = cb
    return this
  }
  onGetRemoteLastModified(cb) {
    this._onGetRemoteLastModified = cb
    return this
  }
  async _remoteIsNewer(remoteLastModified) {
    return remoteLastModified > (await this.getLocalLastModified())
  }
  async getLocalLastModified() {
    const record = await this._dexie.log.orderBy("time").last()
    return record ? record.time : new Date(0)
  }
  transaction(cb) {
    return this._dexie.transaction("rw", this._dexie.log, this._dexie.data, cb)
  }
  async pull() {
    const { lastModified: remoteLastModified, content } = await this._onPull()
    this.transaction(async () => {
      // the fetched content was already in db
      if (!(await this._remoteIsNewer(remoteLastModified))) return

      this._dexie.log.clear().then(() => {
        return this._dexie.log.add({ time: remoteLastModified })
      })

      await this._dexie.data.clear()
      while (!content.done()) {
        const bulk = content.next()
        await this._dexie.data.bulkAdd(bulk)
      }
    })
  }
  async init() {
    const remoteLastModified = await this._onGetRemoteLastModified()
    if (!(await this._remoteIsNewer(remoteLastModified))) return
    await this.pull()
  }
}
