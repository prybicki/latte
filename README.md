# Implementation of a simple C-like language compiler in Rust

# Polish documentation:

1) Frontend: src/frontend.rs
2) Backend: src/backend.rs
    użycie rejestrów i phi zamiast alloc: TAK
5) Optymalizacje:
    Na frontendzie jest constant-folding (bez zaglądania do zmiennych), które jest tam używane do sprawdzenia return-ów oraz na backendzie do ominięcia trywialnych if-ów, typu if (5 > 3 || false) {...}.
    Dodatkowo użycie LLVM API do generacji kodu w postaci SSA de facto implikuje upraszczanie lokalnych wyrażeń, które dają się policzyć w czasie kompilacji (constant folding z zaglądaniem do zmiennych).

6) Rozszerzenia:
    Jak narazie żadne.

Używane biblioteki:

    Generacja parsera:
        lalrpop = "0.17.2"

    Runtime parsera:
        lalrpop-util = "0.17.2"

    Wyświetlanie komunikatów diagnostycznych:
        codespan-reporting = "0.6.0"
        codespan = "0.6.0"
        termcolor = "1.0.5"

    Wrapper LLVM API:
        inkwell = { git = "https://github.com/TheDan64/inkwell", branch = "llvm6-0" }


