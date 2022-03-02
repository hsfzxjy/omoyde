import { toRaw } from "vue"
import { ITEM_NULL } from "./local"

export function timeGapIsLarge(dt, prevDt) {
  return dt - prevDt >= 1000 * 60 * 30 // 30 minutes
}

export function moveForward(dataSource, index, item) {
  item = toRaw(item)
  const {
    prev: [{ dt: prev0dt }, { dt: prev1dt, kind: prev1kind }],
    dt,
  } = item
  delete item.prev
  delete item.next
  if (timeGapIsLarge(dt, prev0dt)) {
    item.dt = prev0dt + 1
    dataSource.inplaceMutate(index, item)
    return index
  } else {
    const newDt =
      prev1kind === ITEM_NULL || timeGapIsLarge(prev0dt, prev1dt)
        ? prev0dt - 1
        : (prev0dt + prev1dt) / 2
    item.dt = newDt
    dataSource.moveForward(index, item)
    return index - 1
  }
}

export function moveBackward(dataSource, index, item) {
  item = toRaw(item)
  const {
    next: [{ dt: next0dt }, { dt: next1dt, kind: next1kind }],
    dt,
  } = item
  delete item.prev
  delete item.next
  if (timeGapIsLarge(next0dt, dt)) {
    item.dt = next0dt - 1
    dataSource.inplaceMutate(index, item)
    return index
  } else {
    const newDt =
      next1kind === ITEM_NULL || timeGapIsLarge(next1dt, next0dt)
        ? next0dt + 1
        : (next0dt + next1dt) / 2
    item.dt = newDt
    dataSource.moveBackward(index, item)
    return index + 1
  }
}
