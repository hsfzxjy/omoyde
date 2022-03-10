<script setup>
import { computed, inject } from "vue"
import { ITEM_NULL, ITEM_UNKNOWN } from "../services/media/local"
import { timeGapIsLarge } from "../services/media/misc"
import * as misc from "../services/media/misc"
import { store } from "../states"
import { Dialog } from "../states/dialog"

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
const braceClasses = computed(() => ({
  "basic-flow-item-brace": true,
  added: props.data.isAdded,
  "del-after": props.data.delAfter,
}))

function jumpTo(newIndex = null) {
  if (newIndex === null) newIndex = props.globalIndex
  flowBus.emit("update-index", newIndex, props.localIndex)
}

function onMoveUp() {
  const index = props.globalIndex
  if (index === 0) return
  const newIndex = misc.moveForward(dataSource, index, props.data)
  jumpTo(newIndex)
}
function onMoveDown() {
  const index = props.globalIndex
  if (index === dataSource.countAll().value - 1) return
  const newIndex = misc.moveBackward(dataSource, index, props.data)
  jumpTo(newIndex)
}
async function onEdit() {
  const newItem = await Dialog.show("edit-widget", props.data)
  if (newItem === null) return
  misc.edit(dataSource, props.globalIndex, newItem)
  jumpTo()
}
async function onAddBefore() {
  let item = { kind: "widget", text: "", type: "q" }
  item = await Dialog.show("edit-widget", item)
  if (item === null) return
  misc.addBefore(dataSource, props.globalIndex - 1, item, props.data)
  jumpTo()
}
async function onAddAfter() {
  let item = { kind: "widget", text: "", type: "q" }
  item = await Dialog.show("edit-widget", item)
  if (item === null) return
  misc.addAfter(dataSource, props.globalIndex, item, props.data)
  jumpTo()
}
function onRemove() {
  misc.remove(dataSource, props.globalIndex, props.data)
  jumpTo()
}
</script>

<template>
  <template v-if="show">
    <div class="basic-flow-item-toolbar top">
      <div class="basic-flow-item-toolbar-button" @click="onAddBefore">ADD</div>
      <template v-if="edittable">
        <div class="basic-flow-item-toolbar-button" @click="onEdit">EDIT</div>
        <div class="basic-flow-item-toolbar-button" @click="onRemove">DEL</div>
        <div class="basic-flow-item-toolbar-splitter"></div>
        <div :class="moveDownClasses" @click="onMoveDown">DOWN</div>
        <div :class="moveUpClasses" @click="onMoveUp">UP</div>
      </template>
    </div>
    <div class="basic-flow-item-toolbar bottom">
      <template v-if="showBottomToolbar">
        <div class="basic-flow-item-toolbar-button" @click="onAddAfter">
          ADD
        </div>
      </template>
    </div>
    <div class="basic-flow-item-toolbar spacing"></div>
    <div :class="braceClasses"></div>
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

  &.added {
    border-style: solid;
    border-color: green;
  }
  &.del-after::after {
    content: " ";
    height: 0;
    width: 0;
    border-style: solid;
    position: absolute;

    $h: 0.5rem;
    bottom: -$h * 2;
    left: -$h;
    border-top: $h solid transparent;
    border-bottom: $h solid transparent;
    border-right-color: transparent;
    border-left: $h solid red;
  }
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
