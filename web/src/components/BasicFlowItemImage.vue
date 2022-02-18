<script setup>
import { computed, ref } from "vue"
import { ReactiveURL } from "../infrastructures/reactive_url"
import { randomRange } from "../utils/misc"

const props = defineProps({ data: Object })
const imageSrc = ref("")
const rotDegree = computed(() => `${randomRange(-2.5, 2.5)}deg`)

defineExpose({
  wrapperClasses: ["image"],
})

const filePath = computed(() => `/assets/m/${props.data.pid}.jpg`)
const rURL = ReactiveURL(filePath.value)
  .afterReady((url) => {
    imageSrc.value = url
  })
  .drive()

function onImageError(_evt) {
  rURL.forceExpire()
}
</script>

<template>
  <div
    class="basic-flow-item-image-wrapper"
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
.basic-flow-item-image-wrapper {
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
