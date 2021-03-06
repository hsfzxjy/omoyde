import { toRaw } from "vue"
import { clone } from "../../utils/misc"
import { ITEM_NULL } from "./local"

export function timeGapIsLarge([dt, o], [pdt, po]) {
  return dt * 1000 + o - (pdt * 1000 + po) >= 1000 * 60 * 30 // 30 minutes
}

export function dto2Date([dt, o]) {
  return new Date(dt * 1000 + o)
}

export function remove(dataSource, index, item) {
  dataSource.remove(index, index, [item.dto])
  return index
}

export function addBefore(dataSource, index, item, anchorItem) {
  item = clone(toRaw(item))
  let { dt, offset, kind } = anchorItem
  if (kind === "image") offset -= 1
  item.dt = dt
  item.offset = offset
  item._modified = true
  dataSource.insert(index, [item])
  return index
}

export function addAfter(dataSource, index, item, anchorItem) {
  item = clone(toRaw(item))
  let { dt, offset } = anchorItem
  item.dt = dt
  item.offset = offset
  item._modified = true
  dataSource.insert(index, [item])
  return index
}

export function edit(dataSource, index, item) {
  item = clone(toRaw(item))
  item._modified = true
  dataSource.inplaceMutate(index, item, [item.dto])
  return index
}

export function moveForward(dataSource, index, item) {
  item = clone(toRaw(item))
  const {
    prev: [{ dto: prev0dto }, { dto: prev1dto, kind: prev1kind }],
    dto,
  } = item
  delete item.prev
  delete item.next
  if (item._origIndex === undefined) item._origIndex = index
  if (timeGapIsLarge(dto, prev0dto)) {
    ;[item.dt, item.offset] = prev0dto
    item.offset += 1
    // if (item._isAdded) item._modified = true
    dataSource.inplaceMutate(index, item, [dto])
    return index
  } else {
    const newDto =
      prev1kind === ITEM_NULL || timeGapIsLarge(prev0dto, prev1dto)
        ? [prev0dto[0], prev0dto[1] - 1]
        : prev1dto

    ;[item.dt, item.offset] = newDto
    dataSource.moveForward(index, item, [dto])
    return index - 1
  }
}

export function moveBackward(dataSource, index, item) {
  item = clone(toRaw(item))
  const {
    next: [{ dto: next0dto }, { dto: next1dto, kind: next1kind }],
    dto,
  } = item
  delete item.prev
  delete item.next
  if (item._origIndex === undefined) item._origIndex = index
  if (timeGapIsLarge(next0dto, dto)) {
    ;[item.dt, item.offset] = next0dto
    item.offset -= 1
    // if (item._isAdded) item._modified = true
    dataSource.inplaceMutate(index, item, [dto])
    return index
  } else {
    const newDto =
      next1kind === ITEM_NULL || timeGapIsLarge(next1dto, next0dto)
        ? next0dto
        : [next1dto[0], next1dto[1] - 1]

    ;[item.dt, item.offset] = newDto
    dataSource.moveBackward(index, item, [dto])
    return index + 1
  }
}
