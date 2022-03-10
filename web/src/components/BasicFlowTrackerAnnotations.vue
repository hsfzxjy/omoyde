<script setup>
import { computed, inject } from "vue"

const dataSource = inject("dataSource")
const hlItems = await dataSource.getHlItems()
const nItems = await dataSource.countAll()

function makeAnnotation(index, type) {
  if (type === "del") index -= 0.5
  return {
    key: `${index}-${type}`,
    index,
    style: {
      top: `${(index / (nItems.value - 1)) * 100}%`,
    },
    classes: ["basic-flow-tracker-annotation", type],
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
  border-width: 1px 0 0 0;
  &.m {
    border-color: #999;
    border-style: dashed;
  }
  &.add {
    border-color: green;
    border-style: solid;
  }
  &.del {
    border-color: red;
    border-style: solid;
  }
}
</style>
