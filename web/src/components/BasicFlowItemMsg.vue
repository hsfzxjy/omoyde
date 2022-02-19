<script setup>
import { computed } from "vue"

const props = defineProps({ data: Object })
const classes = computed(() => ["basic-flow-item-msg", props.data.type])

defineExpose({
  wrapperClasses: computed(() => ["msg", props.data.type]),
})
</script>

<template>
  <span :class="classes">{{ props.data.text }}</span>
</template>

<style lang="scss">
@import "../fonts/custom-font.scss";

.basic-flow-item {
  &.msg.q {
    padding: 0.8rem 10%;

    + .msg.q {
      padding-top: 0;
      margin-top: -0.5rem;
    }
  }
}

.basic-flow-item-msg {
  @extend .custom-font;

  display: inline-block;

  &.m {
    margin: 0.66em 0 0.33em 0;
    text-align: center;
    position: relative;
    font-size: 4rem;
    font-weight: bold;
    order: -1;
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
    font-size: 1.4rem !important;
    font-weight: 500 !important;
  }
}
</style>
