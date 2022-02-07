import { sleep } from "./aio"

function isNetworkFailure(err) {
  if (err.error === "error" && err.statusCode === undefined) return true

  let { code, message, response } = err
  return (
    response === undefined &&
    (code === "ECONNABORTED" || message === "Network Error")
  )
}

export function retryOnNetworkFailure(performRequest, retryInterval = 5000) {
  return performRequest().catch((err) => {
    if (!isNetworkFailure(err)) {
      return Promise.reject(err)
    }

    return sleep(retryInterval).then(() =>
      retryOnNetworkFailure(performRequest, retryInterval)
    )
  })
}
