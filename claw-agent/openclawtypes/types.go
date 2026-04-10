package openclawtypes

// OpenClawConfig is the canonical structure for ~/.openclaw/openclaw.json.
type OpenClawConfig struct {
	Meta     *OpenClawMeta     `json:"meta,omitempty"`
	Models   *OpenClawModels   `json:"models,omitempty"`
	Agents   *OpenClawAgents   `json:"agents,omitempty"`
	Bindings []interface{}     `json:"bindings,omitempty"`
}

// OpenClawMeta 配置元数据
type OpenClawMeta struct {
	LastTouchedVersion string `json:"lastTouchedVersion,omitempty"`
	LastTouchedAt      string `json:"lastTouchedAt,omitempty"`
	ClawliteSchema     uint32 `json:"clawliteSchema,omitempty"`
}

type OpenClawModels struct {
	Providers map[string]OpenClawProvider `json:"providers,omitempty"`
}

type OpenClawProvider struct {
	BaseURL string `json:"baseUrl,omitempty"`
	APIKey  string `json:"apiKey,omitempty"`
}

type OpenClawAgents struct {
	Defaults *OpenClawDefaults `json:"defaults,omitempty"`
}

type OpenClawDefaults struct {
	Model *OpenClawModel `json:"model,omitempty"`
}

type OpenClawModel struct {
	Primary   string   `json:"primary,omitempty"`
	Fallbacks []string `json:"fallbacks,omitempty"`
}
