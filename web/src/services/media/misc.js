import { toRaw } from "vue"
import { clone } from "../../utils/misc"
import { ITEM_NULL } from "./local"

export function timeGapIsLarge(dt, prevDt) {
  return dt - prevDt >= 1000 * 60 * 30 // 30 minutes
}

function between(x, y) {
  const MAX_STEP = 0.1
  const sign = x > y ? 1 : -1
  const gap = sign === 1 ? x - y : y - x
  if (gap >= 2 * MAX_STEP) {
    return x - sign * MAX_STEP
  } else {
    return (x + y) / 2
  }
}

export function moveForward(dataSource, index, item) {
  item = clone(toRaw(item))
  const {
    prev: [{ dt: prev0dt }, { dt: prev1dt, kind: prev1kind }],
    dt,
  } = item
  delete item.prev
  delete item.next
  if (timeGapIsLarge(dt, prev0dt)) {
    item.dt = prev0dt + 1
    dataSource.inplaceMutate(index, item, [dt])
    return index
  } else {
    const newDt =
      prev1kind === ITEM_NULL || timeGapIsLarge(prev0dt, prev1dt)
        ? prev0dt - 1
        : between(prev0dt, prev1dt)

    item.dt = newDt
    dataSource.moveForward(index, item, [dt])
    return index - 1
  }
}

export function moveBackward(dataSource, index, item) {
  item = clone(toRaw(item))
  const {
    next: [{ dt: next0dt }, { dt: next1dt, kind: next1kind }],
    dt,
  } = item
  delete item.prev
  delete item.next
  if (timeGapIsLarge(next0dt, dt)) {
    item.dt = next0dt - 1
    dataSource.inplaceMutate(index, item, [dt])
    return index
  } else {
    const newDt =
      next1kind === ITEM_NULL || timeGapIsLarge(next1dt, next0dt)
        ? next0dt + 1
        : between(next0dt, next1dt)

    item.dt = newDt
    dataSource.moveBackward(index, item, [dt])
    return index + 1
  }
}
