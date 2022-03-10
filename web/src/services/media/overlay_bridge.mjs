export function Bridge(bottom_size, adds, dels) {
  let size =
    bottom_size -
    (dels.length - 1) +
    adds.reduce((s, [_, addx]) => s + addx.length, 0)
  return {
    _internal() {
      return { adds, dels, size }
    },
    squash() {
      let ia = 0
      let id = 0
      while (ia < adds.length && id < dels.length) {
        const [addi, addx] = adds[ia]
        const addxl = addx.length
        const [deli, _] = dels[id]
        const delta = addi - deli
        if (delta < -1) {
          ia++
        } else if (delta > 0) {
          id++
        } else if (delta === 0) {
          const paddi = ia === 0 ? null : adds[ia - 1][0]
          let n = 0
          let common
          while (true) {
            common = true
            for (let i = 0, j = id; i < n && j > 0; i++, j--) {
              const [delj, deljx] = dels[j]
              const addc = addx[n - 1 - i]
              if (
                delj === paddi ||
                delj !== deli - i ||
                delj !== addc._origIndex ||
                deljx[0] !== addc.dt ||
                addc._modified
              ) {
                common = false
                break
              }
            }
            if ((common && n) || n === addxl || n === id) break
            n++
          }
          if (n && common) {
            addx.splice(0, n)
            if (!addx.length && ia !== adds.length - 1) {
              adds.splice(ia, 1)
              ia -= 1
            }
            dels.splice(id - n + 1, n)
            id -= n
          }
          ia++
          id++
        } else {
          /* else if (delta === -1) */
          let n = 0
          const naddi = ia === adds.length - 1 ? null : adds[ia + 1][0]
          const dl = dels.length
          let common
          while (true) {
            common = true
            for (let i = 0, j = id; i < n && j < dl; i++, j++) {
              const [delj, deljx] = dels[j]
              const addc = addx[addxl - n + i]
              if (
                delj === naddi ||
                delj !== deli + i ||
                delj !== addc._origIndex ||
                deljx[0] !== addc.dt ||
                addc._modified
              ) {
                common = false
                break
              }
            }
            if ((common && n) || n === addxl || n + id === dl) break
            n++
          }
          if (n && common) {
            addx.splice(addxl - n, n)
            if (!addx.length && ia !== adds.length - 1) {
              adds.splice(ia, 1)
              ia -= 1
            }
            dels.splice(id, n)
            id -= 1
          }
          ia++
          id++
        }
      }
    },
    // remove range [tstart, tend]
    remove(tstart, tend, extras = []) {
      const [bstart, _, range] = this.range_t2b(tstart, tend, true)
      let id = dels.length - 1
      for (let i = range.length - 1; i >= 0; i--) {
        let x = range[i]
        if (typeof x === "number") {
          x += bstart
          while (dels[id][0] > x) id--
          dels.splice(id + 1, 0, [x, extras[i]])
        } else {
          const {
            handle: { ia, i },
          } = x
          adds[ia][1].splice(i, 1)
          if (!adds[ia][1].length && ia !== adds.length - 1) adds.splice(ia, 1)
        }
      }
      size -= tend - tstart + 1
    },
    // insert items after tstart-th item
    insert(tstart, items) {
      if (tstart === -1) {
        const first = adds[0]
        if (first[0] !== -1) {
          adds.splice(0, 0, [-1, items])
        } else {
          first[1].splice(0, 0, ...items)
        }
      } else if (tstart === size - 1) {
        adds[adds.length - 1][1].splice(+Infinity, 0, ...items)
      } else {
        const [bstart, _, [x, xnext]] = this.range_t2b(tstart, tstart + 1, true)
        let ia, i
        if (typeof x !== "number") {
          const { handle } = x
          ia = handle.ia
          i = handle.i + 1
        } else if (typeof xnext !== "number") {
          const { handle } = xnext
          ia = handle.ia
          i = handle.i
        } else {
          ia = 0
          while (bstart > adds[ia][0]) ia++
          adds.splice(ia, 0, [bstart, []])
          i = 0
        }
        adds[ia][1].splice(i, 0, ...items)
      }
      size += items.length
    },
    size() {
      return size
    },
    has_del_at_front() {
      return adds[0][0] !== -1 && dels.length > 1 && dels[1][0] === 0
    },
    range_t2b(tstart, tend, with_handle = false, with_da = false) {
      if (tstart < 0) tstart = 0
      if (tend >= size) tend = size - 1
      if (tstart > tend) {
        const ret = [tstart, tend, []]
        return with_da ? [ret, []] : []
      }

      function make_ret() {
        let i = ret.length - 1
        while (i >= 0 && typeof ret[i] !== "number") i--
        const new_r = [bstart, bstart + (i === -1 ? -1 : ret[i]), ret]
        return with_da ? [new_r, das] : new_r
      }

      let ia = 0
      let id = 0
      let la = adds.length
      let ld = dels.length
      let bi = -2
      let ti = -1
      let ri = -1
      let found_start = false
      let bstart = 0
      const tn = tend - tstart + 1
      const ret = new Array(tn)
      const das = with_da ? new Array(tn) : []
      while (ia < la || id < ld) {
        function wrap_addx(i) {
          if (!with_handle) return addx[i]
          else return { handle: { adds, ia, i }, item: addx[i] }
        }

        let ndel = 0
        const deli = (dels[id] || [])[0]
        const [addi, addx] = adds[ia] || []
        const addxl = addx && addx.length
        if (ia == la || (id < ld && deli <= addi)) {
          const defer = ia < la && deli === addi
          id += 1
          if (defer) {
            ndel = 1
          } else {
            if (!found_start && tstart <= ti + deli - bi - 1) {
              found_start = true
              bstart = bi + tstart - ti
              bi = bstart - 1
              ti = tstart - 1
            }
            if (!found_start) {
              ti += deli - bi - 1
              bi = deli
            } else {
              while (ri < tn - 1 && bi < deli - 1) {
                ti++
                bi++
                ri++
                ret[ri] = bi - bstart
              }
              if (with_da && bi === deli - 1 && ri >= 0) das[ri] = true
              bi += 1
              if (ri === tn - 1) return make_ret()
            }
            continue
          }
        }

        if (!found_start) {
          if (tstart <= addi - bi + ti - ndel) {
            bstart = bi + tstart - ti
            found_start = true
            bi = bstart - 1
            ti = tstart - 1
          } else if (tstart <= addi - bi + ti - ndel + addxl) {
            found_start = true
            bstart = addi + 1
            let i = tstart - (addi - bi + ti - ndel) - 2
            ti += i + 1 + addi - bi - 1
            if (with_da && i === -1 && ri >= 0 && ndel) das[ri] = true
            do {
              ri++
              ti++
              i++
              ret[ri] = wrap_addx(i)
            } while (ri < tn - 1 && i < addxl - 1)
            if (
              with_da &&
              i === addxl - 1 &&
              id < ld &&
              dels[id][0] === addi + 1
            )
              das[ri] = true
            bi = bstart - 1
            if (ri === tn - 1) return make_ret()
            else {
              bi = addi
              ia++
              continue
            }
          }
        }

        if (!found_start) {
          ti += addi - bi + addxl - ndel
          bi = addi
        } else {
          if (tend <= addi - bi + ti - ndel) {
            while (ri < tn - 1) {
              bi++
              ti++
              ri++
              ret[ri] = bi - bstart
            }
            if (with_da && tend <= addi - bi + ti - ndel && ndel && ri >= 0)
              das[ri] = true
            return make_ret()
          } else {
            while (bi < addi - ndel) {
              bi++
              ti++
              ri++
              ret[ri] = bi - bstart
            }
            if (with_da && ndel && ri >= 0) das[ri] = true
            bi += ndel
            let i = -1
            while (ri < tn - 1 && i < addxl - 1) {
              i++
              ri++
              ti++
              ret[ri] = wrap_addx(i)
            }
            if (
              with_da &&
              i === addxl - 1 &&
              id < ld &&
              dels[id][0] === addi + 1
            )
              das[ri] = true
            if (ri === tn - 1) {
              return make_ret()
            }
          }
        }
        ia += 1
      }
    },
  }
}
