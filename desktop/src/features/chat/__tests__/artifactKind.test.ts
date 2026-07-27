// TJ-ARCH-MOB-001 compliant
//
// Artifact kind dispatch. React handled a2ui, svg, and "everything else", so
// markdown, mermaid, video, link previews, code, documents, attachments and
// every `agui_*` runtime event collapsed into a collapsed <details> showing raw
// JSON — including `agui_confirmation`, which on Flutter asks the user a
// question. The aliases below are Flutter's; a divergence here is invisible
// until someone compares the two surfaces on the same artifact.
import { describe, expect, it, vi } from 'vitest'

// The renderer resolves stored paths through Tauri's asset scheme. Stubbed so
// these stay pure-function tests rather than needing a Tauri host.
vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
  convertFileSrc: (path: string) => `asset://localhost/${encodeURIComponent(path)}`,
}))

import { aguiSummary, aguiTitle, artifactKind, mediaUrl } from '../components/ArtifactKindBody'

describe('artifactKind', () => {
  it('maps every markdown alias', () => {
    expect(artifactKind('markdown')).toBe('markdown')
    expect(artifactKind('md')).toBe('markdown')
    expect(artifactKind('document')).toBe('markdown')
  })

  it('maps every media alias', () => {
    expect(artifactKind('mermaid')).toBe('mermaid')
    expect(artifactKind('svg')).toBe('svg')
    expect(artifactKind('image/svg+xml')).toBe('svg')
    expect(artifactKind('video')).toBe('video')
    expect(artifactKind('mp4')).toBe('video')
    expect(artifactKind('webm')).toBe('video')
  })

  it('maps every link alias', () => {
    expect(artifactKind('opengraph')).toBe('link')
    expect(artifactKind('open_graph')).toBe('link')
    expect(artifactKind('link')).toBe('link')
  })

  it('maps documents and attachments to the document body', () => {
    expect(artifactKind('pdf')).toBe('document')
    expect(artifactKind('doc')).toBe('document')
    expect(artifactKind('docx')).toBe('document')
    expect(artifactKind('attachment')).toBe('document')
  })

  it('maps both a2ui spellings', () => {
    expect(artifactKind('a2ui')).toBe('a2ui')
    expect(artifactKind('application/a2ui+json')).toBe('a2ui')
  })

  it('normalises case and surrounding whitespace', () => {
    expect(artifactKind('  MERMAID  ')).toBe('mermaid')
  })

  it('routes any agui_ prefix before the kind arms', () => {
    // Flutter's prefix test runs BEFORE its switch, so an agui_ kind never
    // reaches a kind arm even when it collides with one.
    expect(artifactKind('agui_state_delta')).toBe('agui')
    expect(artifactKind('agui_confirmation')).toBe('agui')
    expect(artifactKind('agui_activity_search')).toBe('agui')
    expect(artifactKind('agui_code')).toBe('agui')
  })

  it('falls back for an unrecognised kind rather than guessing', () => {
    expect(artifactKind('something-new')).toBe('unknown')
  })
})

describe('mediaUrl', () => {
  it('unwraps a url envelope', () => {
    expect(mediaUrl('{"url":"https://example.test/a.mp4"}')).toBe('https://example.test/a.mp4')
  })

  it('treats a bare string as the url', () => {
    expect(mediaUrl('https://example.test/a.mp4')).toBe('https://example.test/a.mp4')
  })

  it('falls back to the content when the envelope has no url', () => {
    expect(mediaUrl('{"poster":"x"}')).toBe('{"poster":"x"}')
  })

  it('resolves a stored path through the asset scheme', () => {
    // A user-attached video carries a durable `path`, not a `url`. Without this
    // the whole JSON envelope was handed to <video src> and the player was
    // permanently broken.
    expect(mediaUrl('{"path":"/data/attachments/c/abc.mp4","name":"clip.mp4"}'))
      .toBe(`asset://localhost/${encodeURIComponent('/data/attachments/c/abc.mp4')}`)
  })

  it('prefers a url over a path when both are present', () => {
    expect(mediaUrl('{"url":"https://example.test/a.mp4","path":"/local/a.mp4"}'))
      .toBe('https://example.test/a.mp4')
  })
})

describe('aguiTitle', () => {
  it('names the known event types', () => {
    expect(aguiTitle('agui_state_snapshot')).toBe('Agent state synchronized')
    expect(aguiTitle('agui_state_delta')).toBe('Agent state updated')
    expect(aguiTitle('agui_messages_snapshot')).toBe('Conversation synchronized')
    expect(aguiTitle('agui_raw')).toBe('External agent event')
  })

  it('humanises an activity kind, dropping the delta suffix', () => {
    expect(aguiTitle('agui_activity_web_search')).toBe('Web search activity')
    expect(aguiTitle('agui_activity_web_search_delta')).toBe('Web search activity')
  })

  it('falls back for an unknown agui event', () => {
    expect(aguiTitle('agui_something')).toBe('Agent event')
  })
})

describe('aguiSummary', () => {
  it('lists up to three changed keys', () => {
    expect(aguiSummary('{"a":1,"b":2}')).toBe('Updated a, b')
  })

  it('counts beyond three rather than listing them all', () => {
    expect(aguiSummary('{"a":1,"b":2,"c":3,"d":4}')).toBe('4 updates')
  })

  it('reports an empty object honestly', () => {
    expect(aguiSummary('{}')).toBe('No visible fields')
  })

  it('counts a messages snapshot', () => {
    expect(aguiSummary('[1,2,3]')).toBe('3 messages')
  })

  it('degrades to a hint when the payload is not JSON', () => {
    expect(aguiSummary('not json')).toBe('Inspect event details')
  })
})
