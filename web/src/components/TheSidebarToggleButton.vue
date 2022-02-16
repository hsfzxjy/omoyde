<script setup>
import { computed } from "vue"
import { store } from "../states"

function sidebarClicked() {
  store.ui.sidebarExpanded = !store.ui.sidebarExpanded
}

const buttonClass = computed(() => ({
  ["vt-hamburger"]: true,
  ["is-active"]: store.ui.sidebarExpanded,
}))
</script>

<template>
  <button @click="sidebarClicked" :class="buttonClass">
    <span class="vt-hamburger-container">
      <span class="vt-hamburger-top"></span>
      <span class="vt-hamburger-middle"></span>
      <span class="vt-hamburger-bottom"></span>
    </span>
  </button>
</template>

<style lang="scss">
.vt-hamburger {
  display: flex;
  justify-content: center;
  align-items: center;
}
.vt-hamburger-container {
  position: relative;
  width: 16px;
  height: 14px;
  overflow: hidden;
}
.vt-hamburger-top,
.vt-hamburger-middle,
.vt-hamburger-bottom {
  position: absolute;
  width: 16px;
  height: 2px;
  background-color: #213547;
  transition: top 0.25s, background-color 0.5s, transform 0.25s;
}
.vt-hamburger-top {
  top: 0;
  left: 0;
  transform: translate(0);
}
.vt-hamburger-middle {
  top: 6px;
  left: 0;
  transform: translate(8px);
}
.vt-hamburger-bottom {
  top: 12px;
  left: 0;
  transform: translate(4px);
}
.vt-hamburger:hover {
  .vt-hamburger-top {
    top: 0;
    left: 0;
    transform: translate(4px);
  }
  .vt-hamburger-middle {
    top: 6px;
    left: 0;
    transform: translate(0);
  }
  .vt-hamburger-bottom {
    top: 12px;
    left: 0;
    transform: translate(8px);
  }
}
.vt-hamburger.is-active {
  .vt-hamburger-top {
    top: 6px;
    transform: translate(0) rotate(225deg);
  }
  .vt-hamburger-middle {
    top: 6px;
    transform: translate(16px);
  }
  .vt-hamburger-bottom {
    top: 6px;
    transform: translate(0) rotate(135deg);
  }
}
</style>
