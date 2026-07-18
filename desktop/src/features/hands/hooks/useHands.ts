// TJ-ARCH-MOB-001 compliant
export function useHands() {
  return { hands: [
    { name: 'Morning brief', status: 'Idle', purpose: 'Summarizes overnight memory writes and calendar into one note at 6am.', meta: 'Next run: 6:00 AM tomorrow · 12 runs this week · On-device' },
    { name: 'Sync watchdog', status: 'Running', purpose: 'Runs every 15 minutes, resolves conflicts, and pings if a device falls behind.', meta: 'Running now · 640 runs this week · On-device' },
    { name: 'Weekly roadmap digest', status: 'Scheduled', purpose: 'Compiles project memory into a written update for async standups.', meta: 'Next run: Friday 9:00 AM · 1 run this week · My server' },
  ] }
}
