package store

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sync"
)

const indexFileName = "_index.jsonl"

// indexCache caches session metadata in memory, backed by _index.jsonl.
type indexCache struct {
	path    string
	entries map[string]SessionInfo // keyed by session ID, last write wins
	mu      sync.RWMutex
	loaded  bool
}

func newIndexCache(dir string) *indexCache {
	return &indexCache{
		path:    filepath.Join(dir, indexFileName),
		entries: make(map[string]SessionInfo),
	}
}

// load reads the entire _index.jsonl into memory. Idempotent — only reads once.
func (c *indexCache) load() error {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.loaded {
		return nil
	}
	return c.loadLocked()
}

// loadLocked reads the file. Caller must hold c.mu write lock.
func (c *indexCache) loadLocked() error {
	f, err := os.Open(c.path)
	if err != nil {
		if os.IsNotExist(err) {
			c.loaded = true
			return nil
		}
		return fmt.Errorf("open index: %w", err)
	}
	defer func() { _ = f.Close() }()

	scanner := bufio.NewScanner(f)
	scanner.Buffer(make([]byte, 0, 4096), 64*1024)
	for scanner.Scan() {
		line := scanner.Bytes()
		if len(line) == 0 {
			continue
		}
		var info SessionInfo
		if err := json.Unmarshal(line, &info); err != nil {
			continue // skip malformed lines
		}
		if info.ID != "" {
			// Tombstone entries (sentinel title) remove the key from cache.
			if info.Title == "\x00_tombstone" {
				delete(c.entries, info.ID)
				continue
			}
			c.entries[info.ID] = info
		}
	}
	if err := scanner.Err(); err != nil {
		return fmt.Errorf("read index: %w", err)
	}
	c.loaded = true
	return nil
}

// save appends a SessionInfo to the file and updates the cache.
func (c *indexCache) save(info SessionInfo) error {
	if err := c.load(); err != nil {
		return err
	}

	data, err := json.Marshal(info)
	if err != nil {
		return fmt.Errorf("marshal session info: %w", err)
	}

	c.mu.Lock()
	defer c.mu.Unlock()

	f, err := os.OpenFile(c.path, os.O_WRONLY|os.O_APPEND|os.O_CREATE, 0o644)
	if err != nil {
		return fmt.Errorf("open index for append: %w", err)
	}
	defer func() { _ = f.Close() }()

	if _, err := f.Write(append(data, '\n')); err != nil {
		return fmt.Errorf("write index entry: %w", err)
	}

	c.entries[info.ID] = info
	return nil
}

// get returns the metadata for a single session.
func (c *indexCache) get(sessionID string) (SessionInfo, bool) {
	if err := c.load(); err != nil {
		return SessionInfo{}, false
	}
	c.mu.RLock()
	defer c.mu.RUnlock()
	info, ok := c.entries[sessionID]
	return info, ok
}

// remove deletes a session from the in-memory cache and appends a tombstone
// to the index file so the entry is effectively removed on next load.
func (c *indexCache) remove(sessionID string) error {
	if err := c.load(); err != nil {
		return err
	}

	c.mu.Lock()
	defer c.mu.Unlock()

	delete(c.entries, sessionID)

	// Write a tombstone entry so that previous lines for this ID are ignored.
	tombstone := SessionInfo{ID: sessionID, Archived: true, Title: "\x00_tombstone"}
	data, err := json.Marshal(tombstone)
	if err != nil {
		return fmt.Errorf("marshal tombstone: %w", err)
	}

	f, err := os.OpenFile(c.path, os.O_WRONLY|os.O_APPEND|os.O_CREATE, 0o644)
	if err != nil {
		return fmt.Errorf("open index for append: %w", err)
	}
	defer func() { _ = f.Close() }()

	if _, err := f.Write(append(data, '\n')); err != nil {
		return fmt.Errorf("write tombstone: %w", err)
	}
	return nil
}

// list returns all session metadata, optionally filtering out archived sessions.
func (c *indexCache) list(includeArchived bool) []SessionInfo {
	if err := c.load(); err != nil {
		return nil
	}
	c.mu.RLock()
	defer c.mu.RUnlock()

	result := make([]SessionInfo, 0, len(c.entries))
	for _, info := range c.entries {
		if !includeArchived && info.Archived {
			continue
		}
		result = append(result, info)
	}
	return result
}

// FileStore methods that delegate to indexCache.

func (s *FileStore) SaveInfo(info SessionInfo) error {
	return s.index.save(info)
}

func (s *FileStore) LoadInfo(sessionID string) (SessionInfo, error) {
	info, ok := s.index.get(sessionID)
	if !ok {
		return SessionInfo{}, fmt.Errorf("session %q not found in index", sessionID)
	}
	return info, nil
}

func (s *FileStore) ListInfo(includeArchived bool) ([]SessionInfo, error) {
	return s.index.list(includeArchived), nil
}
