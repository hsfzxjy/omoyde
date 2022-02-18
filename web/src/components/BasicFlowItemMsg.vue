<script setup>
import { computed } from "vue"

const props = defineProps({ data: Object })
const classList = computed(() => ["basic-flow-item-msg", props.data.type])

defineExpose({
  wrapperClasses: ["msg", props.data.type],
})
</script>

<template>
  <span :class="classList" ref="$el">{{ props.data.text }}</span>
</template>

<style lang="scss">
@import "../fonts/custom-font.scss";

.basic-flow-item-msg {
  @extend .custom-font;

  display: inline-block;

  &.m {
    margin: 5rem 0;
    text-align: center;
    position: relative;
    font-size: 4rem;
    font-weight: bold;
    &:before,
    &:after {
      content: " ";
      position: absolute;
      left: -0.25em;
      right: -0.25em;
      border-style: solid;
      border-color: black;
      border-width: 1px 0;
      height: 3px;
    }
    &:before {
      top: 0;
    }
    &:after {
      bottom: 0;
    }
  }

  &.q {
    align-self: flex-start;
    margin: 1rem 10%;
    font-size: 1.8rem !important;
    font-weight: 500 !important;
  }
}
</style>
