<script setup>
import { computed, inject } from "vue"

const dataSource = inject("dataSource")
const hlItems = await dataSource.getHlItems()
const nItems = await dataSource.countAll()

function makeAnnotation(index, classes) {
  return {
    index,
    style: {
      top: `${(index / (nItems.value - 1)) * 100}%`,
    },
    classes: ["basic-flow-tracker-annotation", ...classes],
  }
}

const annotations = computed(() =>
  hlItems.map(([index, type]) => makeAnnotation(index, type))
)
</script>

<template>
  <div
    v-for="anno in annotations"
    :key="anno.index"
    :style="anno.style"
    :class="anno.classes"
  ></div>
</template>

<style lang="scss">
.basic-flow-tracker-annotation {
  &.hl {
    border-color: #999;
    border-style: dashed;
    border-width: 1px 0 0 0;
  }
}
</style>
