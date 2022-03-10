<script setup>
import { computed, inject, onBeforeUnmount, onMounted, ref } from "vue"
import BasicFlowItemTimeMarker from "./BasicFlowItemTimeMarker.vue"
import BasicFlowItemImage from "./BasicFlowItemImage.vue"
import BasicFlowItemWidget from "./BasicFlowItemWidget.vue"
import BasicFlowItemToolbar from "./BasicFlowItemToolbar.vue"
import { store } from "../states"

const itemObserver = inject("basic-flow-observer")
const props = defineProps({
  data: Object,
  localIndex: Number,
  globalIndex: Number,
})
const $wrapper = ref()
const $inner = ref()
const componentMapping = {
  image: BasicFlowItemImage,
  widget: BasicFlowItemWidget,
}
const currentComponent = computed(() => componentMapping[props.data.kind])
const classes = computed(() => {
  const classes = ["basic-flow-item"]
  if (store.fragment.editting) classes.push("editting")
  if ($inner.value) classes.extend($inner.value.wrapperClasses)
  return classes
})

onMounted(() => {
  itemObserver.observe($wrapper.value)
})
onBeforeUnmount(() => {
  itemObserver.unobserve($wrapper.value)
})
</script>

<template>
  <div ref="$wrapper" :class="classes" :data-local-index="props.localIndex">
    <basic-flow-item-time-marker :data="data" />
    <basic-flow-item-toolbar
      :data="data"
      :globalIndex="globalIndex"
      :localIndex="localIndex"
    />
    <component :is="currentComponent" :data="data" ref="$inner"></component>
  </div>
</template>

<style lang="scss">
.basic-flow-item {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  position: relative;

  &.editting {
    margin: 0 !important;
    margin-bottom: 1rem !important;
  }
}
</style>
