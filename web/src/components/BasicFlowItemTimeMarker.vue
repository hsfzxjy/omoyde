<script setup>
import { computed } from "vue"

const props = defineProps({ data: Object })
const visibility = computed(() => {
  let { dt, prevDt } = props.data
  if (!Number.isFinite(prevDt)) {
    const show = prevDt === Number.NEGATIVE_INFINITY
    return { year: show, date: show, time: show }
  }
  const delta = dt - prevDt
  dt = new Date(dt)
  prevDt = new Date(prevDt)
  return {
    year: dt.getFullYear() !== prevDt.getFullYear(),
    date:
      dt.getMonth() !== prevDt.getMonth() || dt.getDate() !== prevDt.getDate(),
    time: delta >= 1000 * 60 * 30, // 30 minutes
  }
})
const texts = computed(() => {
  const pad2 = (x) => x.toString().padStart(2, "0")
  const dt = new Date(props.data.dt)
  const Y = dt.getFullYear()
  const M = pad2(dt.getMonth() + 1)
  const D = pad2(dt.getDate())
  const h = pad2(dt.getHours())
  const m = pad2(dt.getMinutes())
  return { year: `~~ ${Y} ~~`, date: `${M} / ${D}`, time: `${h}:${m}` }
})
const visibleTexts = computed(() =>
  ["year", "date", "time"]
    .filter((kind) => visibility.value[kind])
    .map((kind) => [kind, texts.value[kind]])
)
</script>

<template>
  <div class="basic-flow-item-time-marker" v-if="visibleTexts.length">
    <span v-for="[kind, text] in visibleTexts" :class="kind">{{ text }}</span>
  </div>
</template>

<style lang="scss">
@font-face {
  font-family: aotc;
  src: url(@/fonts/aotc.ttf);
}

.basic-flow-item-time-marker {
  display: flex;
  flex-direction: column;
  align-items: center;
  color: #777;
  font-family: aotc;
  margin: 1rem 0;
  span {
    display: block;
  }
  .year {
    padding: 1em 0 1em 0;
    font-size: 2rem;
  }
  .date {
    font-size: 1.5rem;
    padding: 0.75em 0;
  }
  .time {
    font-size: 1rem;
    padding: 0.25em 0;
  }
}
</style>
