# AGENTS

## Escopo

Estas instrucoes valem para todo o repositorio `cloudvibe-database-operator`.

O projeto e um Kubernetes Operator para provisionar databases, usuarios, grants e secrets de aplicacoes usando CRDs do grupo `database.cloudvibe.dev`.

## Limite maximo de linhas por arquivo

Manter arquivos pequenos e focados. Quando um arquivo se aproximar do limite, dividir por responsabilidade antes de adicionar nova logica.

Limites:

- Arquivos Rust de producao: maximo de 300 linhas.
- Arquivos Rust de teste: maximo de 500 linhas.
- Arquivos YAML de manifests, samples e Helm templates: maximo de 250 linhas.
- Arquivos Markdown de documentacao: maximo de 700 linhas.
- Arquivos shell/scripts: maximo de 200 linhas.
- Arquivos gerados por ferramentas, como CRDs: isentos do limite, mas nao devem ser editados manualmente.

Regras adicionais:

- Um arquivo deve ter uma responsabilidade principal clara.
- Reconciler grande deve ser quebrado em helpers, services ou pacotes internos.
- Testes grandes devem ser divididos por comportamento ou componente.
- Evitar arquivos `utils` genericos; preferir pacotes nomeados pelo dominio.

## Padroes de design e arquitetura

### Kubernetes Operator

- Usar Rust, kube-rs e Tokio como base do projeto.
- Usar Axum para endpoints operacionais, como health, readiness e metricas.
- Instrumentar o servidor Axum com OpenTelemetry.
- A API publica do Kubernetes deve viver em `src/api/v1alpha1`.
- O reconciler deve orquestrar o fluxo, nao concentrar regra de negocio.
- O reconciler deve ser idempotente: reconciliar varias vezes o mesmo recurso nao pode recriar senha nem quebrar grants.
- O estado externo deve ser refletido em `status.conditions`, nunca em logs soltos apenas.
- Erros devem atualizar condition `Ready=False` com mensagem segura e acionavel.
- Usar finalizers com comportamento conservador; a politica inicial de delecao deve reter recursos externos.

### Separacao de responsabilidades

Separar o projeto em camadas simples:

- `src/api/v1alpha1`: tipos Kubernetes e validacoes da API.
- `src/controller`: reconcilers e integracao com Kubernetes.
- `src/database/postgres`: provisionamento PostgreSQL com sqlx.
- `src/aws/secretsmanager`: integracao com AWS Secrets Manager.
- `src/http`: servidor Axum para endpoints operacionais.
- `src/telemetry`: OpenTelemetry, tracing e metricas.
- `src/password`: geracao de senhas.
- `src/naming`: nomes de secrets e validacao de identificadores.

Regras:

- Nao colocar chamadas AWS diretamente no reconciler quando puder usar uma interface.
- Nao colocar SQL diretamente no reconciler.
- Nao misturar geracao de senha com persistencia de secret.
- Nao expor senha em `status`, eventos, logs ou erros.
- Preferir interfaces pequenas nos limites com AWS e banco de dados.

### PostgreSQL

- Validar nomes de database, schema e usuario com allowlist antes de montar SQL.
- Usar quoting seguro para identifiers.
- Usar parametros para valores sempre que o driver permitir.
- Grants devem ser reaplicaveis sem erro.
- `readonly` e `readwrite` devem ser implementados com testes de permissao reais.
- Nao rotacionar senha automaticamente quando o secret ja existir, exceto em fluxo explicito de rotacao.

### AWS Secrets Manager

- O operator deve ler o secret admin e criar secrets de aplicacao.
- Secrets criados devem seguir um padrao previsivel de nome.
- O status do recurso deve conter somente ARN/nome do secret, nunca o valor.
- O codigo deve ser testavel com fake/mock de Secrets Manager.
- Permissoes IAM documentadas devem seguir menor privilegio.

### Testes

- Toda regra de validacao deve ter teste unitario.
- Todo comportamento de reconciler deve ter teste com fake client ou teste e2e quando adequado.
- Provisionamento PostgreSQL deve ter testes contra PostgreSQL real em container ou ambiente local controlado.
- Bugs corrigidos devem receber teste de regressao quando for pratico.

### Rust

- Rodar `cargo fmt` antes de finalizar mudancas Rust.
- Rodar `cargo clippy --all-targets --all-features` quando houver mudanca de codigo.
- Preferir erros tipados com `thiserror`.
- Evitar `unwrap` e `expect` em codigo de producao, exceto em inicializacao onde a falha deve encerrar o processo claramente.
- Usar `tracing` para logs e spans; nao usar `println!` em codigo de producao.
- Manter funcoes pequenas e orientadas a uma responsabilidade.
- Evitar traits grandes; preferir interfaces pequenas nos limites com Kubernetes, AWS e banco.

### Dependencias Rust

- Usar as versoes estaveis mais recentes possiveis das crates do projeto.
- Antes de adicionar ou atualizar qualquer crate no `Cargo.toml`, pesquisar a versao estavel mais recente.
- Preferir `cargo info <crate>`, crates.io e documentacao oficial da crate para confirmar versao e feature flags.
- Nao usar versoes alpha, beta, rc, prerelease, forks ou crates abandonadas sem justificativa registrada.
- Quando houver incompatibilidade entre crates, escolher a combinacao estavel mais recente e documentar a razao no commit ou na documentacao tecnica.
- Evitar pinagem excessivamente restritiva sem necessidade. Preferir requisitos compativeis com SemVer.
- Apos alterar dependencias, rodar `cargo update` quando apropriado e validar com `cargo test`, `cargo clippy` e build.

## Regras de mudanca

- Fazer mudancas pequenas e coesas.
- Nao misturar refactor amplo com feature funcional.
- Nao reformatar arquivos nao relacionados.
- Preservar alteracoes existentes que nao foram feitas pelo agente.
- Antes de editar arquivos gerados, preferir alterar a fonte e regenerar.
- Atualizar `TASKS.md` quando uma tarefa planejada for concluida.

## Regras de commit e push

Depois de qualquer mudanca feita no projeto:

1. Executar as verificacoes relevantes para o tipo de mudanca.
2. Conferir `git status` e garantir que o commit contem apenas arquivos relacionados.
3. Criar um commit com mensagem clara e objetiva.
4. Fazer push para a branch atual.
5. Se a branch atual for `main`, acompanhar todos os workflows do GitHub Actions disparados pelo push ate a conclusao.
6. Considerar o trabalho concluido apenas quando todas as pipelines disparadas por esse push terminarem com sucesso.
7. Se alguma pipeline falhar, investigar a causa, corrigir, fazer novo commit/push quando necessario e acompanhar novamente ate tudo ficar verde.

Mensagens de commit devem seguir formato simples:

```text
<tipo>: <descricao curta>
```

Tipos recomendados:

- `docs`
- `feat`
- `fix`
- `test`
- `refactor`
- `build`
- `ci`

Exemplos:

```text
docs: add implementation plan
feat: add databaseaccess api
fix: keep existing secret password during reconcile
```
