<script setup>
import { inject } from "vue"
import { mediaModifier } from "../services/media/mod"

const dataSource = inject("dataSource")
function onSave() {
  mediaModifier.commit(dataSource)
}
</script>

<template>
  <div class="basic-flow-cmd">
    <div class="basic-flow-cmd-button" @click="onSave">SAVE</div>
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
