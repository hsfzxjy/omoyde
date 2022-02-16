<script setup>
import { computed, inject, onBeforeUnmount, onMounted, ref } from "vue"
import { ReactiveURL } from "../infrastructures/reactive_url"
import { randomRange } from "../utils/misc"

const itemObserver = inject("basic-flow-observer")
const props = defineProps({ data: Object, localIndex: Number })
const $wrapper = ref()
const imageSrc = ref("")
const rotDegree = computed(() => `${randomRange(-2.5, 2.5)}deg`)

const filePath = computed(() => `/assets/m/${props.data.pid}.jpg`)
const rURL = ReactiveURL(filePath.value)
  .afterReady((url) => {
    imageSrc.value = url
  })
  .drive()

onMounted(() => {
  itemObserver.observe($wrapper.value)
})
onBeforeUnmount(() => {
  itemObserver.unobserve($wrapper.value)
})

function onImageError(evt) {
  rURL.forceExpire()
}
</script>

<template>
  <div
    ref="$wrapper"
    class="basic-flow-item-wrapper"
    :data-local-index="props.localIndex"
  >
    <img
      v-if="imageSrc"
      :src="imageSrc"
      loading="lazy"
      @error="onImageError"
      class="basic-flow-item-image"
      alt=""
    />
  </div>
</template>

<style lang="scss">
.basic-flow-item-wrapper {
  --width: 80vw;
  width: var(--width);
  height: calc(var(--width) / v-bind(props.data.w) * v-bind(props.data.h));
  border: solid 1px black;
  padding: 0.3rem;
  margin: 1rem 0;
  transform: rotate(v-bind(rotDegree));
}
.basic-flow-item-image {
  width: 100%;
}
</style>
