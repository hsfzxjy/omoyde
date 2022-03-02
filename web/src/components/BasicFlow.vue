<script setup>
/* this component should be recycled after dataSource changed */

import { onBeforeUnmount, onMounted, provide, reactive, ref, watch } from "vue"
import EventEmitter from "events"
import { getDataSource } from "../services/fragment"
import { store } from "../states"
import { Mutex } from "../utils/aio"
import { DebouncedIntersectionObserver, ScrollHelper } from "../utils/dom"
import { patch, Tumbler } from "../utils/misc"
import { TrashBin } from "../utils/trashbin"
import { LSRefValue } from "../utils/value"
import BasicFlowItem from "./BasicFlowItem.vue"
import BasicFlowTracker from "./BasicFlowTracker.vue"
import { MediaLocalView } from "../services/media/local"

const trashbin = new TrashBin()
const dataSource = await getDataSource()
provide("dataSource", dataSource)
const flowBus = new EventEmitter()
provide("flowBus", flowBus)
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
    if (ctx.holdOn.value()) return

    if (ctx.bottom) await itemsPuller.backward()
    if (ctx.top) await itemsPuller.forward()
  },
  setupCtx: () => ({
    top: false,
    bottom: false,
    holdOn: new Tumbler({ init: () => false, timeout: 500 }),
  }),
  options: { timeout: 150, threshold: 0 },
})
patch(sentinelObserver, {
  holdOn() {
    this._deb.ctx.holdOn.mutate(true)
  },
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
  () => $itemElAt(tracker.localIndex)
)
const $itemElAt = (idx) => $flowList.value.children[idx]

onMounted(() => {
  sentinelObserver.observe($topSentinel.value)
  sentinelObserver.observe($bottomSentinel.value)
})
onBeforeUnmount(() => {
  sentinelObserver.disconnect()
  itemObserver.disconnect()
})

flowBus.on("update-index", (index, anchorLocalIndex) => {
  itemsPuller.jumpTo({ targetIndex: index, initial: false, anchorLocalIndex })
})

const itemsPuller = {
  _mutex: new Mutex(),
  _localView: new MediaLocalView({
    dataSource,
    items: currentItems,
    tracker,
    limit: LIMIT,
  }),
  jumpTo({ targetIndex, anchorLocalIndex = null, initial = false }) {
    return this._mutex.guardOrSkip(async () => {
      if (initial) {
        await this._localView.jumpTo({ targetIndex, loadForward: false })
      } else {
        const recoverScrollTop = scrollHelper.dictate(
          () => anchorLocalIndex && $itemElAt(anchorLocalIndex)
        )
        await this._localView.jumpTo({ targetIndex, loadForward: true })
        await recoverScrollTop(() => $itemElAt(tracker.localIndex))
      }
      sentinelObserver.holdOn()
    })
  },
  forward() {
    return this._mutex.guardOrSkip(async () => {
      // wait until localIndex become fresh
      await itemObserver.nextTick()

      const recoverScrollTop = scrollHelper.dictate()

      if (await this._localView.loadForward()) {
        await recoverScrollTop()
      }
    })
  },
  backward() {
    return this._mutex.guardOrSkip(async () => {
      // wait until localIndex become fresh
      await itemObserver.nextTick()

      const recoverScrollTop = scrollHelper.dictate()

      if (await this._localView.loadBackward()) {
        await recoverScrollTop()
      }
    })
  },
}

await itemsPuller.jumpTo({
  targetIndex: tracker.globalIndex + Math.round(tracker.offset),
  initial: true,
})
</script>

<template>
  <div class="basic-flow-wrapper">
    <div ref="$flowContainer" class="basic-flow-container">
      <div
        ref="$topSentinel"
        class="basic-flow-sentinel basic-flow-sentinel-top"
      ></div>
      <div ref="$flowList" class="basic-flow-list">
        <basic-flow-item
          v-for="(item, index) in currentItems"
          :localIndex="index"
          :globalIndex="tracker.globalIndex - tracker.localIndex + index"
          :key="item.id"
          :data="item"
        />
      </div>
      <div
        ref="$bottomSentinel"
        class="basic-flow-sentinel basic-flow-sentinel-bottom"
      ></div>
    </div>
    <suspense>
      <basic-flow-tracker :currentItemGlobalIndex="tracker.globalIndex" />
    </suspense>
  </div>
</template>

<style lang="scss">
.basic-flow-wrapper {
  position: relative;
  height: 100%;
  max-height: 100%;
}
.basic-flow-container {
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
