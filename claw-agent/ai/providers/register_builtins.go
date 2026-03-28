package providers

import (
	"claw-agent/ai/providers/anthropic"
	"claw-agent/ai/providers/openai"
	openairesponse "claw-agent/ai/providers/openai-response"
	"claw-agent/ai/registry"
)

// RegisterBuiltins registers first-party providers in a registry.
func RegisterBuiltins(r *registry.Registry) {
	r.Register(openai.New(openai.Config{}))
	r.Register(openairesponse.New(openairesponse.Config{}))
	r.Register(anthropic.New(anthropic.Config{}))
}
