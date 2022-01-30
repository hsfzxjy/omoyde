let PhotoTable = {
  // for a specific MPID at a time
  entries: new Array(),
  pid2entry: new Map(),
  selected_pids: new Set(),
  reviewed_pids: new Set(),
  current_list: new Array(),
  init() {},
  clear() {
    this.pids = []
    this.entries = []
    this.pid2entry.clear()
    this.selected_pids.clear()
    this.reviewed_pids.clear()
    this.current_list = []
  },
  parse_entry(line) {
    line = line.split(" ")
    // prettier-ignore
    let [pid, mpid, filename, status, selected, exif_time, ctime, commit_time] = line
    return {
      pid: parseInt(pid),
      mpid,
      filename,
      selected: selected === "true",
      status,
      reviewed: status === "Committed",
      exif_time: parse_date(exif_time),
      ctime: parse_date(ctime),
      commit_time: parse_date(commit_time),
    }
  },
  from_lines(lines) {
    this.clear()
    const parse_entry = this.parse_entry
    for (const line of lines) {
      const entry = parse_entry(line)
      this.entries.push(entry)
      this.pid2entry.set(entry.pid, entry)
      if (entry.selected) this.selected_pids.add(entry.pid)
      if (entry.reviewed) this.reviewed_pids.add(entry.pid)
    }
    this.entries.sort((a, b) => a.exif_time - b.exif_time || -1)
  },

  async from_cmd(args) {
    this.from_lines(await run_cmd(args))
  },

  async reload(mpid) {
    await this.from_cmd(["list", mpid])
    await Toolbar.reload()
  },

  update_current_list() {
    let lst
    switch (Toolbar.filter) {
      case "all":
        lst = this.entries
        break
      case "reviewed":
        lst = this.entries.filter((x) => x.reviewed)
        break
      case "unreviewed":
        lst = this.entries.filter((x) => !x.reviewed)
        break
      case "selected":
        lst = this.entries.filter((x) => x.selected)
        break
      case "unselected":
        lst = this.entries.filter((x) => !x.selected)
        break
    }
    this.current_list = lst
  },
}

let MPTable = {
  mpid2entry: new Map(),
  async init() {
    await this.from_cmd(["mount"])
    await SideBar.reload()
  },
  clear() {
    this.mpid2entry.clear()
  },
  parse_entry(line) {
    let [mpid, path, alias] = line.split(" ")
    if (alias === "<NOALIAS>") alias = path
    return { mpid, path, alias }
  },
  from_lines(lines) {
    this.clear()
    for (const line of lines) {
      const entry = this.parse_entry(line)
      this.mpid2entry.set(entry.mpid, entry)
    }
  },
  async from_cmd(args) {
    this.from_lines(await run_cmd(args))
  },
}

function parse_date(o) {
  const try_int = +o
  if (Number.isNaN(try_int)) return o
  return new Date(try_int * 1000)
}

async function run_cmd(args) {
  const response = await window.fetch("/cmd", {
    method: "POST",
    body: JSON.stringify(args),
    headers: {
      ["Content-Type"]: "application/json",
    },
  })
  const content = await response.text()
  return content.trim().split("\n")
}

const $ = document.querySelector.bind(document)
const h = document.createElement.bind(document)
const listen = ($el, name, func) => $el.addEventListener(name, func)

const SideBar = {
  init() {},
  $el() {
    if (!this._$el) this._$el = $("#main > .sidebar > ul")
    return this._$el
  },
  derive_select_fn() {
    let $selected = null
    return ($item) => {
      if ($item === $selected) return false
      if ($selected) $selected.classList.remove("active")
      $item.classList.add("active")
      $selected = $item
      return true
    }
  },
  async reload() {
    const $el = this.$el()
    const select_fn = this.derive_select_fn()
    this.selected_mpid = null
    $el.innerHTML = ""
    for (const entry of MPTable.mpid2entry.values()) {
      const $item = h("li")
      $item.innerText = entry.alias
      listen($item, "click", async () => {
        this.selected_mpid = entry.mpid
        if (select_fn($item)) await PhotoTable.reload(entry.mpid)
      })
      $el.appendChild($item)
    }
  },
}

const Toolbar = {
  page_size: 10,
  init() {
    const $filter = $("#filter")
    const $since = $("#since")
    this.filter = $filter.value
    this.since = 0

    this.filter_changed = listen($filter, "change", () => {
      this.filter = $filter.value
      this.reload()
    })
    listen($since, "change", () => {
      this.since = parseInt($since.value)
      ClientArea.reload()
    })
  },

  reload() {
    console.log("tb")
    PhotoTable.update_current_list()
    const $since = $("#since")
    $since.innerHTML = ""
    const length = PhotoTable.current_list.length
    for (let i = 0; i < length; i += this.page_size) {
      const $option = h("option")
      $option.innerHTML = i
      $option.setAttribute("value", i)
      $since.appendChild($option)
    }
    ClientArea.reload()
  },
}

const ClientArea = {
  create_item(entry) {
    const $item = h("div")
    $item.classList.add("item")
    const ext = entry.filename.match(/\.([^\.]+)/)[0]
    const src = `/img/${entry.pid}${ext}`
    const $img = h("img")
    $img.src = src
    $item.appendChild($img)
    const $meta = h("div")
    $meta.classList.add("meta")
    $meta.innerHTML = `
    PID: ${entry.pid}  <br>
    FNAME: ${entry.filename} <br>
    CTIME: ${entry.ctime} <br>
    ETIME: ${entry.exif_time} <br>
    SELECTED: ${entry.selected}<br>
    STATUS: ${entry.status} <br>
    `
    $item.appendChild($meta)
    return $item
  },
  reload() {
    const $clientArea = $("#main .client-area")
    $clientArea.innerHTML = ""
    if (!PhotoTable.current_list.length) {
      $clientArea.innerHTML =
        '<span class="empty"><span>No Photos</span></span>'
      return
    }
    for (
      let i = Toolbar.since;
      i < Toolbar.since + Toolbar.page_size &&
      i < PhotoTable.current_list.length;
      i++
    ) {
      const $item = this.create_item(PhotoTable.current_list[i])
      $clientArea.appendChild($item)
    }
  },
}

async function main() {
  await SideBar.init()
  await Toolbar.init()
  await MPTable.init()
}
