<script setup>
import { inject } from "vue"
import { mediaDB } from "../services/media/db"
import { mediaModifier } from "../services/media/mod"
import { store } from "../states"

const dataSource = inject("dataSource")
async function onSave() {
  await mediaModifier.commit(dataSource)
  mediaDB.forceExpire()
  await mediaDB.val()
  store.fragment.editting = false
}
</script>

<template>
  <div class="basic-flow-cmd">
    <div
      class="basic-flow-cmd-button"
      @click="onSave"
      v-if="store.fragment.editting"
    >
      SAVE
    </div>
  </div>
</template>

<style lang="scss">
.basic-flow-cmd {
  position: fixed;
  display: flex;
  flex-direction: row-reverse;
  width: 100%;
  z-index: 1000;
}

.basic-flow-cmd-button {
  font-size: 0.75rem;
  padding: 0.5rem;
  text-align: center;
  background-color: #ddd;
}
</style>
