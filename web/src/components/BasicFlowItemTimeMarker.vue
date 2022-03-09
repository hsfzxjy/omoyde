<script setup>
import { computed } from "vue"
import { ITEM_NULL } from "../services/media/local"
import { timeGapIsLarge, dto2Date } from "../services/media/misc"

const props = defineProps({ data: Object })
const visibility = computed(() => {
  let { dto, prev } = props.data
  if (typeof prev[0].kind === "symbol") {
    const show = prev[0].kind === ITEM_NULL
    return { year: show, date: show, time: show }
  }
  const prevDto = prev[0].dto
  const showTime = timeGapIsLarge(dto, prevDto)
  const dt = dto2Date(dto)
  const prevDt = dto2Date(prevDto)
  return {
    year: showTime && dt.getFullYear() !== prevDt.getFullYear(),
    date:
      showTime &&
      (dt.getMonth() !== prevDt.getMonth() ||
        dt.getDate() !== prevDt.getDate()),
    time: showTime,
  }
})
const texts = computed(() => {
  const pad2 = (x) => x.toString().padStart(2, "0")
  const dt = dto2Date(props.data.dto)
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
