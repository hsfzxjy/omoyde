<script setup>
import TheSidebar from "./components/TheSidebar.vue"
import TheMainContainer from "./components/TheMainContainer.vue"
import { store } from "./states"
import { computed } from "vue"

const mainWrapperClass = computed(() => ({
  ["sidebar-expanded"]: !store.ui.sidebarExpanded,
}))
</script>

<template>
  <div id="main-wrapper" :class="mainWrapperClass">
    <the-sidebar />
    <the-main-container />
  </div>
</template>

<style lang="scss">
* {
  box-sizing: border-box;
}

html {
  font-size: 20px;
}

button {
  padding: 0;
  background-color: transparent;
  background-image: none;
  border: 0;
  padding: 0;
  line-height: inherit;
  color: inherit;
}

html,
body,
#app,
#main-wrapper {
  padding: 0;
  margin: 0;
  height: 100%;
  max-height: 100%;
}

#app {
  overflow-x: hidden;
}

#main-wrapper {
  position: relative;
  transition: transform 0.5s ease-in-out;
}
</style>
