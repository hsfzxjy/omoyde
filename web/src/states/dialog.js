import { reactive, toRaw } from "vue"
import { clone, patch } from "../utils/misc"
import { store, bus } from "./index"

export const Dialog = {
  show(name, initialData) {
    const data = reactive(toRaw(clone(initialData)))
    patch(store.ui.dialog, { show: true, component: name, data })
    return new Promise((resolve) => {
      bus.once("dialog-dismissed", (data) => resolve(data))
    })
  },
  dismiss(data) {
    data = toRaw(data)
    patch(store.ui.dialog, { show: false, component: "", data: null })
    bus.emit("dialog-dismissed", data)
  },
}
