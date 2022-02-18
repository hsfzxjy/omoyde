import Dexie from "dexie"
import { Resource } from "../../utils/resource"
import { allMedias } from "./grocery"

const DB_VERSION = 1

// TODO: better name
class MediaDBInternal {
  constructor(dexie) {
    this._dexie = dexie
  }
  _transaction(cb) {
    return this._dexie.transaction("rw", this._dexie.log, this._dexie.data, cb)
  }
  async _isLocalExpired(kind, remoteTS) {
    const record = await this._dexie.log.get(kind)
    const localTS = record ? record.time : new Date(0)
    return localTS < remoteTS
  }
  async _updateMediaContent(media, file) {
    const { lastModified: remoteTS, content } = file
    const kind = media.kind
    if (!(await this._isLocalExpired(kind, remoteTS))) return

    this._dexie.log.put({ kind, time: remoteTS })

    await this._dexie.data.where("kind").equals(kind).delete()
    while (!content.done()) {
      const bulk = content.next()
      await this._dexie.data.bulkAdd(bulk)
    }
  }
  async pull(medias) {
    if (medias === undefined) medias = allMedias
    const files = await medias.aMap(async (m) => m.getFile())
    return this._transaction(() =>
      medias
        .zip(files)
        .aMap(([media, file]) => this._updateMediaContent(media, file))
    )
  }

  async init() {
    const toPull = await allMedias.aFilter(async (media) => {
      const remoteTS = await media.getFileLastModified()
      return this._isLocalExpired(media.kind, remoteTS)
    })
    await this.pull(toPull)
  }
}

export const mediaDB = new Resource("mediaDB")
  .onInit(async (h) => {
    const dexie = new Dexie("media")
    dexie
      .version(DB_VERSION)
      .stores({ data: "++id, dt, kind", log: "kind, time" })
    dexie.open()
    const internal = new MediaDBInternal(dexie)
    await internal.init()
    h.ready(internal)
  })
  .extend({
    beforeDt({ dt, limit = 10, includes = false }) {
      const opName = includes ? "belowOrEqual" : "below"
      return this._dexie.data
        .where("dt")
        [opName](dt)
        .limit(limit)
        .reverse()
        .sortBy("dt")
        .then((data) => data.reverse())
    },
    afterDt({ dt, limit = 10, includes = false }) {
      const opName = includes ? "aboveOrEqual" : "above"
      return this._dexie.data.where("dt")[opName](dt).limit(limit).sortBy("dt")
    },
  })
