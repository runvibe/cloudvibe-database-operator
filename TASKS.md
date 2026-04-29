# CloudVibe Database Operator - Tarefas

Este arquivo divide a implementacao do CloudVibe Database Operator em tarefas pequenas e verificaveis.

Legenda:

- `[ ]` pendente
- `[x]` concluido
- `MVP` necessario para primeira versao utilizavel
- `Later` pode ficar para depois do MVP

## 0. Fundacao do Repositorio

- [x] `MVP` Inicializar projeto Rust com Cargo.
- [x] `MVP` Definir package name e metadata do crate.
- [x] `MVP` Adicionar dependencias base: `tokio`, `kube`, `k8s-openapi`, `serde`, `schemars`, `thiserror`, `tracing`, `axum`, `sqlx` e AWS SDK.
- [x] `MVP` Pesquisar e registrar as versoes estaveis mais recentes das crates antes de adiciona-las ao `Cargo.toml`.
- [x] `MVP` Gerar estrutura base com `src/api`, `src/controller`, `src/database`, `src/aws`, `src/http` e `src/telemetry`.
- [x] `MVP` Configurar comandos de desenvolvimento via `Makefile` ou `justfile`.
- [x] `MVP` Configurar `.gitignore` para binarios, builds e artefatos temporarios.
- [x] `MVP` Criar `Dockerfile` do manager.
- [x] `MVP` Criar README inicial com objetivo do projeto.
- [x] `MVP` Garantir que `cargo test` roda sem falhas no projeto vazio.
- [x] `MVP` Garantir que `cargo clippy` roda sem warnings relevantes.
- [x] `MVP` Garantir que CRDs sao exportados a partir dos tipos Rust.

## 1. API Kubernetes v1alpha1

### 1.1 DatabaseInstance

- [x] `MVP` Criar tipo `DatabaseInstance`.
- [x] `MVP` Criar `DatabaseInstanceSpec`.
- [x] `MVP` Adicionar campo `engine`.
- [x] `MVP` Adicionar campo `region`.
- [x] `MVP` Adicionar campo `host`.
- [x] `MVP` Adicionar campo `port`.
- [x] `MVP` Adicionar campo `adminSecretArn`.
- [x] `MVP` Adicionar campo `secretPrefix`.
- [x] `MVP` Adicionar campo `allowedNamespaces`.
- [x] `MVP` Criar `DatabaseInstanceStatus` minimo.
- [ ] `MVP` Adicionar validacoes OpenAPI para campos obrigatorios.
- [x] `MVP` Restringir `engine` inicialmente para `aurora-postgres`.
- [x] `MVP` Gerar sample `config/samples/database_v1alpha1_databaseinstance.yaml`.

### 1.2 DatabaseAccess

- [x] `MVP` Criar tipo `DatabaseAccess`.
- [x] `MVP` Criar `DatabaseAccessSpec`.
- [x] `MVP` Adicionar `instanceRef.name`.
- [x] `MVP` Adicionar campo `database`.
- [x] `MVP` Adicionar campo `schemas`.
- [x] `MVP` Adicionar lista `users`.
- [x] `MVP` Adicionar `users[].name`.
- [x] `MVP` Adicionar `users[].permissions`.
- [x] `MVP` Adicionar `users[].secretName` como opcional.
- [x] `MVP` Restringir `permissions` para `readonly` e `readwrite`.
- [x] `MVP` Criar `DatabaseAccessStatus`.
- [x] `MVP` Adicionar `status.phase`.
- [x] `MVP` Adicionar `status.observedGeneration`.
- [x] `MVP` Adicionar `status.users[].secretArn`.
- [x] `MVP` Adicionar `status.conditions`.
- [x] `MVP` Adicionar colunas para `kubectl get databaseaccess`.
- [x] `MVP` Gerar sample `config/samples/database_v1alpha1_databaseaccess.yaml`.

### 1.3 Validacao da API

- [ ] `MVP` Validar nomes de database com regex segura.
- [ ] `MVP` Validar nomes de schema com regex segura.
- [ ] `MVP` Validar nomes de usuario com regex segura.
- [ ] `MVP` Definir default de `schemas` como `public`.
- [ ] `Later` Implementar webhook de validacao para regras que OpenAPI nao cobre bem.

## 2. Controller Skeleton

- [x] `MVP` Criar reconciler para `DatabaseAccess`.
- [x] `MVP` Registrar reconciler no manager.
- [x] `MVP` Configurar RBAC para ler `DatabaseInstance`.
- [x] `MVP` Configurar RBAC para ler e atualizar `DatabaseAccess`.
- [x] `MVP` Configurar RBAC para atualizar `DatabaseAccess/status`.
- [ ] `MVP` Configurar eventos Kubernetes.
- [x] `MVP` Implementar lookup de `DatabaseInstance` por `instanceRef.name`.
- [x] `MVP` Implementar validacao de `allowedNamespaces`.
- [x] `MVP` Atualizar condition `Ready=False` quando `DatabaseInstance` nao existir.
- [x] `MVP` Atualizar condition `Ready=False` quando namespace nao for permitido.
- [x] `MVP` Atualizar `observedGeneration` apos reconciliacao.
- [ ] `MVP` Criar testes de reconciler com fake client para estados basicos.

## 3. Condicoes, Eventos e Status

- [ ] `MVP` Definir constantes de conditions.
- [ ] `MVP` Definir phases iniciais: `Pending`, `Ready`, `Error`.
- [ ] `MVP` Criar helper para setar `Ready=True`.
- [ ] `MVP` Criar helper para setar `Ready=False`.
- [x] `MVP` Garantir que mensagens de erro nao vazam senha.
- [ ] `MVP` Emitir evento quando provisionamento comeca.
- [ ] `MVP` Emitir evento quando provisionamento termina com sucesso.
- [ ] `MVP` Emitir evento quando provisionamento falha.

## 4. AWS Secrets Manager

### 4.1 Contratos Internos

- [ ] `MVP` Criar interface `SecretStore`.
- [ ] `MVP` Criar struct para admin secret.
- [ ] `MVP` Criar struct para app secret.
- [ ] `MVP` Definir serializacao JSON dos secrets.
- [ ] `MVP` Criar testes unitarios de serializacao.

### 4.2 Implementacao AWS

- [ ] `MVP` Adicionar AWS SDK for Rust.
- [x] `MVP` Criar client de Secrets Manager.
- [x] `MVP` Implementar `GetSecretValue` para admin secret.
- [ ] `MVP` Implementar `DescribeSecret`.
- [x] `MVP` Implementar `CreateSecret`.
- [x] `MVP` Implementar `PutSecretValue`.
- [ ] `MVP` Implementar tags padrao nos secrets criados.
- [x] `MVP` Tratar secret existente de forma idempotente.
- [ ] `MVP` Criar mocks/fakes para testes.

### 4.3 Nomes de Secrets

- [x] `MVP` Implementar gerador de nome padrao `{secretPrefix}/{namespace}/{database}/{username}`.
- [x] `MVP` Permitir override via `users[].secretName`.
- [x] `MVP` Validar que o secret gerado nao sai do prefixo permitido.
- [ ] `Later` Permitir template de nomes por `DatabaseInstance`.

## 5. Geracao de Senhas

- [x] `MVP` Criar pacote `internal/password`.
- [x] `MVP` Gerar senha criptograficamente segura.
- [x] `MVP` Definir tamanho minimo da senha.
- [x] `MVP` Evitar caracteres problematicos para URL/connection string quando necessario.
- [x] `MVP` Criar testes para tamanho e variedade.
- [x] `MVP` Garantir que senha nunca aparece em logs.

## 6. Aurora PostgreSQL Provisioner

### 6.1 Contratos

- [ ] `MVP` Criar interface `DatabaseProvisioner`.
- [ ] `MVP` Criar modelo de entrada com instance, database, schemas e users.
- [ ] `MVP` Criar modelo de resultado por usuario.
- [ ] `MVP` Definir erros tipados para conflito, credencial invalida e permissao insuficiente.

### 6.2 Conexao

- [ ] `MVP` Adicionar `sqlx` com suporte a PostgreSQL e rustls.
- [x] `MVP` Implementar conexao no database administrativo.
- [x] `MVP` Implementar conexao no database alvo.
- [x] `MVP` Configurar timeout de conexao.
- [x] `MVP` Configurar TLS/SSL mode adequado para Aurora PostgreSQL.
- [ ] `Later` Suportar CA bundle customizado.

### 6.3 SQL Seguro

- [x] `MVP` Implementar quote seguro de identifiers.
- [x] `MVP` Validar identifiers antes de montar SQL.
- [x] `MVP` Usar parametros para valores sempre que possivel.
- [x] `MVP` Criar testes unitarios para quoting.

### 6.4 Database e Schemas

- [x] `MVP` Verificar se database existe.
- [x] `MVP` Criar database quando nao existir.
- [x] `MVP` Reconectar no database alvo apos criacao.
- [x] `MVP` Verificar se schema existe.
- [x] `MVP` Criar schema quando nao existir.

### 6.5 Usuarios

- [x] `MVP` Verificar se usuario existe.
- [x] `MVP` Criar usuario com senha gerada quando nao existir.
- [x] `MVP` Atualizar senha quando fluxo exigir criacao de novo secret.
- [x] `MVP` Nao rotacionar senha automaticamente se secret ja existe.
- [ ] `MVP` Marcar erro claro quando usuario existe mas secret esperado nao existe.

### 6.6 Grants Readonly

- [x] `MVP` Aplicar `GRANT CONNECT`.
- [x] `MVP` Aplicar `GRANT USAGE ON SCHEMA`.
- [x] `MVP` Aplicar `GRANT SELECT ON ALL TABLES`.
- [x] `MVP` Aplicar default privileges para tabelas futuras.
- [x] `MVP` Criar teste comprovando que readonly consegue ler.
- [x] `MVP` Criar teste comprovando que readonly nao consegue escrever.

### 6.7 Grants Readwrite

- [x] `MVP` Aplicar `GRANT CONNECT`.
- [x] `MVP` Aplicar `GRANT USAGE, CREATE ON SCHEMA`.
- [x] `MVP` Aplicar grants de tabelas.
- [x] `MVP` Aplicar grants de sequencias.
- [x] `MVP` Aplicar default privileges para tabelas futuras.
- [x] `MVP` Aplicar default privileges para sequencias futuras.
- [x] `MVP` Criar teste comprovando que readwrite consegue ler e escrever.

## 7. Integracao do Reconciler

- [x] `MVP` Conectar reconciler ao `SecretStore`.
- [x] `MVP` Conectar reconciler ao `DatabaseProvisioner`.
- [x] `MVP` Ler admin secret durante reconciliacao.
- [x] `MVP` Gerar senha apenas quando necessario.
- [x] `MVP` Criar secret da aplicacao apos provisionar usuario.
- [x] `MVP` Atualizar `status.users` com `secretArn`.
- [x] `MVP` Reaplicar grants em reconciliacoes repetidas.
- [ ] `MVP` Reconciliar com backoff quando AWS falhar.
- [ ] `MVP` Reconciliar com backoff quando banco falhar.
- [ ] `MVP` Criar teste de caminho feliz com fakes.
- [ ] `MVP` Criar teste de erro de permissao de namespace.
- [ ] `MVP` Criar teste de erro de secret admin ausente.

## 8. Finalizers e Delecao

- [ ] `MVP` Adicionar finalizer em `DatabaseAccess`.
- [ ] `MVP` Implementar comportamento inicial `Retain`.
- [ ] `MVP` Ao deletar, nao remover database.
- [ ] `MVP` Ao deletar, nao remover usuarios.
- [ ] `MVP` Emitir evento informando que recursos externos foram mantidos.
- [ ] `MVP` Remover finalizer apos registrar retencao.
- [ ] `Later` Adicionar `spec.deletionPolicy`.
- [ ] `Later` Implementar `RevokeUsers`.
- [ ] `Later` Implementar `DropDatabase` apenas com protecoes explicitas.

## 9. Observabilidade

- [ ] `MVP` Usar logs estruturados com `tracing`.
- [ ] `MVP` Criar modulo `src/telemetry`.
- [ ] `MVP` Configurar OpenTelemetry com export OTLP.
- [ ] `MVP` Configurar propagacao de contexto.
- [ ] `MVP` Instrumentar servidor Axum com spans por request.
- [ ] `MVP` Instrumentar reconciliacao de `DatabaseAccess`.
- [ ] `MVP` Instrumentar chamadas ao AWS Secrets Manager.
- [ ] `MVP` Instrumentar operacoes PostgreSQL relevantes.
- [ ] `MVP` Logar namespace/name do recurso reconciliado.
- [ ] `MVP` Logar instanceRef e database sem senha.
- [ ] `MVP` Expor endpoint HTTP `GET /healthz` com Axum.
- [ ] `MVP` Expor endpoint HTTP `GET /readyz` com Axum.
- [ ] `MVP` Expor endpoint HTTP `GET /metrics` quando o exporter escolhido suportar scrape.
- [ ] `MVP` Criar metricas para reconciliacoes com sucesso.
- [ ] `MVP` Criar metricas para reconciliacoes com erro.
- [ ] `Later` Criar metricas por tipo de erro.
- [ ] `Later` Criar dashboard Grafana.

## 10. Empacotamento e Deploy

### 10.1 Imagem

- [x] `MVP` Criar build multi-stage no Dockerfile.
- [x] `MVP` Rodar manager como usuario nao-root.
- [ ] `MVP` Publicar imagem com tag versionada.
- [ ] `MVP` Documentar variaveis de ambiente do manager.

### 10.2 Manifests

- [ ] `MVP` Gerar manifests `config/default`.
- [x] `MVP` Validar RBAC gerado.
- [ ] `MVP` Validar leader election.
- [ ] `MVP` Validar namespace de instalacao.

### 10.3 Helm Chart

- [x] `MVP` Criar chart `charts/cloudvibe-database-operator`.
- [x] `MVP` Templatear deployment do manager.
- [x] `MVP` Templatear service account.
- [x] `MVP` Templatear RBAC.
- [ ] `MVP` Templatear CRDs ou documentar instalacao separada.
- [x] `MVP` Suportar annotation de IRSA/EKS Pod Identity no service account.
- [x] `MVP` Criar `values.yaml` padrao.
- [x] `MVP` Rodar `helm lint`.

## 11. AWS/IAM

- [ ] `MVP` Documentar IAM minimo do operator.
- [ ] `MVP` Criar exemplo de policy para Secrets Manager.
- [ ] `MVP` Criar exemplo de trust policy para EKS IRSA.
- [ ] `MVP` Criar exemplo de ServiceAccount anotado.
- [ ] `Later` Criar modulo Terraform para IAM.
- [ ] `Later` Criar exemplo para EKS Pod Identity.

## 12. External Secrets Operator

- [ ] `MVP` Documentar como consumir o secret criado via External Secrets Operator.
- [ ] `MVP` Criar exemplo de `ClusterSecretStore`.
- [ ] `MVP` Criar exemplo de `ExternalSecret`.
- [ ] `MVP` Criar exemplo de Deployment consumindo env vars.
- [ ] `Later` Adicionar opcao para o operator criar `ExternalSecret` automaticamente.

## 13. Testes

### 13.1 Unitarios

- [x] `MVP` Testar validacao de nomes.
- [x] `MVP` Testar geracao de nomes de secrets.
- [x] `MVP` Testar serializacao de app secret.
- [x] `MVP` Testar parsing de admin secret.
- [x] `MVP` Testar geracao de senha.
- [x] `MVP` Testar SQL/quote de identifiers.

### 13.2 Controller

- [ ] `MVP` Configurar testes com fake Kubernetes client.
- [ ] `MVP` Testar criacao de `DatabaseAccess`.
- [ ] `MVP` Testar status de erro sem `DatabaseInstance`.
- [ ] `MVP` Testar bloqueio por `allowedNamespaces`.
- [ ] `MVP` Testar caminho feliz com fakes de AWS e banco.

### 13.3 Integracao PostgreSQL

- [ ] `MVP` Subir PostgreSQL local para testes.
- [ ] `MVP` Testar criacao de database.
- [ ] `MVP` Testar criacao de schema.
- [ ] `MVP` Testar criacao de usuario readonly.
- [ ] `MVP` Testar criacao de usuario readwrite.
- [ ] `MVP` Testar reaplicacao idempotente.

### 13.4 E2E

- [x] `MVP` Criar `docker-compose.e2e.yaml`.
- [x] `MVP` Configurar servico PostgreSQL no Docker Compose.
- [x] `MVP` Configurar servico LocalStack no Docker Compose com Secrets Manager.
- [x] `MVP` Configurar variaveis AWS locais para apontar SDK para LocalStack.
- [x] `MVP` Criar script de bootstrap do admin secret no LocalStack.
- [ ] `MVP` Criar script para aplicar CRDs e samples no ambiente E2E.
- [x] `MVP` Rodar operator localmente contra PostgreSQL e LocalStack.
- [x] `MVP` Validar criacao do database no PostgreSQL.
- [x] `MVP` Validar criacao dos usuarios no PostgreSQL.
- [x] `MVP` Validar grants readonly e readwrite no PostgreSQL.
- [x] `MVP` Validar criacao dos secrets de aplicacao no LocalStack.
- [x] `MVP` Criar comando unico para executar E2E local.
- [ ] `Later` Criar teste com kind.
- [ ] `Later` Instalar operator no kind.
- [ ] `Later` Aplicar CRDs reais dentro do kind.

## 14. CI/CD

- [x] `MVP` Criar workflow de CI para pull requests.
- [x] `MVP` Rodar `cargo fmt --check`.
- [x] `MVP` Rodar `cargo clippy --all-targets --all-features`.
- [x] `MVP` Rodar `cargo test --all-features`.
- [x] `MVP` Rodar geracao de CRDs e verificar diff limpo.
- [x] `MVP` Rodar `helm lint`.
- [x] `MVP` Buildar imagem Docker.
- [x] `Later` Publicar imagem em GHCR.
- [ ] `Later` Publicar chart Helm.
- [ ] `Later` Gerar release semantica.

## 15. Documentacao

- [ ] `MVP` Atualizar README com descricao do projeto.
- [ ] `MVP` Documentar arquitetura.
- [ ] `MVP` Documentar instalacao via Helm.
- [ ] `MVP` Documentar criacao de `DatabaseInstance`.
- [ ] `MVP` Documentar criacao de `DatabaseAccess`.
- [ ] `MVP` Documentar formato do admin secret.
- [ ] `MVP` Documentar formato do app secret.
- [ ] `MVP` Documentar permissoes `readonly` e `readwrite`.
- [ ] `MVP` Documentar troubleshooting.
- [ ] `Later` Criar pagina de comparacao com Vault e RDS IAM Auth.
- [ ] `Later` Criar guia para GitOps com Argo CD.

## 16. Segurança e Hardening

- [ ] `MVP` Garantir que logs nao incluem secrets.
- [ ] `MVP` Garantir que status nao inclui senha.
- [ ] `MVP` Garantir que eventos nao incluem senha.
- [ ] `MVP` Restringir permissoes Kubernetes ao minimo necessario.
- [ ] `MVP` Restringir permissoes IAM ao minimo documentado.
- [ ] `MVP` Rodar container como nao-root.
- [ ] `MVP` Definir resource requests/limits no Helm chart.
- [ ] `Later` Adicionar NetworkPolicy.
- [ ] `Later` Adicionar PodSecurityContext mais restritivo.
- [ ] `Later` Adicionar assinatura de imagem.

## 17. Funcionalidades Futuras

- [ ] `Later` Suporte a MySQL.
- [ ] `Later` Suporte a Aurora MySQL.
- [ ] `Later` Suporte a RDS PostgreSQL standalone com configuracoes especificas.
- [ ] `Later` Usuario `migration`.
- [ ] `Later` Rotacao automatica de senhas.
- [ ] `Later` Rotacao manual por anotacao.
- [ ] `Later` Ownership de database por namespace.
- [ ] `Later` Policies reutilizaveis de grants.
- [ ] `Later` CRD `DatabaseGrantPolicy`.
- [ ] `Later` API HTTP opcional para integracoes externas.
- [ ] `Later` CLI administrativo.
- [ ] `Later` Dashboard web.
- [ ] `Later` Suporte multi-cloud.

## Ordem Recomendada de Execucao

1. Fundacao do repositorio.
2. API Kubernetes v1alpha1.
3. Controller skeleton.
4. Conditions, eventos e status.
5. AWS Secrets Manager.
6. Geracao de senhas.
7. PostgreSQL provisioner.
8. Integracao do reconciler.
9. Finalizers e delecao conservadora.
10. Empacotamento e Helm chart.
11. Testes e CI.
12. Documentacao de uso.

## Criterio de Pronto do MVP

- O operator instala em um cluster Kubernetes via Helm.
- Um `DatabaseInstance` define um cluster Aurora PostgreSQL alvo.
- Um `DatabaseAccess` cria database, schemas, usuarios e grants.
- Credenciais sao salvas no AWS Secrets Manager.
- O `status` do `DatabaseAccess` exibe os ARNs dos secrets.
- Reaplicar o mesmo manifest e seguro.
- Senhas nao aparecem em logs, eventos ou status.
- Testes unitarios, controller tests e testes de PostgreSQL passam.
- Documentacao explica o fluxo de ponta a ponta.
