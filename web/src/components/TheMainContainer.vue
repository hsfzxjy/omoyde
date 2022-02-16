<script setup>
import BasicFlow from "./BasicFlow.vue"
</script>

<template>
  <div id="main">
    <Suspense>
      <basic-flow />
    </Suspense>
  </div>
</template>

<style lang="scss">
#main {
  width: 100%;
  height: 100%;
}
</style>
