<script setup>
import { computed } from "vue"

const props = defineProps({ item: Object })
const text = computed(getText)

function getText() {
  const item = props.item
  if (item === null) return "Loading..."
  const lines = [new Date(item.dt).toLocaleString()]
  const { text, type } = item
  if (type === "m") {
    lines.push(`<b>${text}</b>`)
  } else if (type === "q") {
    const overflow = text.length > 13
    const excerpt =
      text.slice(0, overflow ? 12 : 13) + (overflow ? "\u2026" : "")
    lines.push(`\u201c${excerpt}\u201d`)
  }
  return lines.join("<br/>")
}
</script>

<template>
  <div class="basic-flow-tracker-indicator" v-html="text"></div>
</template>

<style lang="scss">
.basic-flow-tracker-indicator {
  font-size: 0.8rem;
  position: absolute;
  right: 120%;
  top: -27px;
  background: white;
  border: 1px solid black;
  padding: 0.5rem;
  max-width: calc(100vw - 40px);
  width: 70vw;
  text-align: center;
  border-radius: 0.5rem;
}
</style>
