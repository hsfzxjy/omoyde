import { APIClient } from "../../infrastructures/api_client"
import { accessToken } from "../../infrastructures/auth"
import { Mutex } from "../../utils/aio"
import { E_AUTH, E_CLIENT_FILE_TOO_OLD } from "../errors"
import { mediaDB } from "./db"

function writeInt(buf, ptr, x, nbytes) {
  for (let i = 0; i < nbytes; i++) {
    buf[ptr + nbytes - 1 - i] = x & 0b11111111
    x >>= 8
  }
  return ptr + nbytes
}

export const mediaModifier = {
  _mutex: new Mutex(),
  _collect(overlay) {
    const [adds, dels] = overlay.collect()

    let textTotalBytes = 0
    const textEncoder = new TextEncoder()
    const texts = adds.map(({ text }) => {
      const buf = textEncoder.encode(text)
      textTotalBytes += buf.byteLength
      return buf
    })
    const bufSize = 8 * adds.length + textTotalBytes + 1 + 5 * dels.length
    const buf = new Uint8Array(bufSize)
    let ptr = 0
    for (let i = 0; i < adds.length; i++) {
      const { dt, type } = adds[i]
      const text = texts[i]
      let offset = dt % 1000
      let base = Math.round((dt - offset) / 1000)
      if (offset > 500) {
        offset -= 1000
        base += 1
      }
      offset = Math.floor(offset)
      buf[ptr] = type.charCodeAt(0)
      ptr += 1
      ptr = writeInt(buf, ptr, base, 4)
      buf[ptr] = offset
      ptr += 1
      ptr = writeInt(buf, ptr, text.byteLength, 2)
      buf.set(text, ptr)
      ptr += text.byteLength
    }
    buf[ptr] = 0
    ptr += 1
    for (const dt of dels) {
      let offset = dt % 1000
      let base = Math.round((dt - offset) / 1000)
      if (offset > 500) {
        offset -= 1000
        base += 1
      }
      offset = Math.floor(offset)
      ptr = writeInt(buf, ptr, base, 4)
      buf[ptr] = offset
      ptr += 1
    }
    return buf
  },
  async _request(encoded) {
    const hash = await mediaDB.getHashByKind("widget")
    const request = () =>
      APIClient.post("/storage/mod_media", encoded, {
        headers: { "expected-hash": hash },
      })
    let r = await request()
    if (r.data.code === E_AUTH) {
      accessToken.forceExpire()
      await accessToken.val()
      r = await request()
    }
    if (r.data.code === E_CLIENT_FILE_TOO_OLD) {
      // TODO
      throw new Error()
    }
  },
  async commit(overlay) {
    const encoded = this._collect(overlay)
    await this._request(encoded)
  },
}
