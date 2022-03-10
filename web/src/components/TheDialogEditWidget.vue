<script setup>
import { Dialog } from "../states/dialog"

const props = defineProps({ data: Object })
const WIDGET_TYPES = [
  { code: "q", display: "quote" },
  { code: "m", display: "milestone" },
]

function onCancel() {
  Dialog.dismiss(null)
}

function onConfirm() {
  Dialog.dismiss(props.data)
}
</script>

<template>
  <div class="dialog-form">
    <textarea
      class="dialog-form-control"
      placeholder="content"
      v-model="props.data.text"
    ></textarea>
    <select class="dialog-form-control" v-model="props.data.type">
      <option v-for="typ in WIDGET_TYPES" :value="typ.code">
        {{ typ.display }}
      </option>
    </select>
    <button class="dialog-form-button default margin" @click="onCancel">
      Cancel
    </button>
    <button class="dialog-form-button primary" @click="onConfirm">
      Confirm
    </button>
  </div>
</template>

<style lang="scss"></style>
