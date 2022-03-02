import { reactive } from "vue"

export const store = reactive({
  ui: {
    sidebarExpanded: false,
    flow: "BasicFlow",
  },
  fragment: { currentName: "default" },
})
