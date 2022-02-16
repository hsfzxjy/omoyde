export const sleep = (milliseconds) =>
  new Promise((resolve) => {
    setTimeout(resolve, milliseconds)
  })

export const setImmediate = (cb) => setTimeout(cb, 0)
