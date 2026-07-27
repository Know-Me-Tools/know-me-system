// TJ-ARCH-MOB-001 compliant
//
// The picker's contract, mirroring mobile/test/features/chat/chat_mode_picker_test.dart:
// the list is "every agent, then Local" — never a generic "Agent" row — and the
// trigger answers what the conversation runs on without being opened.
import { describe, expect, it, vi } from 'vitest'

vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => false }))
vi.mock('@prometheus-ags/tauri-plugin-gen-ui', () => ({}))

import { modeChoices, modeLabel } from '../components/ChatModePicker'
import type { AgentOption } from '../stores/chatControlStore'

const AGENTS: AgentOption[] = [
  { id: 'orchestrator-agent', label: 'Orchestrator', protected: true },
  { id: 'writer-agent', label: 'Writer', protected: false },
]

describe('modeLabel', () => {
  it('names the agent rather than saying "Agent"', () => {
    // The whole point of one trigger is that it answers "what is this running
    // on?" without being opened. "Agent" answers nothing.
    expect(modeLabel('agent', 'writer-agent', AGENTS)).toBe('Writer')
  })

  it('says Local for the on-device mode', () => {
    expect(modeLabel('local', null, AGENTS)).toBe('Local')
  })

  it('falls back to Orchestrator when no agent is set', () => {
    expect(modeLabel('uar', null, AGENTS)).toBe('Orchestrator')
  })

  it('shows an unknown id verbatim rather than inventing a name', () => {
    // A stale policy naming a deleted agent must not silently read as the
    // orchestrator — that would misreport where the turn actually goes.
    expect(modeLabel('agent', 'deleted-agent', AGENTS)).toBe('deleted-agent')
  })
})

describe('modeChoices', () => {
  it('lists every agent then Local, and no generic Agent row', () => {
    const choices = modeChoices(AGENTS, 'uar', 'orchestrator-agent', {})
    expect(choices.map((choice) => choice.label)).toEqual(['Orchestrator', 'Writer', 'Local'])
  })

  it('offers only modes drawn from the catalog plus Local', () => {
    // Flutter offers no cloud entry, and two surfaces presenting different sets
    // cannot be compared apples-to-apples — the entire point of this work.
    // `ChatMode` is the plugin's own union, which excludes 'cloud' outright, so
    // the type is the real guard; this pins the produced set behind it.
    const choices = modeChoices(AGENTS, 'uar', 'orchestrator-agent', {})
    expect([...new Set(choices.map((choice) => choice.mode))].sort()).toEqual(['agent', 'local', 'uar'])
  })

  it('routes the orchestrator to uar and every other agent to agent', () => {
    const choices = modeChoices(AGENTS, 'uar', 'orchestrator-agent', {})
    expect(choices.find((choice) => choice.key === 'orchestrator-agent')?.mode).toBe('uar')
    expect(choices.find((choice) => choice.key === 'writer-agent')?.mode).toBe('agent')
  })

  it('selects the specific agent, not merely the agentic mode', () => {
    const choices = modeChoices(AGENTS, 'agent', 'writer-agent', {})
    expect(choices.find((choice) => choice.key === 'writer-agent')?.selected).toBe(true)
    expect(choices.find((choice) => choice.key === 'orchestrator-agent')?.selected).toBe(false)
  })

  it('marks Local selected only in local mode', () => {
    expect(modeChoices(AGENTS, 'local', null, {}).at(-1)?.selected).toBe(true)
    expect(modeChoices(AGENTS, 'uar', 'orchestrator-agent', {}).at(-1)?.selected).toBe(false)
  })

  it('blocks a lane with the reason the runtime gave', () => {
    // The reason IS the point — a disabled row with no explanation teaches the
    // user nothing, which is why Flutter shows it in place of the subtitle.
    const choices = modeChoices(AGENTS, 'uar', 'orchestrator-agent', {
      local: 'No local inference engine in this build.',
    })
    const local = choices.at(-1)
    expect(local?.blockedReason).toBe('No local inference engine in this build.')
    expect(choices.find((choice) => choice.key === 'writer-agent')?.blockedReason).toBeUndefined()
  })

  it('blocks every agent row when the uar lane is unavailable', () => {
    const choices = modeChoices(AGENTS, 'local', null, { uar: 'Runtime is not configured.' })
    expect(choices.filter((choice) => choice.blockedReason).map((choice) => choice.key))
      .toEqual(['orchestrator-agent', 'writer-agent'])
  })

  it('still offers Local when the catalog is empty', () => {
    // A UAR that returns no agents must not leave the user with no way to run
    // anything — the on-device model is always the fallback.
    expect(modeChoices([], 'local', null, {})).toHaveLength(1)
  })
})
