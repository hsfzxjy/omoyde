import { APIClient } from "./api_client"
import { Resource } from "../utils/resource"
import { accessToken } from "./auth"
import COS from "cos-js-sdk-v5"
import { retryOnNetworkFailure } from "../utils/network"
import { E_AUTH } from "../services/errors"

export const storageCredential = new Resource("storageCredential")
  .onInit(async (h) => {
    await accessToken.readyVal()
    const accessTokenSnapshot = accessToken.version()
    const r = await APIClient.get("/storage/credential")
    if (r.data.code === E_AUTH) {
      accessToken.expire(accessTokenSnapshot)
      h.expire("accessToken expired")
    }
    h.ready(r.data)
  })
  .onExpired((h) => h.reset())

const BUCKET_INFO = {
  Bucket: import.meta.env.VITE_TCLOUD_COS_BUCKET,
  Region: import.meta.env.VITE_TCLOUD_COS_REGION,
}

export const storageClient = new Resource("storageClient")
  .onInit(async (h) => {
    const cred = await storageCredential.readyVal()
    const cos = new COS({
      getAuthorization: (_, callback) => {
        callback({
          TmpSecretId: cred.credentials.tmpSecretId,
          TmpSecretKey: cred.credentials.tmpSecretKey,
          SecurityToken: cred.credentials.sessionToken,
          StartTime: cred.startTime,
          ExpiredTime: cred.expiredTime,
        })
      },
    })
    h.ready(cos)
  })
  .onExpired((h) => {
    storageCredential.forceExpire()
    h.reset()
  })
  .extend({
    service(name, options) {
      const performRequest = () =>
        new Promise((resolve, reject) => {
          this[name]({ ...options, ...BUCKET_INFO }, (err, data) => {
            if (err) reject(err)
            else resolve(data)
          })
        })
      return retryOnNetworkFailure(performRequest, 1000)
    },
    async getURL(filePath) {
      const data = await storageClient.service("getObjectUrl", {
        Key: filePath,
        Expires: 3600,
      })
      return data.Url
    },
    async getFileContent({ filePath, dataType }) {
      const { Body, headers } = await storageClient.service("getObject", {
        Key: filePath,
        DataType: dataType,
      })
      return {
        body: Body,
        headers,
        ETag,
      }
    },
    async getFileMetadata({ filePath }) {
      const { headers } = await storageClient.service("headObject", {
        Key: filePath,
      })
      return { headers, ETag }
    },
  })

function ETag() {
  return this.headers["etag"].slice(1, -1)
}
