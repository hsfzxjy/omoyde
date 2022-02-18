<script setup>
import { computed, inject, onBeforeUnmount, onMounted, ref } from "vue"
import BasicFlowItemTimeMarker from "./BasicFlowItemTimeMarker.vue"
import BasicFlowItemImage from "./BasicFlowItemImage.vue"
import BasicFlowItemMsg from "./BasicFlowItemMsg.vue"

const itemObserver = inject("basic-flow-observer")
const props = defineProps({ data: Object, localIndex: Number })
const $wrapper = ref()
const $inner = ref()
const componentMapping = {
  image: BasicFlowItemImage,
  msg: BasicFlowItemMsg,
}
const currentComponent = computed(() => componentMapping[props.data.kind])
const classList = computed(() => {
  const classes = ["basic-flow-item"]
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
  <div ref="$wrapper" :class="classList" :data-local-index="props.localIndex">
    <basic-flow-item-time-marker :data="data" />
    <component :is="currentComponent" :data="data" ref="$inner"></component>
  </div>
</template>

<style lang="scss">
.basic-flow-item {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
}
</style>
