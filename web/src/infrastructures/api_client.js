import { createClient } from "./http_client"

export const APIClient = createClient({
  baseURL: import.meta.env.VITE_WEB_AUTHURL,
  withCredentials: true,
})
