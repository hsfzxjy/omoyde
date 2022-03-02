<script setup>
import BasicFlowTrackerAnnotations from "./BasicFlowTrackerAnnotations.vue"
import BasicFlowTrackerIndicator from "./BasicFlowTrackerIndicator.vue"
import { computed, onMounted, inject, reactive, ref } from "vue"
import { DebouncedTouchHandler } from "../utils/dom"
import { patch } from "../utils/misc"
import { TrashBin } from "../utils/trashbin"

const trashbin = new TrashBin()
const props = defineProps({ currentItemGlobalIndex: Number })
const $container = ref()
const $main = ref()
const state = reactive({
  touching: false,
  touchingThumbFraction: null,
  touchingItem: null,
})
const thumbStyle = computed(() => {
  const frac =
    state.touching && state.touchingThumbFraction !== null
      ? state.touchingThumbFraction
      : props.currentItemGlobalIndex / (nItems - 1)
  return { top: `${frac * 100}%` }
})
const touchingGlobalIndex = computed(() => {
  const L = nItems
  const frac = state.touchingThumbFraction
  const margin = 1 / (L - 1)
  let index = Math.floor(frac * (L - 1))
  if (frac - index * margin > margin / 2) index++
  index = Math.min(index, L - 1)
  return index
})

onMounted(() => {
  const touchHandler = new DebouncedTouchHandler({
    $el: $main.value,
    options: { timeout: 100 },
    onStart(evt) {
      evt.preventDefault()
      state.touching = true
    },
    onMove(evt) {
      evt.preventDefault()
      const { pageY: currentY } = evt.touches[0]
      const bRect = $main.value.getBoundingClientRect()
      const val = (currentY - bRect.top) / bRect.height
      state.touchingThumbFraction = Number.clamp(val, 0, 1)
    },
    onEnd(evt) {
      evt.preventDefault()
      if (state.touchingThumbFraction !== null)
        flowBus.emit("update-index", touchingGlobalIndex.value)
      patch(state, {
        touchingThumbFraction: null,
        touching: false,
        touchingItem: null,
      })
    },
    async onHandle() {
      const index = touchingGlobalIndex.value
      state.touchingItem = await dataSource.at(index)
    },
  })
  trashbin.collect(touchHandler)
})

const dataSource = inject("dataSource")
const flowBus = inject("flowBus")
const nItems = await dataSource.countAll()
const startText = "START"
const endText = "NOW"
</script>

<template>
  <div class="basic-flow-tracker" ref="$container">
    <div class="basic-flow-tracker-start">{{ startText }}</div>
    <div class="basic-flow-tracker-main" ref="$main">
      <div class="basic-flow-tracker-axis"></div>
      <basic-flow-tracker-annotations />
      <div class="basic-flow-tracker-thumb" :style="thumbStyle">
        <basic-flow-tracker-indicator
          v-if="state.touching"
          :item="state.touchingItem"
        />
      </div>
    </div>
    <div class="basic-flow-tracker-end">{{ endText }}</div>
  </div>
</template>

<style lang="scss">
.basic-flow-tracker {
  position: absolute;
  top: 5%;
  bottom: 5%;
  right: 0;
  width: 40px;
  max-width: 10%;
  display: flex;
  flex-direction: column;
}
.basic-flow-tracker-start,
.basic-flow-tracker-end {
  font-size: 0.4rem;
  font-family: aotc;
  text-align: center;
}
.basic-flow-tracker-start {
  margin-bottom: 3px;
}
.basic-flow-tracker-end {
  margin-top: 5px;
}
.basic-flow-tracker-main {
  flex-grow: 1;
  position: relative;
}
.basic-flow-tracker-axis {
  border-width: 0 1px;
  border-style: solid;
  border-color: #999;
  position: absolute;
  left: 50%;
  top: 0;
  bottom: 0;
}
.basic-flow-tracker-thumb,
.basic-flow-tracker-annotation {
  height: 1px;
  position: absolute;
  left: 15%;
  right: 15%;
}
.basic-flow-tracker-thumb {
  background-color: black;
}
</style>
