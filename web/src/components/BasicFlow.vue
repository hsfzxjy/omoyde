<script setup>
/* this component should be recycled after dataSource changed */

import { onBeforeUnmount, onMounted, provide, reactive, ref, watch } from "vue"
import { getDataSource } from "../services/fragment"
import { store } from "../states"
import { Mutex } from "../utils/aio"
import { DebouncedIntersectionObserver, ScrollHelper } from "../utils/dom"
import { patch } from "../utils/misc"
import { TrashBin } from "../utils/trashbin"
import { LSRefValue } from "../utils/value"
import BasicFlowItem from "./BasicFlowItem.vue"

const trashbin = new TrashBin()
const dataSource = getDataSource()
const currentItems = reactive([])
const tracker = reactive({
  date: 0,
  pid: 0,
  offset: 0,
  localIndex: 0,
  globalIndex: 0,
  atStart: false,
  atEnd: false,
})
watch(
  () => tracker.localIndex,
  (newIdx) => {
    if (!currentItems[newIdx]) return
    tracker.date = currentItems[newIdx].dt
  }
)

trashbin.collect(
  LSRefValue(["basic-flow", store.fragment.currentName, "tracker"], tracker)
)

const LIMIT = 6 // capacity limit for either direction

const $flowList = ref()
const $flowContainer = ref()
const $topSentinel = ref()
const $bottomSentinel = ref()

const sentinelObserver = new DebouncedIntersectionObserver({
  onEvent(entries, ctx) {
    for (const entry of entries) {
      const target = entry.target
      if (!entry.intersectionRatio > 0) continue
      if (target === $bottomSentinel.value) ctx.bottom = true
      if (target === $topSentinel.value) ctx.top = true
    }
  },
  async handle(ctx) {
    if (ctx.bottom) await itemsPuller.backward()
    if (ctx.top) await itemsPuller.forward()
  },
  setupCtx: () => ({ top: false, bottom: false }),
  options: { timeout: 150, threshold: 0 },
})
const itemObserver = new DebouncedIntersectionObserver({
  onEvent(entries, ctx) {
    entries.forEach((entry) => {
      const idx = parseInt(entry.target.getAttribute("data-local-index"))
      entry.localIndex = idx
      ctx.entries.set(idx, entry)
      ctx.min = Math.min(ctx.min, idx)
      ctx.max = Math.max(ctx.max, idx)
    })
  },
  handle(ctx) {
    const set = (localIndex, offset) => {
      const oldLocalIndex = tracker.localIndex
      const oldGlobalIndex = tracker.globalIndex
      patch(tracker, {
        localIndex,
        offset,
        globalIndex: localIndex - oldLocalIndex + oldGlobalIndex,
      })
    }
    for (let i = ctx.min; i <= ctx.max; i++) {
      const entry = ctx.entries.get(i)
      if (!entry) continue
      const {
        intersectionRatio: ratio,
        boundingClientRect: { y },
      } = entry

      if (ratio === 0 && y < 0) {
        if (i === ctx.max) {
          set(i + 1, 0)
          break
        }
      } else if ((ratio > 0 && y < 0) || (ratio === 1 && i === 0)) {
        set(i, 1 - ratio)
        break
      } else {
        break
      }
    }
  },
  setupCtx: () => ({ entries: new Map(), min: +Infinity, max: -Infinity }),
  options: { timeout: 150, threshold: [0, 0.5, 1.0] },
})
provide("basic-flow-observer", itemObserver)

const scrollHelper = new ScrollHelper(
  () => $flowContainer.value,
  () => $flowList.value.children[tracker.localIndex]
)

onMounted(() => {
  sentinelObserver.observe($topSentinel.value)
  sentinelObserver.observe($bottomSentinel.value)
})
onBeforeUnmount(() => {
  sentinelObserver.disconnect()
  itemObserver.disconnect()
})

const itemsPuller = {
  _mutex: new Mutex(),
  _preprocItems(items) {
    items.forEach((item) => {
      item.prevDt = null
    })
  },
  _postprocItems() {
    const L = currentItems.length
    if (!L) return
    let prevDt = tracker.atStart ? -Infinity : undefined
    for (let i = 0; i < L; i++) {
      if (prevDt === currentItems[i].prevDt) break
      currentItems[i].prevDt = prevDt
      prevDt = currentItems[i].dt
    }
    for (let i = L - 1; i >= 1; --i) {
      prevDt = currentItems[i - 1].dt
      if (prevDt === currentItems[i].prevDt) break
      currentItems[i].prevDt = prevDt
    }
  },
  initial(cond) {
    return this._mutex.guardOrSkip(async () => {
      const [globalIndex, items] = await dataSource.after({
        ...cond,
        limit: LIMIT + 1,
        includes: true,
        withFirstIndex: true,
      })
      this._preprocItems(items)
      currentItems.replaceAll(items)
      this._postprocItems()
      if (items.length) {
        const first = items[0]
        patch(tracker, {
          date: +first.dt,
          pid: first.pid,
          offset: 0,
          localIndex: 0,
        })
      }
      patch(tracker, {
        globalIndex,
        atStart: false,
        atEnd: items.length < LIMIT + 1,
      })

      await scrollHelper.toTop()
    })
  },
  forward() {
    return this._mutex.guardOrSkip(async () => {
      // no more items to pull
      if (tracker.atStart) return

      // wait until localIndex become fresh
      await itemObserver.nextTick()

      const breach = LIMIT - tracker.localIndex
      // items are adequate, no need to pull
      if (breach <= 0) return

      const items = await dataSource.beforeDt({
        dt: currentItems[0].dt,
        limit: breach,
      })
      this._preprocItems(items)

      const recoverScrollTop = scrollHelper.dictate()

      // if no new items, we reach the very beginning
      if (!items.length) tracker.atStart = true
      // new items would be prepended, so we increase localIndex
      tracker.localIndex += items.length
      // prepend new items
      currentItems.unshift.apply(currentItems, items)
      // drop redundant items at the tail
      const numToSplice = currentItems.length - 1 - tracker.localIndex - LIMIT
      if (numToSplice > 0) {
        currentItems.splice(-numToSplice, numToSplice)
        tracker.atEnd = false
      }
      this._postprocItems()

      await recoverScrollTop()
    })
  },
  backward() {
    return this._mutex.guardOrSkip(async () => {
      // no more items to pull
      if (tracker.atEnd) return

      // wait until localIndex become fresh
      await itemObserver.nextTick()

      const breach = LIMIT - (currentItems.length - 1 - tracker.localIndex)
      // items are adequate, no need to pull
      if (breach <= 0) return

      const items = await dataSource.afterDt({
        dt: currentItems.at(-1).dt,
        limit: breach,
      })
      this._preprocItems(items)

      const recoverScrollTop = scrollHelper.dictate()

      // if no new items, we reach the very end
      if (!items.length) tracker.atEnd = true
      // since new items would be appended, we don't have to mutate localIndex
      // append new items
      currentItems.extend(items)
      // drop excess items at the front
      const numToSplice = tracker.localIndex - LIMIT
      if (numToSplice > 0) {
        currentItems.splice(0, numToSplice)
        tracker.localIndex -= numToSplice
        tracker.atStart = false
      }
      this._postprocItems()

      await recoverScrollTop()
    })
  },
}

await itemsPuller.initial({ dt: tracker.date })
</script>

<template>
  <div ref="$flowContainer" class="basic-flow">
    <div
      ref="$topSentinel"
      class="basic-flow-sentinel basic-flow-sentinel-top"
    ></div>
    <div ref="$flowList" class="basic-flow-list">
      <basic-flow-item
        v-for="(item, index) in currentItems"
        :localIndex="index"
        :key="item.id"
        :data="item"
      />
    </div>
    <div
      ref="$bottomSentinel"
      class="basic-flow-sentinel basic-flow-sentinel-bottom"
    ></div>
  </div>
</template>

<style lang="scss">
.basic-flow {
  position: relative;
  height: 100%;
  max-height: 100%;
  overflow-y: auto;
}
.basic-flow-list {
  display: flex;
  flex-direction: column;
  align-items: center;
}
.basic-flow-sentinel {
  height: 1rem;
  width: 1px;
  float: left;
  z-index: 0;
  border: 1px solid red;
}
.basic-flow-sentinel-top {
  top: 1rem;
}
.basic-flow-sentinel-bottom {
  bottom: 1rem;
}
</style>
