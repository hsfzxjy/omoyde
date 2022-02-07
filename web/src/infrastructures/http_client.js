"use strict"

import Axios from "axios"
import { retryOnNetworkFailure } from "../utils/network"

export function createClient(options = {}) {
  const DEFAULT_ADAPTER = Axios.defaults.adapter
  const DEFAULT_OPTIONS = {
    validateStatus: (status) => {
      return (status >= 200 && status < 300) || (status >= 400 && status < 500)
    },
  }

  function _adapter(config) {
    config.timeout = config.timeout || 5000
    const { retryInterval } = config
    return retryOnNetworkFailure(() => DEFAULT_ADAPTER(config), retryInterval)
  }

  return Axios.create({ ...DEFAULT_OPTIONS, ...options, adapter: _adapter })
}
