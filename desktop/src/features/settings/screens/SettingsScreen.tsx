// TJ-ARCH-MOB-001 compliant
import { useMemo, useState } from 'react'
import { KeyRound, Trash2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { useProviderSettings } from '../hooks/useProviderSettings'
import { useSettings } from '../hooks/useSettings'

const fieldClass = 'h-11 border-0 bg-[color:var(--color-bg)] ring-0 focus-visible:border-0 focus-visible:ring-2'

export function SettingsScreen() {
  const { settings, toggle, devices } = useSettings()
  const providerState = useProviderSettings()
  const [kind, setKind] = useState('')
  const [configurationName, setConfigurationName] = useState('')
  const [activeProviderId, setActiveProviderId] = useState('')
  const [baseUrl, setBaseUrl] = useState('')
  const [apiKey, setApiKey] = useState('')
  const [modelId, setModelId] = useState('')

  const selectedCatalog = useMemo(
    () => providerState.catalog.find((entry) => entry.id === kind),
    [kind, providerState.catalog],
  )
  const enabledProviders = providerState.providers.filter((provider) => provider.enabled)
  const rows = [
    ['Sync conversations to my server', 'Keeps every message on infrastructure you control.', 'sync'],
    ['Allow cloud provider fallback', 'Used only when you explicitly choose a configured cloud model.', 'cloud'],
    ['Share anonymous diagnostics', 'Crash and performance signals only—never conversation content.', 'diagnostics'],
  ] as const

  async function addProvider() {
    const id = configurationName.trim() || kind
    const saved = await providerState.saveProvider({
      id,
      kind,
      baseUrl: baseUrl.trim() || selectedCatalog?.defaultBaseUrl || null,
      apiKey: apiKey.trim() || null,
      enabled: true,
    })
    if (saved) {
      setApiKey('')
      setConfigurationName('')
      setBaseUrl('')
    }
  }

  return (
    <div className="h-full overflow-y-auto px-6 py-10 sm:px-12 lg:px-16">
      <div className="mx-auto max-w-5xl">
        <h1 className="text-4xl font-bold">Settings</h1>

        <section className="mt-9">
          <h2 className="text-lg font-bold">Cloud models · BYOK</h2>
          <p className="mt-1 text-sm text-[color:var(--color-fg-sub)]">
            Use any chat provider compiled into Liter-LLM. {providerState.desktopAvailable
              ? 'Keys stay in your operating system keychain.'
              : 'Keys stay in this browser tab and are sent only with the chat request.'}
          </p>
          {!providerState.desktopAvailable && (
            <div className="mt-3 rounded-xl bg-[color:var(--color-surface)] p-6 text-sm text-[color:var(--color-fg-sub)]">
              Local WebLLM remains the zero-configuration default. Hosted BYOK is optional and session-scoped; closing the tab clears it.
            </div>
          )}
          <div className="mt-3 space-y-3">
              <div className="grid gap-4 rounded-xl bg-[color:var(--color-surface)] p-6 md:grid-cols-2">
                <div className="space-y-2">
                  <Label>Provider</Label>
                  <Select value={kind} onValueChange={(value) => setKind(value ?? '')}>
                    <SelectTrigger className={`${fieldClass} w-full`}>
                      <SelectValue placeholder="Choose a Liter-LLM provider" />
                    </SelectTrigger>
                    <SelectContent className="border-0 ring-0">
                      {providerState.catalog.map((provider) => (
                        <SelectItem key={provider.id} value={provider.id}>{provider.displayName}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label htmlFor="provider-id">Configuration name</Label>
                  <Input id="provider-id" className={fieldClass} value={configurationName} onChange={(event) => setConfigurationName(event.target.value)} placeholder={kind || 'personal-openai'} />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="provider-url">Base URL · optional</Label>
                  <Input id="provider-url" className={fieldClass} value={baseUrl} onChange={(event) => setBaseUrl(event.target.value)} placeholder={selectedCatalog?.defaultBaseUrl ?? 'Provider default'} />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="provider-key"><KeyRound className="size-4" /> API key</Label>
                  <Input id="provider-key" type="password" autoComplete="off" className={fieldClass} value={apiKey} onChange={(event) => setApiKey(event.target.value)} placeholder={providerState.desktopAvailable ? 'Stored in OS keychain' : 'Cleared when this tab closes'} />
                </div>
                <Button className="h-11 border-0 md:col-span-2" disabled={!kind || providerState.saving} onClick={() => void addProvider()}>
                  Save provider
                </Button>
              </div>

              {providerState.providers.map((provider) => (
                <div key={provider.id} className="flex items-center justify-between gap-4 rounded-xl bg-[color:var(--color-surface)] p-5">
                  <div>
                    <p className="font-bold">{provider.id}</p>
                    <p className="text-sm text-[color:var(--color-fg-sub)]">{provider.kind} · {provider.hasApiKey ? 'Key secured' : 'No key stored'}</p>
                  </div>
                  <Button variant="ghost" size="icon" aria-label={`Delete ${provider.id}`} onClick={() => void providerState.removeProvider(provider.id)}>
                    <Trash2 />
                  </Button>
                </div>
              ))}

              {enabledProviders.length > 0 && (
                <div className="grid gap-4 rounded-xl bg-[color:var(--color-surface)] p-6 md:grid-cols-[1fr_2fr_auto] md:items-end">
                  <div className="space-y-2">
                    <Label>Active provider</Label>
                    <Select value={activeProviderId || providerState.cloudModel?.providerId || ''} onValueChange={(value) => setActiveProviderId(value ?? '')}>
                      <SelectTrigger className={`${fieldClass} w-full`}><SelectValue placeholder="Provider" /></SelectTrigger>
                      <SelectContent className="border-0 ring-0">
                        {enabledProviders.map((provider) => <SelectItem key={provider.id} value={provider.id}>{provider.id}</SelectItem>)}
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="cloud-model">Model identifier</Label>
                    <Input id="cloud-model" className={fieldClass} value={modelId} onChange={(event) => setModelId(event.target.value)} placeholder={providerState.cloudModel?.modelId ?? 'gpt-4.1-mini'} />
                  </div>
                  <Button className="h-11 border-0" disabled={providerState.saving || !(activeProviderId || providerState.cloudModel?.providerId) || !modelId.trim()} onClick={() => void providerState.saveCloudModel(activeProviderId || providerState.cloudModel?.providerId || '', modelId)}>
                    Use model
                  </Button>
                </div>
              )}
              {providerState.error && <p role="alert" className="rounded-xl bg-destructive/15 p-4 text-sm text-destructive">{providerState.error}</p>}
          </div>
        </section>

        <section className="mt-9">
          <h2 className="text-lg font-bold">Appearance</h2>
          <div className="mt-3 flex items-center justify-between rounded-xl bg-[color:var(--color-surface)] p-6">
            <span>Dark theme</span><Switch checked={settings.dark} onCheckedChange={() => toggle('dark')} aria-label="Dark theme" />
          </div>
        </section>
        <section className="mt-9">
          <h2 className="text-lg font-bold">Devices &amp; sync</h2>
          <div className="mt-3 space-y-3">
            {devices.map(([name, role, state]) => (
              <div key={name} className="flex items-center justify-between rounded-xl bg-[color:var(--color-surface)] p-6">
                <div><p className="font-bold">{name}</p><p className="text-sm text-[color:var(--color-fg-sub)]">{role}</p></div>
                <span className="font-mono text-xs text-[color:var(--color-fg-faint)]">{state}</span>
              </div>
            ))}
          </div>
        </section>
        <section className="mt-9">
          <h2 className="text-lg font-bold">Privacy &amp; data</h2>
          <div className="mt-3 space-y-3">
            {rows.map(([label, caption, key]) => (
              <div key={key} className="flex items-center justify-between gap-8 rounded-xl bg-[color:var(--color-surface)] p-6">
                <div><p className="font-bold">{label}</p><p className="mt-1 text-sm text-[color:var(--color-fg-sub)]">{caption}</p></div>
                <Switch checked={settings[key]} onCheckedChange={() => toggle(key)} aria-label={label} />
              </div>
            ))}
          </div>
        </section>
      </div>
    </div>
  )
}
