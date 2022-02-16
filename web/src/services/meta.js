import { Database } from "../infrastructures/db"
import { storageClient } from "../infrastructures/storage"
import { Resource } from "../utils/resource"

function decodeMetaList(content, chunkSize = 1000) {
  let ptr = 0
  let done = false
  const arr = new Uint8Array(content)
  return {
    done() {
      return done
    },
    next() {
      let counter = 0
      let chunk = []
      while (counter < chunkSize && ptr < arr.length) {
        const pid =
          (arr[ptr] << 24) |
          (arr[ptr + 1] << 16) |
          (arr[ptr + 2] << 8) |
          arr[ptr + 3]
        const dt =
          ((arr[ptr + 4] << 24) |
            (arr[ptr + 5] << 16) |
            (arr[ptr + 6] << 8) |
            arr[ptr + 7]) *
          1000
        const h = arr[ptr + 8]
        const w = arr[ptr + 9]
        const item = { pid, h, w, dt }
        chunk.push(item)
        ptr += 10
        counter += 1
      }
      if (ptr === arr.length) done = true
      return chunk
    },
  }
}

async function getMetaListFile() {
  const r = await storageClient.getFileContent({
    filePath: "/assets/metas.bin",
    dataType: "arraybuffer",
  })
  return {
    content: decodeMetaList(r.body),
    lastModified: r.lastModified(),
  }
}

async function getMetaListFileLastModified() {
  const r = await storageClient.getFileMetadata({
    filePath: "/assets/metas.bin",
  })
  return r.lastModified()
}

export const metaDB = new Resource("metaDB")
  .onInit(async (h) => {
    const db = new Database({
      name: "metas",
      schema: "pid,dt",
    })
      .onPull(getMetaListFile)
      .onGetRemoteLastModified(getMetaListFileLastModified)

    await db.init()
    h.ready(db)
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
