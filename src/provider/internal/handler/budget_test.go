package handler

// 预算 API 测试。

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/quanttide/qtcloud-finance/src/provider/internal/store"
)

func newTestServer() *httptest.Server {
	bs := store.NewBudgetStore()
	bh := NewBudgetHandler(bs)
	mux := http.NewServeMux()
	mux.HandleFunc("GET /budgets", bh.List)
	mux.HandleFunc("POST /budgets", bh.Create)
	mux.HandleFunc("GET /budgets/{id}", bh.Get)
	mux.HandleFunc("PUT /budgets/{id}", bh.Update)
	mux.HandleFunc("DELETE /budgets/{id}", bh.Delete)
	ts := httptest.NewServer(mux)
	return ts
}

func TestBudgetCRUD(t *testing.T) {
	ts := newTestServer()
	defer ts.Close()

	// Create
	resp, err := http.Post(ts.URL+"/budgets", "application/json",
		bytes.NewBufferString(`{"name":"研发部 2026Q3","category":"部门","period":"2026Q3","amount":5000000,"owner":"黎想"}`))
	if err != nil {
		t.Fatal(err)
	}
	if resp.StatusCode != http.StatusCreated {
		t.Fatalf("create: %d", resp.StatusCode)
	}
	var created struct {
		ID     string `json:"id"`
		Status string `json:"status"`
	}
	json.NewDecoder(resp.Body).Decode(&created)
	resp.Body.Close()
	if created.ID == "" || created.Status != "draft" {
		t.Fatalf("created: %+v", created)
	}

	// Get
	resp, _ = http.Get(ts.URL + "/budgets/" + created.ID)
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("get: %d", resp.StatusCode)
	}
	resp.Body.Close()

	// Update
	req, _ := http.NewRequest("PUT", ts.URL+"/budgets/"+created.ID,
		bytes.NewBufferString(`{"name":"研发部 2026Q3","category":"部门","period":"2026Q3","amount":6000000,"owner":"黎想","status":"active"}`))
	resp, err = http.DefaultClient.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("update: %d", resp.StatusCode)
	}
	resp.Body.Close()

	// 校验
	http.Get(ts.URL + "/budgets/" + created.ID)
	// Delete
	req, _ = http.NewRequest("DELETE", ts.URL+"/budgets/"+created.ID, nil)
	resp, _ = http.DefaultClient.Do(req)
	if resp.StatusCode != http.StatusNoContent {
		t.Fatalf("delete: %d", resp.StatusCode)
	}
	resp.Body.Close()

	// 删除后 404
	resp, _ = http.Get(ts.URL + "/budgets/" + created.ID)
	if resp.StatusCode != http.StatusNotFound {
		t.Fatalf("get after delete: %d", resp.StatusCode)
	}
	resp.Body.Close()
}

func TestBudgetCreateValidation(t *testing.T) {
	ts := newTestServer()
	defer ts.Close()

	// 缺 name → 400
	resp, _ := http.Post(ts.URL+"/budgets", "application/json",
		bytes.NewBufferString(`{"category":"部门","amount":100}`))
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("missing name: %d", resp.StatusCode)
	}
	resp.Body.Close()

	// 缺 amount → 400
	resp, _ = http.Post(ts.URL+"/budgets", "application/json",
		bytes.NewBufferString(`{"name":"测试","amount":0}`))
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("zero amount: %d", resp.StatusCode)
	}
	resp.Body.Close()

	// 非法 JSON → 400
	resp, _ = http.Post(ts.URL+"/budgets", "application/json", bytes.NewBufferString(`not json`))
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("bad json: %d", resp.StatusCode)
	}
	resp.Body.Close()
}
