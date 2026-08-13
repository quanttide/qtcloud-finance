// Package store 内存存储（后续可换 SQLite/Postgres）。
package store

import (
	"sync"
	"time"

	"github.com/quanttide/qtcloud-finance/src/provider/internal/domain"
)

type BudgetStore struct {
	mu    sync.RWMutex
	data  map[string]*domain.Budget
	seq   int
}

func NewBudgetStore() *BudgetStore {
	return &BudgetStore{data: make(map[string]*domain.Budget), seq: 1}
}

func (s *BudgetStore) List() []*domain.Budget {
	s.mu.RLock()
	defer s.mu.RUnlock()
	out := make([]*domain.Budget, 0, len(s.data))
	for _, v := range s.data {
		out = append(out, v)
	}
	return out
}

func (s *BudgetStore) Get(id string) (*domain.Budget, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	v, ok := s.data[id]
	return v, ok
}

func (s *BudgetStore) Create(b *domain.Budget) *domain.Budget {
	s.mu.Lock()
	defer s.mu.Unlock()
	clone := *b
	clone.ID = "bdg-" + time.Now().Format("20060102150405") + "-" + itoa(s.seq)
	s.seq++
	clone.CreatedAt = time.Now()
	clone.UpdatedAt = time.Now()
	if clone.Status == "" {
		clone.Status = "draft"
	}
	s.data[clone.ID] = &clone
	return &clone
}

func (s *BudgetStore) Update(b *domain.Budget) (*domain.Budget, bool) {
	s.mu.Lock()
	defer s.mu.Unlock()
	existing, ok := s.data[b.ID]
	if !ok {
		return nil, false
	}
	existing.Name = b.Name
	existing.Category = b.Category
	existing.Period = b.Period
	existing.Amount = b.Amount
	existing.Owner = b.Owner
	existing.Status = b.Status
	existing.Remark = b.Remark
	existing.UpdatedAt = time.Now()
	return existing, true
}

func (s *BudgetStore) Delete(id string) bool {
	s.mu.Lock()
	defer s.mu.Unlock()
	if _, ok := s.data[id]; !ok {
		return false
	}
	delete(s.data, id)
	return true
}

func itoa(n int) string {
	if n == 0 {
		return "0"
	}
	var buf [20]byte
	i := len(buf)
	for n > 0 {
		i--
		buf[i] = byte('0' + n%10)
		n /= 10
	}
	return string(buf[i:])
}
