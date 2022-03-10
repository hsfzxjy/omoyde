import Dexie from "dexie"
import EventEmitter from "events"
import { ref } from "vue"
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
  async _isLocalExpired(kind, remoteHash) {
    const record = await this._dexie.log.get(kind)
    const localHash = record && record.hash
    return localHash !== remoteHash
  }
  async _updateMediaContent(media, file) {
    const { hash: remoteHash, content } = file
    const kind = media.kind
    if (!(await this._isLocalExpired(kind, remoteHash))) return

    this._dexie.log.put({ kind, hash: remoteHash })

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
      const remoteHash = await media.getFileHash()
      return this._isLocalExpired(media.kind, remoteHash)
    })
    await this.pull(toPull)
  }
}

export const mediaDB = new Resource("mediaDB")
  .onInit(async (h) => {
    const dexie = new Dexie("media")
    dexie
      .version(DB_VERSION)
      .stores({ data: "++id, [dt+offset], kind, type", log: "kind, hash" })
    dexie.open()
    const internal = new MediaDBInternal(dexie)
    await internal.init()
    h.ready(internal)
  })
  .onExpired((h) => h.reset())
  .extend({
    async getHashByKind(kind) {
      const item = await this._dexie.log.where("kind").equals(kind).first()
      return item.hash
    },
    before: dispatch({ index: "beforeIndex" }, () => mediaDB),
    beforeIndex({ index, limit = 10, includes = false }) {
      let end = index
      if (!includes) end--
      let start = Math.max(end - limit + 1, 0)
      limit = Math.max(end - start + 1, 0)
      if (!limit) return []
      const items = this._dexie.data
        .orderBy("[dt+offset]")
        .offset(start)
        .limit(limit)
        .toArray()
      return items
    },
    after: dispatch({ index: "afterIndex" }, () => mediaDB),
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
    async countAll() {
      return ref(await this._dexie.data.count())
    },
    async at(index) {
      return await this._dexie.data.orderBy("[dt+offset]").offset(index).first()
    },
    async getHighlightedIndices() {
      const hit = this.cache.get(CE_HIGHLIGHTED_INDICES)
      if (hit !== null) return hit

      const highlightedIds = new Set(
        await this._dexie.data.where("type").equals("m").primaryKeys()
      )
      const allIds = await this._dexie.data.orderBy("[dt+offset]").primaryKeys()
      const ret = []
      for (let idx = 0; idx < allIds.length; idx++) {
        if (highlightedIds.has(allIds[idx])) ret.push(idx)
      }
      this.cache.set(CE_HIGHLIGHTED_INDICES, ret)
      return ret
    },
  })
