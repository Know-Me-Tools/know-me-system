# Personal peer replication — AFC C12

Native profile, driven by the host's existing lifetime. No daemon, scheduler,
organization authority replica or second CRDT engine is installed.

Fabric exchanges both peers' credentials on a bounded QUIC admission stream and
checks the TLS-proven endpoint key. Discovery/pairing alone never authorizes data.
OFP revalidates the admitted credential before each round, fences consent revision
changes, and checks current grant/offline expiry before every document and frame.
The bounded disclosure transaction serializes with consent changes. Revocation
closes admitted sessions after the authority change commits; dispatched plaintext
cannot be recalled. A round can hold that authority transaction for at most 30 seconds.

Consent is an atomic, fsynced, versioned local file with stable revision/CAS and
append-only inspectable event history. It contains public keys, credential hashes,
selected document IDs, validity and tombstones, never raw credentials/private keys.
Each new event preserves the public identity, exact consent scopes/validity and
document classifications/tombstones at that revision. Older events without these
snapshots remain readable and are marked as lacking historical scope details.
One host owns that file and its write lock. Document classes are an exact closed
vocabulary: personal notes, conversations, memory and personal settings. Unknown
classes, unregistered IDs, class changes, grants/policies/organization membership
and runtime execution authority cannot enter a personal merge.

The host supplies `SurrealCrdtStore` over its already-open durable database and one
file-backed `RedbOpStore`. Local edits queue before checkpoint; queued operations
replay on recovery. Full Loro snapshots are exchanged for a bounded manual round,
so retries and disconnected concurrent edits converge through the existing merge
engine. The durable op journal is retained; automated journal compaction and large
document/chunk transfer are outside this profile. Frames over 8 MiB deny execution.

The native Settings panel configures a local owner-asserted personal partition,
imports identity only for this session, grants selected registered documents with
explicit offline/overall validity, reads/edits documents, tombstones removed ones,
revokes peers and displays history/current status. Relay fallback and mDNS are
disabled; copy the public direct address between devices with a reachable route.
One device explicitly listens, then the other connects; reconnect is another round.

Identity import must match the persisted public key. Explicit recovery records a
replacement key and revokes every existing peer grant. Both devices must establish
fresh consent; no old grants silently follow the replacement. Keys and credentials
are not persisted and identity starts locked after restart. This is session import
and manual secure backup, **not** automatic OS-keychain recovery. Previously copied
plaintext is not erased by peer revocation. Tombstones prevent document resurrection
and purge the receiving local checkpoint when the authorized removal is received.

Organization membership, runtime authority, hands/schedules, skill registrations,
models and arbitrary business tables remain unsupported. Historical UI enum defaults
are compatibility declarations and do not authorize this service. Browser transport,
automatic discovery, background gossip, held-out key recovery/device and real network
qualification remain separate. No runtime/physical-device validation was run during
this production delivery; source implementation and qualification are distinct.
