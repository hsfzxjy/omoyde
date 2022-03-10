<script setup>
import { computed } from "vue"
import { store } from "../states"
import TheDialogEditWidget from "./TheDialogEditWidget.vue"

const componentName = computed(() => {
  const name = store.ui.dialog.component
  if (name === null) return TheDialogEditWidget
  return {
    "edit-widget": TheDialogEditWidget,
  }[name]
})
</script>

<template>
  <div class="dialog-cover" v-if="store.ui.dialog.show">
    <component :is="componentName" :data="store.ui.dialog.data" />
  </div>
</template>

<style lang="scss">
.dialog-cover {
  z-index: 10001;
  position: fixed;
  top: 0;
  bottom: 0;
  left: 0;
  right: 0;
  opacity: 0.95;
  background-color: white;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
}

.dialog-form {
  width: 100%;
  padding: 0 1rem;

  textarea {
    height: 50vw;
    max-height: 50vw;
  }

  .margin {
    margin-bottom: 1rem;
  }
}

.dialog-form-control {
  display: block;
  width: 100%;
  margin-bottom: 1rem;
  font-size: 1.1rem;
  padding: 0.25rem;
  border: 1px solid #ced4dad8;
  background-color: #fff;
  transition: border-color 0.15s ease-in-out, box-shadow 0.15s ease-in-out;

  &:focus {
    color: #212529;
    border-color: #baf1d6;
    outline: 0;
    box-shadow: 0 0 0 0.2rem #baf1d6;
  }
}

.dialog-form-button {
  display: inline-block;
  width: 100%;
  padding: 0.5em;
  font-weight: 400;
  line-height: 1.5;
  vertical-align: middle;

  &.default {
    background-color: #e0e2e4;
  }

  &.primary {
    background-color: #0d6efd;
    color: #fff;
  }
}
</style>
