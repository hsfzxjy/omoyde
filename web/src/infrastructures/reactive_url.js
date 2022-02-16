import { LSResource } from "../utils/resource"
import { storageClient } from "./storage"

export function ReactiveURL(filePath) {
  return new LSResource(filePath)
    .onInit(async (h) => {
      const version = storageClient.version()
      let url
      try {
        url = await storageClient.getURL(filePath)
      } catch (e) {
        storageClient.expire(version)
        throw e
      }
      h.ready(url)
    })
    .onExpired((h) => h.reset())
}
