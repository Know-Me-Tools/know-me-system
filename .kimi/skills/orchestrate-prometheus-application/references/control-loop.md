# Control loop rules

## Entry rule

Start with Feynman/KBD when requirements, domain knowledge, source currency, or
architecture boundaries are incomplete. Do not implement first.

## Producer and critic

Select producer and critic roles from the dated model registry. The producer implements; the critic verifies. A producer cannot certify completion.

## Authority

State allowed reads, local writes, external writes, destructive actions, and
publication authority before implementation. Stop before authority expansion.

## Karpathy retention

Record intent, evidence, failures, decisions, and reusable lessons at phase
boundaries. Public docs receive only reviewed synthesis.

## Completion gate

No app or generated project is “working” or “complete” without
`hybrid-runtime-verification` or an equivalent public-boundary verification gate.

## Missing capability routing

Use the skill creator when the missing capability is repeatable process
knowledge. Use the native-agent creator when the missing capability requires an
independent runtime, protocols, persistence, launch, or packaging lifecycle.
