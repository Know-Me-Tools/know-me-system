## Default Permission

Default gen-ui permissions: startup/read/chat intents. Mutating ENTITY commands require the gen-ui:allow-write set explicitly. set-active-lane writes an app setting (a user preference), not entity data, so it belongs here alongside the other chat intents — same category as memory-ingest.

#### This default permission set includes the following:

- `allow-stream-agent-a2ui`
- `allow-get-active-lane`
- `allow-set-active-lane`
- `allow-has-local-engine`
- `allow-run-migrations`
- `allow-load-seeds`
- `allow-attach-sync-shapes`
- `allow-memory-ingest`
- `allow-entity-runtime-start`
- `allow-entity-runtime-stop`
- `allow-entity-list`
- `allow-entity-get`
- `allow-memory-search`
- `allow-graph-expand`
- `allow-scribe-start`
- `allow-scribe-stop`

## Permission Table

<table>
<tr>
<th>Identifier</th>
<th>Description</th>
</tr>


<tr>
<td>

`gen-ui:allow-attach-sync-shapes`

</td>
<td>

Enables the attach_sync_shapes command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-attach-sync-shapes`

</td>
<td>

Denies the attach_sync_shapes command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-entity-create`

</td>
<td>

Enables the entity_create command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-entity-create`

</td>
<td>

Denies the entity_create command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-entity-delete`

</td>
<td>

Enables the entity_delete command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-entity-delete`

</td>
<td>

Denies the entity_delete command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-entity-get`

</td>
<td>

Enables the entity_get command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-entity-get`

</td>
<td>

Denies the entity_get command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-entity-list`

</td>
<td>

Enables the entity_list command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-entity-list`

</td>
<td>

Denies the entity_list command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-entity-runtime-start`

</td>
<td>

Enables the entity_runtime_start command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-entity-runtime-start`

</td>
<td>

Denies the entity_runtime_start command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-entity-runtime-stop`

</td>
<td>

Enables the entity_runtime_stop command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-entity-runtime-stop`

</td>
<td>

Denies the entity_runtime_stop command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-entity-update`

</td>
<td>

Enables the entity_update command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-entity-update`

</td>
<td>

Denies the entity_update command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-get-active-lane`

</td>
<td>

Enables the get_active_lane command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-get-active-lane`

</td>
<td>

Denies the get_active_lane command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-graph-expand`

</td>
<td>

Enables the graph_expand command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-graph-expand`

</td>
<td>

Denies the graph_expand command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-has-local-engine`

</td>
<td>

Enables the has_local_engine command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-has-local-engine`

</td>
<td>

Denies the has_local_engine command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-load-seeds`

</td>
<td>

Enables the load_seeds command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-load-seeds`

</td>
<td>

Denies the load_seeds command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-memory-ingest`

</td>
<td>

Enables the memory_ingest command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-memory-ingest`

</td>
<td>

Denies the memory_ingest command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-memory-search`

</td>
<td>

Enables the memory_search command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-memory-search`

</td>
<td>

Denies the memory_search command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-run-migrations`

</td>
<td>

Enables the run_migrations command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-run-migrations`

</td>
<td>

Denies the run_migrations command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-scribe-start`

</td>
<td>

Enables the scribe_start command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-scribe-start`

</td>
<td>

Denies the scribe_start command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-scribe-stop`

</td>
<td>

Enables the scribe_stop command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-scribe-stop`

</td>
<td>

Denies the scribe_stop command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-set-active-lane`

</td>
<td>

Enables the set_active_lane command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-set-active-lane`

</td>
<td>

Denies the set_active_lane command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-stream-agent-a2ui`

</td>
<td>

Enables the stream_agent_a2ui command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-stream-agent-a2ui`

</td>
<td>

Denies the stream_agent_a2ui command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-write`

</td>
<td>

Allow entity create/update/delete. Grant only to capabilities that own writes.

</td>
</tr>
</table>
