/** One 1-second live sample: bytes transferred during that second. */
export interface LiveSample {
  timestamp: number
  bytesReceived: number
  bytesSent: number
}

/** Per-interface live series grouped out of the sampler's ring buffer. */
export interface InterfaceLive {
  interfaceId: string
  samples: LiveSample[]
}

/**
 * Per-application live series. `appId` is the canonical executable path;
 * a PID never appears anywhere in the read API.
 */
export interface ApplicationLive {
  appId: string
  processName: string
  executablePath: string | null
  samples: LiveSample[]
}

/**
 * The complete live view. `totals` is the merged per-second series across
 * all interfaces (the throughput chart series); `applications` may be
 * empty — no attributable TCP traffic is not an error.
 */
export interface NetworkLive {
  updatedAt: number | null
  totals: LiveSample[]
  interfaces: InterfaceLive[]
  applications: ApplicationLive[]
  error: string | null
}

/** One aggregated history bucket of `bucketSeconds` starting at `ts`. */
export interface HistoryPoint {
  ts: number
  bytesReceived: number
  bytesSent: number
}

export interface InterfaceHistory {
  interfaceId: string
  points: HistoryPoint[]
}

export interface NetworkHistory {
  since: number
  until: number
  bucketSeconds: number
  totals: HistoryPoint[]
  interfaces: InterfaceHistory[]
}

/** Per-application usage totals over a range, ranked by total bytes. */
export interface ApplicationUsage {
  appId: string
  processName: string
  executablePath: string | null
  bytesReceived: number
  bytesSent: number
}

export type HistoryRange = '1h' | '6h' | '24h'

export const HISTORY_RANGE_SECONDS: Record<HistoryRange, number> = {
  '1h': 60 * 60,
  '6h': 6 * 60 * 60,
  '24h': 24 * 60 * 60,
}
