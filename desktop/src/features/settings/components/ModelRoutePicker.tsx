// TJ-ARCH-MOB-001 compliant — a pure picker over the provider store's catalog.
// Renders a provider → model cascade that writes a { provider_id, model_id }
// ModelRoute (or null to inherit the lower scope). Used at every model-selection
// tier: global run policy, per-agent policy, and per-conversation policy.
import { useEffect, useMemo } from 'react'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useProviderStore } from '../stores/providerStore'

/** A resolved model route, or null when the tier inherits the scope below it. */
export interface ModelRoute {
  provider_id: string
  model_id: string
}

interface ModelRoutePickerProps {
  /** Current route, or null/undefined to mean "inherit". */
  value: ModelRoute | null
  /** Emit the new route, or null when the user clears the selection. */
  onChange: (route: ModelRoute | null) => void
  /** Label for the "no override" option, e.g. "Runtime default" or "Inherit". */
  inheritLabel?: string
  /** Disable the control (e.g. while saving). */
  disabled?: boolean
}

const INHERIT = '__inherit__'
const triggerClass =
  'h-11 w-full border-0 bg-[color:var(--color-surface)] shadow-none focus-visible:ring-2'

/**
 * Provider → model cascade. Providers come from the configured-provider catalog
 * (those with credentials, plus the on-device provider). Selecting a provider
 * narrows the model list; selecting the inherit option clears the route.
 */
export function ModelRoutePicker({
  value,
  onChange,
  inheritLabel = 'Runtime default',
  disabled = false,
}: ModelRoutePickerProps) {
  const catalog = useProviderStore((state) => state.catalog)
  const load = useProviderStore((state) => state.load)
  const loading = useProviderStore((state) => state.loading)

  // The catalog is shared across settings surfaces; load it lazily the first
  // time a picker mounts so per-conversation/agent pickers work without opening
  // the providers panel first.
  useEffect(() => {
    if (!catalog.length && !loading) void load()
  }, [catalog.length, loading, load])

  // Only providers that are actually usable (configured with a key, or the
  // on-device provider which needs none) can be selected as a route.
  const providers = useMemo(
    () =>
      catalog.filter(
        (provider) => provider.configured || provider.id === 'on-device',
      ),
    [catalog],
  )

  const selectedProvider = providers.find(
    (provider) => provider.id === value?.provider_id,
  )
  const models = selectedProvider?.models ?? []

  const onProvider = (providerId: string | null) => {
    if (!providerId || providerId === INHERIT) {
      onChange(null)
      return
    }
    const provider = providers.find((entry) => entry.id === providerId)
    const firstModel =
      provider?.models.find((model) => model.enabled)?.id ??
      provider?.models[0]?.id ??
      ''
    onChange({ provider_id: providerId, model_id: firstModel })
  }

  const onModel = (modelId: string | null) => {
    if (!value || !modelId) return
    onChange({ ...value, model_id: modelId })
  }

  const providerLabel = (id: string) =>
    id === 'on-device'
      ? 'On-device'
      : providers.find((provider) => provider.id === id)?.displayName || id

  return (
    <div className="grid gap-3 sm:grid-cols-2">
      <label className="grid gap-2 text-sm font-semibold">
        Provider
        <Select
          value={value?.provider_id ?? INHERIT}
          onValueChange={onProvider}
          disabled={disabled}
        >
          <SelectTrigger className={triggerClass} aria-label="Model provider">
            <SelectValue placeholder={inheritLabel} />
          </SelectTrigger>
          <SelectContent className="border-0 shadow-none ring-0">
            <SelectItem value={INHERIT}>{inheritLabel}</SelectItem>
            {providers.map((provider) => (
              <SelectItem key={provider.id} value={provider.id}>
                {providerLabel(provider.id)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </label>
      <label className="grid gap-2 text-sm font-semibold">
        Model
        <Select
          value={value?.model_id ?? ''}
          onValueChange={onModel}
          disabled={disabled || !value || !models.length}
        >
          <SelectTrigger className={triggerClass} aria-label="Model">
            <SelectValue
              placeholder={value ? 'Choose a model' : 'Select a provider first'}
            />
          </SelectTrigger>
          <SelectContent className="border-0 shadow-none ring-0">
            {models.map((model) => (
              <SelectItem key={model.id} value={model.id}>
                {model.displayName || model.id}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </label>
    </div>
  )
}
