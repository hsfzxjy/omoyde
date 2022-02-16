<script setup>
import TheSidebarToggleButton from "./TheSidebarToggleButton.vue"
</script>

<template>
  <div id="sidebar" @click="sidebarClicked">
    <the-sidebar-toggle-button class="toggle-menu" />
  </div>
</template>

<style lang="scss">
#sidebar {
  background: green;
  width: 100%;
  height: 100%;
  position: absolute;
  right: 100%;

  > .toggle-menu {
    position: absolute;
    left: 100%;
    top: 0;
    width: 1rem;
    height: 1rem;
  }
}
</style>
