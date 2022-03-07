import { E_AUTH } from "../services/errors"
import { Resource } from "../utils/resource"
import { APIClient } from "./api_client"

export const refreshToken = new Resource("refreshToken").onExpired(
  async (h) => {
    await h.expect("LOGIN")
    h.ready()
  }
)

export const accessToken = new Resource("accessToken").onExpired(async (h) => {
  const refreshTokenExpecting = refreshToken.isExpecting()
  await refreshToken.val()
  if (refreshTokenExpecting) h.ready()
  const pincode = await h.expect("PINCODE")
  const r = await APIClient.post("/refresh", { password: pincode })
  if (r.data.code === E_AUTH) {
    refreshToken.forceExpire()
    await refreshToken.val()
  }
  h.ready()
})

export async function loginWithPassword(password) {
  const r = await APIClient.post("/login", { password })
  if (r.data.code === E_AUTH) throw new Error(r.data.detail)

  refreshToken.send("LOGIN")
}

export function unlockWithPincode(pincode) {
  accessToken.send("PINCODE", pincode)
}
