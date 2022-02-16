import { reactive } from "vue"

export const store = reactive({
  ui: {
    sidebarExpanded: false,
  },
  fragment: { currentName: "default" },
})
