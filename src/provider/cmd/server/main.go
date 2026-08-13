// 财务管理最小平台：预算 CRUD API。
package main

import (
	"log"
	"net/http"
	"os"

	"github.com/quanttide/qtcloud-finance/src/provider/internal/handler"
	"github.com/quanttide/qtcloud-finance/src/provider/internal/store"
)

func main() {
	bs := store.NewBudgetStore()
	bh := handler.NewBudgetHandler(bs)

	mux := http.NewServeMux()
	mux.HandleFunc("GET /budgets", bh.List)
	mux.HandleFunc("POST /budgets", bh.Create)
	mux.HandleFunc("GET /budgets/{id}", bh.Get)
	mux.HandleFunc("PUT /budgets/{id}", bh.Update)
	mux.HandleFunc("DELETE /budgets/{id}", bh.Delete)
	mux.HandleFunc("GET /healthz", func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte(`{"status":"ok"}`))
	})

	addr := ":8080"
	if a := envOr("LISTEN_ADDR", ""); a != "" {
		addr = a
	}
	log.Printf("qtadmin-finance starting on %s", addr)
	if err := http.ListenAndServe(addr, mux); err != nil {
		log.Fatalf("server error: %v", err)
	}
}

func envOr(k, d string) string {
	if v := os.Getenv(k); v != "" {
		return v
	}
	return d
}
