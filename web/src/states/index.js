import EventEmitter from "events"
import { reactive, watch } from "vue"

export const store = reactive({
  ui: {
    sidebarExpanded: false,
    flow: "BasicFlow",
  },
  fragment: { currentName: "default", editting: true },
})

export const bus = new EventEmitter()
watch(
  () => store.fragment.editting,
  () => bus.emit("edittingChanged")
)
