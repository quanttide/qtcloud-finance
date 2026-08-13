// Package handler 预算 API。
package handler

import (
	"encoding/json"
	"net/http"

	"github.com/quanttide/qtcloud-finance/src/provider/internal/domain"
	"github.com/quanttide/qtcloud-finance/src/provider/internal/store"
)

type BudgetHandler struct {
	store *store.BudgetStore
}

func NewBudgetHandler(s *store.BudgetStore) *BudgetHandler {
	return &BudgetHandler{store: s}
}

func writeJSON(w http.ResponseWriter, status int, v any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(v)
}

func (h *BudgetHandler) List(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, http.StatusOK, h.store.List())
}

func (h *BudgetHandler) Create(w http.ResponseWriter, r *http.Request) {
	var b domain.Budget
	if err := json.NewDecoder(r.Body).Decode(&b); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid request body"})
		return
	}
	if b.Name == "" || b.Amount <= 0 {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "name and amount required"})
		return
	}
	created := h.store.Create(&b)
	writeJSON(w, http.StatusCreated, created)
}

func (h *BudgetHandler) Get(w http.ResponseWriter, r *http.Request) {
	b, ok := h.store.Get(r.PathValue("id"))
	if !ok {
		writeJSON(w, http.StatusNotFound, map[string]string{"error": "not found"})
		return
	}
	writeJSON(w, http.StatusOK, b)
}

func (h *BudgetHandler) Update(w http.ResponseWriter, r *http.Request) {
	var b domain.Budget
	if err := json.NewDecoder(r.Body).Decode(&b); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid request body"})
		return
	}
	b.ID = r.PathValue("id")
	updated, ok := h.store.Update(&b)
	if !ok {
		writeJSON(w, http.StatusNotFound, map[string]string{"error": "not found"})
		return
	}
	writeJSON(w, http.StatusOK, updated)
}

func (h *BudgetHandler) Delete(w http.ResponseWriter, r *http.Request) {
	if !h.store.Delete(r.PathValue("id")) {
		writeJSON(w, http.StatusNotFound, map[string]string{"error": "not found"})
		return
	}
	w.WriteHeader(http.StatusNoContent)
}
