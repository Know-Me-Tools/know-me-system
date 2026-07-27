## Default Permission

Default gen-ui permissions: startup/read/chat intents. Mutating ENTITY commands require the gen-ui:allow-write set explicitly. set-active-lane writes an app setting (a user preference), not entity data, so it belongs here alongside the other chat intents — same category as memory-ingest.

#### This default permission set includes the following:

- `allow-stream-agent-a2ui`
- `allow-chat-approve-tool-confirmation`
- `allow-stage-attachment`
- `allow-resolve-remote-content`
- `allow-send-agent-interaction-a2ui`
- `allow-get-active-lane`
- `allow-set-active-lane`
- `allow-lane-availability`
- `allow-has-local-engine`
- `allow-cancel-agent-run`
- `allow-prepare-local-model`
- `allow-local-model-status`
- `allow-cancel-model-operation`
- `allow-local-runtime-diagnostics`
- `allow-uar-runtime-status`
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

`gen-ui:allow-cancel-agent-run`

</td>
<td>

Enables the cancel_agent_run command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-cancel-agent-run`

</td>
<td>

Denies the cancel_agent_run command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-cancel-model-operation`

</td>
<td>

Enables the cancel_model_operation command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-cancel-model-operation`

</td>
<td>

Denies the cancel_model_operation command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-chat-approve-tool-confirmation`

</td>
<td>

Enables the chat_approve_tool_confirmation command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-chat-approve-tool-confirmation`

</td>
<td>

Denies the chat_approve_tool_confirmation command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-chat-control-catalog`

</td>
<td>

Enables the chat_control_catalog command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-chat-control-catalog`

</td>
<td>

Denies the chat_control_catalog command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-cloud-model-get`

</td>
<td>

Enables the cloud_model_get command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-cloud-model-get`

</td>
<td>

Denies the cloud_model_get command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-cloud-model-save`

</td>
<td>

Enables the cloud_model_save command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-cloud-model-save`

</td>
<td>

Denies the cloud_model_save command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-conversation-effective-config`

</td>
<td>

Enables the conversation_effective_config command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-conversation-effective-config`

</td>
<td>

Denies the conversation_effective_config command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-conversation-policy-delete`

</td>
<td>

Enables the conversation_policy_delete command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-conversation-policy-delete`

</td>
<td>

Denies the conversation_policy_delete command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-conversation-policy-get`

</td>
<td>

Enables the conversation_policy_get command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-conversation-policy-get`

</td>
<td>

Denies the conversation_policy_get command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-conversation-policy-save`

</td>
<td>

Enables the conversation_policy_save command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-conversation-policy-save`

</td>
<td>

Denies the conversation_policy_save command without any pre-configured scope.

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

`gen-ui:allow-lane-availability`

</td>
<td>

Enables the lane_availability command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-lane-availability`

</td>
<td>

Denies the lane_availability command without any pre-configured scope.

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

`gen-ui:allow-local-model-status`

</td>
<td>

Enables the local_model_status command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-local-model-status`

</td>
<td>

Denies the local_model_status command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-local-runtime-diagnostics`

</td>
<td>

Enables the local_runtime_diagnostics command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-local-runtime-diagnostics`

</td>
<td>

Denies the local_runtime_diagnostics command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-memory-delete-conversation-context`

</td>
<td>

Enables the memory_delete_conversation_context command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-memory-delete-conversation-context`

</td>
<td>

Denies the memory_delete_conversation_context command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-memory-delete-message-context`

</td>
<td>

Enables the memory_delete_message_context command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-memory-delete-message-context`

</td>
<td>

Denies the memory_delete_message_context command without any pre-configured scope.

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

`gen-ui:allow-memory-promote-context`

</td>
<td>

Enables the memory_promote_context command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-memory-promote-context`

</td>
<td>

Denies the memory_promote_context command without any pre-configured scope.

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

`gen-ui:allow-prepare-local-model`

</td>
<td>

Enables the prepare_local_model command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-prepare-local-model`

</td>
<td>

Denies the prepare_local_model command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-promote-a2ui-component`

</td>
<td>

Enables the promote_a2ui_component command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-promote-a2ui-component`

</td>
<td>

Denies the promote_a2ui_component command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-provider-catalog`

</td>
<td>

Enables the provider_catalog command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-provider-catalog`

</td>
<td>

Denies the provider_catalog command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-provider-delete`

</td>
<td>

Enables the provider_delete command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-provider-delete`

</td>
<td>

Denies the provider_delete command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-provider-list`

</td>
<td>

Enables the provider_list command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-provider-list`

</td>
<td>

Denies the provider_list command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-provider-save`

</td>
<td>

Enables the provider_save command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-provider-save`

</td>
<td>

Denies the provider_save command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-provider-test`

</td>
<td>

Enables the provider_test command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-provider-test`

</td>
<td>

Denies the provider_test command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-resolve-remote-content`

</td>
<td>

Enables the resolve_remote_content command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-resolve-remote-content`

</td>
<td>

Denies the resolve_remote_content command without any pre-configured scope.

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

`gen-ui:allow-send-agent-interaction-a2ui`

</td>
<td>

Enables the send_agent_interaction_a2ui command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-send-agent-interaction-a2ui`

</td>
<td>

Denies the send_agent_interaction_a2ui command without any pre-configured scope.

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

`gen-ui:allow-stage-attachment`

</td>
<td>

Enables the stage_attachment command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-stage-attachment`

</td>
<td>

Denies the stage_attachment command without any pre-configured scope.

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

`gen-ui:allow-uar-admin-snapshot`

</td>
<td>

Enables the uar_admin_snapshot command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-uar-admin-snapshot`

</td>
<td>

Denies the uar_admin_snapshot command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-uar-agent-delete`

</td>
<td>

Enables the uar_agent_delete command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-uar-agent-delete`

</td>
<td>

Denies the uar_agent_delete command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-uar-agent-save`

</td>
<td>

Enables the uar_agent_save command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-uar-agent-save`

</td>
<td>

Denies the uar_agent_save command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-uar-knowledge-base-create`

</td>
<td>

Enables the uar_knowledge_base_create command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-uar-knowledge-base-create`

</td>
<td>

Denies the uar_knowledge_base_create command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-uar-knowledge-base-delete`

</td>
<td>

Enables the uar_knowledge_base_delete command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-uar-knowledge-base-delete`

</td>
<td>

Denies the uar_knowledge_base_delete command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-uar-mcp-server-delete`

</td>
<td>

Enables the uar_mcp_server_delete command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-uar-mcp-server-delete`

</td>
<td>

Denies the uar_mcp_server_delete command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-uar-mcp-server-save`

</td>
<td>

Enables the uar_mcp_server_save command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-uar-mcp-server-save`

</td>
<td>

Denies the uar_mcp_server_save command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-uar-memory-delete`

</td>
<td>

Enables the uar_memory_delete command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-uar-memory-delete`

</td>
<td>

Denies the uar_memory_delete command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-uar-runtime-status`

</td>
<td>

Enables the uar_runtime_status command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-uar-runtime-status`

</td>
<td>

Denies the uar_runtime_status command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-uar-service-config`

</td>
<td>

Enables the uar_service_config command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-uar-service-config`

</td>
<td>

Denies the uar_service_config command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-uar-service-probe`

</td>
<td>

Enables the uar_service_probe command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-uar-service-probe`

</td>
<td>

Denies the uar_service_probe command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-uar-service-save`

</td>
<td>

Enables the uar_service_save command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-uar-service-save`

</td>
<td>

Denies the uar_service_save command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-uar-setting-save`

</td>
<td>

Enables the uar_setting_save command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:deny-uar-setting-save`

</td>
<td>

Denies the uar_setting_save command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`gen-ui:allow-settings-admin`

</td>
<td>

Allow the settings overlay to administer UAR connectivity and UAR-owned provider/model configuration.

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
