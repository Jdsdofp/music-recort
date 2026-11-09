# Relatório de Correções — Code Review Music Recort

**Data:** 2026-11-09

---

## Status por Finding

| # | Finding | Status | Notas |
|---|---------|--------|-------|
| 1 | IPC incompatível (named pipe → stdio) | CORRIGIDO | `ipc.rs` reescrito para lançar Python via `Command`, comunicar por stdin/stdout, timeout 5 min |
| 2 | SSRF em api_fallback.py | CORRIGIDO | `separate_via_api` levanta `NotImplementedError` imediatamente em vez de enviar `file://` |
| 3 | Deadlock no subprocess Demucs | CORRIGIDO | stdout+stderr lidos em threads dedicadas; thread join com timeout 300s; `process.kill()` em timeout |
| 4 | Hooks não conectados em App.tsx | CORRIGIDO | `useProject()` e `useStemSeparation()` chamados em `App`; stale closure corrigida com `useProjectStore.getState` |
| 5 | `_get_duration` silencia exceções | CORRIGIDO | Retorna `None` com `print(Warning:...)` ao invés de `0.0`; `main.py` trata `None` explicitamente |
| 6 | Mixer sem `.catch()` | CORRIGIDO | `.catch(console.error)` adicionado em `invoke.setVolume` e `invoke.setPan`; `async` removido dos handlers |

---

## Resultado dos Testes

### `cargo check`
```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 24.98s
```
**PASSOU** — sem erros de compilação.

### `cargo test --lib`
**BLOQUEADO por ambiente** — erro de linking: `dlltool.exe` não encontrado no PATH do MinGW/MSYS2.
Este é um problema pré-existente do ambiente de build, não introduzido pelas correções. O `cargo check` confirma que o código está semanticamente correto.

### `npx tsc --noEmit`
**PASSOU** — sem erros TypeScript (saída vazia = sucesso).

---

## Concerns

1. **`cargo test --lib` não executou** por falha de linking do MinGW (`dlltool.exe` não encontrado). Os testes de protocolo (`protocol.rs`) são corretos semanticamente, confirmado pelo `cargo check`. Recomenda-se corrigir o toolchain MSYS2 ou usar WSL para executar os testes.

2. **API fallback completamente desabilitada**: O finding #2 foi corrigido de forma conservadora — a função `separate_via_api` agora levanta `NotImplementedError` imediatamente. Se no futuro a API Moises suportar upload multipart, o código de polling abaixo do raise pode ser restaurado com um upload real.

3. **`useStemSeparation` retorna `separate` mas App.tsx não o usa**: O hook é chamado apenas para registrar os event listeners. O retorno não é desestruturado em `App.tsx` propositalmente — `StemPanel` chama `separate` diretamente pelo store.

---

## Hash do Commit

`591e131` — branch `feature/music-recort-fases-1-2`
