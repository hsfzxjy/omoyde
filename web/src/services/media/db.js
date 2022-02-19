import Dexie from "dexie"
import EventEmitter from "events"
import { LSCache } from "../../utils/cache"
import { dispatch, keyBuilder } from "../../utils/misc"
import { Resource } from "../../utils/resource"
import { allMedias } from "./grocery"

const DB_VERSION = 1

const build_cache_key = keyBuilder(["mediaDB", "cache"], Symbol)
const CE_HIGHLIGHTED_INDICES = build_cache_key("highlighted_indices")

// TODO: better name
class MediaDBInternal {
  constructor(dexie) {
    this.cache = new LSCache()
    this._dexie = dexie
    this._events = new EventEmitter()
    this._events.on("invalidated", () => this.cache.expireAll())
  }
  _transaction(cb, mode = "rw") {
    return this._dexie.transaction(mode, this._dexie.log, this._dexie.data, cb)
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
    await this._transaction(() =>
      medias
        .zip(files)
        .aMap(([media, file]) => this._updateMediaContent(media, file))
    )
    this.invalidate()
  }
  invalidate() {
    this._events.emit("invalidated")
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
      .stores({ data: "++id, dt, kind, type", log: "kind, time" })
    dexie.open()
    const internal = new MediaDBInternal(dexie)
    await internal.init()
    h.ready(internal)
  })
  .extend({
    invalidate: Proxy,
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
    after: dispatch(
      {
        index: "afterIndex",
        dt: "afterDt",
      },
      () => mediaDB
    ),
    async afterIndex({
      index,
      limit = 10,
      includes = false,
      withFirstIndex = false,
    }) {
      if (!includes) index++
      const items = await this._dexie.data
        .orderBy("dt")
        .offset(index)
        .limit(limit)
        .toArray()
      return withFirstIndex ? [index, items] : items
    },
    async afterDt({
      dt,
      limit = 10,
      includes = false,
      withFirstIndex = false,
    }) {
      const opName = includes ? "aboveOrEqual" : "above"
      const items = await this._dexie.data
        .where("dt")
        [opName](dt)
        .limit(limit)
        .sortBy("dt")
      if (!withFirstIndex) return items
      const revOpName = includes ? "below" : "belowOrEqual"
      const count = await this._dexie.data.where("dt")[revOpName](dt).count()
      return [count, items]
    },
    async countAll() {
      return await this._dexie.data.count()
    },
    async at(index) {
      return await this._dexie.data.orderBy("dt").offset(index).first()
    },
    async getHighlightedIndices() {
      const hit = this.cache.get(CE_HIGHLIGHTED_INDICES)
      if (hit !== null) return hit

      const highlightedIds = new Set(
        await this._dexie.data.where("type").equals("m").primaryKeys()
      )
      const allIds = await this._dexie.data.orderBy("dt").primaryKeys()
      const ret = []
      for (let idx = 0; idx < allIds.length; idx++) {
        if (highlightedIds.has(allIds[idx])) ret.push(idx)
      }
      this.cache.set(CE_HIGHLIGHTED_INDICES, ret)
      return ret
    },
  })
