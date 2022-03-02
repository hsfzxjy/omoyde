export function timeGapIsLarge(dt, prevDt) {
  return dt - prevDt >= 1000 * 60 * 30 // 30 minutes
}
