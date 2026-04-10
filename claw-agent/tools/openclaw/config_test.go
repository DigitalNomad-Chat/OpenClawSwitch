package openclaw

import (
	"os"
	"path/filepath"
	"testing"
)

func TestReadConfig_NotFound(t *testing.T) {
	tool := NewWithPath("/nonexistent/path/openclaw.json")
	_, err := tool.ReadConfig()
	if err == nil {
		t.Fatal("expected error for nonexistent file")
	}
}

func TestNew_DefaultPath(t *testing.T) {
	tool := New()
	if tool == nil {
		t.Fatal("expected non-nil tool")
	}
	if len(tool.configPath) == 0 {
		t.Fatal("expected non-empty config path")
	}
}

func TestSplitModelID(t *testing.T) {
	tests := []struct {
		input     string
		wantProv  string
		wantModel string
	}{
		{"anthropic/claude-sonnet-4-6", "anthropic", "claude-sonnet-4-6"},
		{"openai/gpt-4o", "openai", "gpt-4o"},
		{"invalid", "", "invalid"},
	}

	for _, tt := range tests {
		prov, model := splitModelID(tt.input)
		if prov != tt.wantProv || model != tt.wantModel {
			t.Errorf("splitModelID(%q) = (%q, %q), want (%q, %q)",
				tt.input, prov, model, tt.wantProv, tt.wantModel)
		}
	}
}

func TestWriteConfig_CreatesBackup(t *testing.T) {
	dir := t.TempDir()
	configPath := filepath.Join(dir, "openclaw.json")

	tool := NewWithPath(configPath)
	err := tool.WriteConfig(&Config{
		Models: &ModelsConfig{
			Providers: map[string]ProviderConfig{
				"test": {BaseURL: "https://example.com", APIKey: "test-key"},
			},
		},
	})
	if err != nil {
		t.Fatalf("WriteConfig failed: %v", err)
	}

	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		t.Fatal("config file not created")
	}

	// Write a second time -- should create a backup in ~/.openclaw/backups
	err = tool.WriteConfig(&Config{
		Models: &ModelsConfig{
			Providers: map[string]ProviderConfig{
				"test2": {BaseURL: "https://example2.com", APIKey: "test-key2"},
			},
		},
	})
	if err != nil {
		t.Fatalf("second WriteConfig failed: %v", err)
	}

	// backupConfig uses expandPath(backupDir) which is ~/.openclaw/backups
	home, _ := os.UserHomeDir()
	backupDir := filepath.Join(home, ".openclaw", "backups")
	if _, err := os.Stat(backupDir); os.IsNotExist(err) {
		t.Fatalf("backup directory not created at %s", backupDir)
	}
}
