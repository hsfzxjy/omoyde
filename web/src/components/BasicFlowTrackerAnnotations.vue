<script setup>
import { computed, inject } from "vue"

const dataSource = inject("dataSource")
const highlightedIndices = await dataSource.getHighlightedIndices()
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

const highlightedAnnotations = highlightedIndices.map((index) =>
  makeAnnotation(index, ["hl"])
)
const annotations = computed(() => highlightedAnnotations)
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
