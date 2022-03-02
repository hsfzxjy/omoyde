<script setup>
import BasicFlow from "./BasicFlow.vue"
import TransitionFlow from "./TransitionFlow.vue"
import { store, bus } from "../states"
import { computed, nextTick } from "vue"
import { Mutex } from "../utils/aio"

const currentFlow = computed(() => {
  return {
    BasicFlow,
    TransitionFlow,
  }[store.ui.flow]
})

const flowMutex = new Mutex()
bus.on("edittingChanged", () =>
  flowMutex.guardOrSkip(async () => {
    const oldFlow = store.ui.flow
    store.ui.flow = "TransitionFlow"
    await nextTick()
    store.ui.flow = oldFlow
    await nextTick()
  })
)
</script>

<template>
  <div id="main">
    <suspense>
      <component :is="currentFlow" />
    </suspense>
  </div>
</template>

<style lang="scss">
#main {
  width: 100%;
  height: 100%;
}
</style>
