import { storageClient } from "../../infrastructures/storage"

// TODO: better name
const shared = {
  async getFile() {
    const r = await storageClient.getFileContent({
      filePath: this.filePath,
      dataType: "arraybuffer",
    })
    return {
      content: this._decode(r.body),
      lastModified: r.lastModified(),
    }
  },
  async getFileLastModified() {
    const r = await storageClient.getFileMetadata({
      filePath: this.filePath,
    })
    return r.lastModified()
  },
}

export const IMAGE_MEDIA = {
  ...shared,
  kind: "image",
  filePath: "/assets/metas.bin",
  _decode(content, chunkSize = 1000) {
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
          const pid = (arr[ptr + 0] << 16) | (arr[ptr + 1] << 8) | arr[ptr + 2]
          ptr += 3
          const dt =
            ((arr[ptr + 0] << 24) |
              (arr[ptr + 1] << 16) |
              (arr[ptr + 2] << 8) |
              arr[ptr + 3]) *
            1000
          ptr += 4
          const h = arr[ptr + 0]
          const w = arr[ptr + 1]
          ptr += 2
          const item = { pid, h, w, dt, kind: "image" }
          chunk.push(item)
          counter += 1
        }
        if (ptr === arr.length) done = true
        return chunk
      },
    }
  },
}

export const MSG_MEDIA = {
  ...shared,
  kind: "msg",
  filePath: "/assets/msg.bin",
  _decode(content, chunkSize = 1000) {
    let ptr = 0
    let done = false
    const arr = new Uint8Array(content)
    const textDecoder = new TextDecoder("utf-16be")
    return {
      done() {
        return done
      },
      next() {
        let counter = 0
        const chunk = []
        while (counter < chunkSize && ptr < arr.byteLength) {
          const type = String.fromCharCode(arr[ptr])
          ptr += 1
          const dtBase =
            ((arr[ptr + 0] << 24) |
              (arr[ptr + 1] << 16) |
              (arr[ptr + 2] << 8) |
              arr[ptr + 3]) *
            1000
          ptr += 4
          const dt = dtBase + (arr[ptr] - 256)
          ptr += 1
          const textLen = (arr[ptr] << 8) | arr[ptr + 1]
          ptr += 2
          const text = textDecoder.decode(arr.subarray(ptr, ptr + textLen))
          ptr += textLen
          chunk.push({ type, dt, text, kind: "msg" })
          counter += 1
        }
        if (ptr === arr.byteLength) done = true
        return chunk
      },
    }
  },
}

export const allMedias = [IMAGE_MEDIA, MSG_MEDIA]
