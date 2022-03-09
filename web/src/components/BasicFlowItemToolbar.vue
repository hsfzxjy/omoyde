<script setup>
import { computed, inject } from "vue"
import { ITEM_NULL, ITEM_UNKNOWN } from "../services/media/local"
import {
  moveBackward,
  moveForward,
  timeGapIsLarge,
} from "../services/media/misc"
import { store } from "../states"

const props = defineProps({
  data: Object,
  globalIndex: Number,
  localIndex: Number,
})
const dataSource = inject("dataSource")
const flowBus = inject("flowBus")
const show = computed(() => store.fragment.editting)
const edittable = computed(() => props.data.kind === "widget")

const showMoveUp = computed(() => {
  const [prev0, prev1] = props.data.prev
  return prev0.kind !== ITEM_NULL && prev1.kind !== ITEM_UNKNOWN
})
const moveUpClasses = computed(() => ({
  ["basic-flow-item-toolbar-button"]: true,
  hide: !showMoveUp.value,
}))
const showMoveDown = computed(() => {
  const [next0, next1] = props.data.next
  return next0.kind !== ITEM_NULL && next1.kind !== ITEM_UNKNOWN
})
const moveDownClasses = computed(() => ({
  ["basic-flow-item-toolbar-button"]: true,
  hide: !showMoveDown.value,
}))
const showBottomToolbar = computed(() => {
  const { dto, next } = props.data
  return (
    next[0].kind === ITEM_NULL ||
    (next[0].kind !== ITEM_UNKNOWN && timeGapIsLarge(next[0].dto, dto))
  )
})

function onMoveUp() {
  const index = props.globalIndex
  if (index === 0) return
  const newIndex = moveForward(dataSource, index, props.data)
  flowBus.emit("update-index", newIndex, props.localIndex)
}
function onMoveDown() {
  const index = props.globalIndex
  if (index === dataSource.countAll() - 1) return
  const newIndex = moveBackward(dataSource, index, props.data)
  flowBus.emit("update-index", newIndex, props.localIndex)
}
</script>

<template>
  <template v-if="show">
    <div class="basic-flow-item-toolbar top">
      <div class="basic-flow-item-toolbar-button">ADD</div>
      <template v-if="edittable">
        <div class="basic-flow-item-toolbar-button">EDIT</div>
        <div class="basic-flow-item-toolbar-button">DEL</div>
        <div class="basic-flow-item-toolbar-splitter"></div>
        <div :class="moveDownClasses" @click="onMoveDown">DOWN</div>
        <div :class="moveUpClasses" @click="onMoveUp">UP</div>
      </template>
    </div>
    <div class="basic-flow-item-toolbar bottom">
      <template v-if="showBottomToolbar">
        <div class="basic-flow-item-toolbar-button">ADD</div>
      </template>
    </div>
    <div class="basic-flow-item-toolbar spacing"></div>
    <div class="basic-flow-item-brace"></div>
  </template>
</template>

<style lang="scss">
.basic-flow-item-brace {
  position: absolute;
  width: 5vw;
  top: 0;
  left: 5vw;
  bottom: 0;
  border-width: 2px 0 2px 2px;
  border-style: dashed;
  border-color: #777;
}

.basic-flow-item-toolbar {
  display: flex;
  justify-content: space-around;
  width: 80vw;

  &.top {
    border-top: #777 dashed 1px;
    order: -2;
  }

  &.bottom {
    border-bottom: #777 dashed 1px;
    order: 10;
  }

  &.spacing {
    height: 0.8rem;
    order: 11;

    + .basic-flow-item-brace {
      bottom: 0.8rem;
    }
  }
}
.basic-flow-item-toolbar-splitter {
  flex-grow: 0.6;
}

.basic-flow-item-toolbar-button {
  font-size: 0.75rem;
  padding: 0.5rem;
  text-align: center;
  background-color: #ddd;

  &.hide {
    visibility: hidden;
  }
}
</style>
