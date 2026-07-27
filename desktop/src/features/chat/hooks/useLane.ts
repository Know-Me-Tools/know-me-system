// TJ-ARCH-MOB-001 compliant
import { isModelOperationBusy, useLaneStore } from '../stores/laneStore'

/** Cloud/local lane state for the chat surface. Components import this, not the store. */
export function useLane() {
  return {
    lane: useLaneStore((state) => state.lane),
    localAvailable: useLaneStore((state) => state.localAvailable),
    uarSupported: useLaneStore((state) => state.uarSupported),
    uarConfigured: useLaneStore((state) => state.uarConfigured),
    uarStatus: useLaneStore((state) => state.uarStatus),
    loadProgress: useLaneStore((state) => state.loadProgress),
    /** True while the local model is downloading, verifying, loading or
     *  cancelling. The composer gates on this — sending mid-download reaches a
     *  lane whose weights are not resident. */
    modelBusy: useLaneStore((state) => isModelOperationBusy(state.loadProgress)),
    throughput: useLaneStore((state) => state.throughput),
    error: useLaneStore((state) => state.error),
    switchLane: useLaneStore((state) => state.switchLane),
    initLane: useLaneStore((state) => state.init),
    cancelPreparation: useLaneStore((state) => state.cancelPreparation),
  }
}
