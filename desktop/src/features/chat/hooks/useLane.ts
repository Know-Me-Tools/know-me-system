// TJ-ARCH-MOB-001 compliant
import { useLaneStore } from '../stores/laneStore'

/** Cloud/local lane state for the chat surface. Components import this, not the store. */
export function useLane() {
  return {
    lane: useLaneStore((state) => state.lane),
    localAvailable: useLaneStore((state) => state.localAvailable),
    loadProgress: useLaneStore((state) => state.loadProgress),
    throughput: useLaneStore((state) => state.throughput),
    error: useLaneStore((state) => state.error),
    switchLane: useLaneStore((state) => state.switchLane),
    initLane: useLaneStore((state) => state.init),
  }
}
