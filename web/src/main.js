import { createApp } from "vue"
import App from "./App.vue"
import { loginWithPassword, unlockWithPincode } from "./infrastructures/auth"
import { store } from "./states"
import { extendBuiltins } from "./utils/builtin_ext"

extendBuiltins()

createApp(App).mount("#app")

if (import.meta.env.MODE === "development") {
  window.addEventListener("keydown", (evt) => {
    if (!evt.altKey) return
    switch (evt.key) {
      case "U":
        unlockWithPincode(import.meta.env.VITE_SECURITY_PINCODE)
        break
      case "L":
        loginWithPassword(import.meta.env.VITE_SECURITY_PASSWORD)
        break
      case "E":
        store.fragment.editting = !store.fragment.editting
        break
    }
  })
}
