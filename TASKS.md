# CloudVibe Database Operator - Tarefas

Este arquivo divide a implementacao do CloudVibe Database Operator em tarefas pequenas e verificaveis.

Legenda:

- `[ ]` pendente
- `[x]` concluido
- `MVP` necessario para primeira versao utilizavel
- `Later` pode ficar para depois do MVP

## 0. Fundacao do Repositorio

- [ ] `MVP` Inicializar projeto Rust com Cargo.
- [ ] `MVP` Definir package name e metadata do crate.
- [ ] `MVP` Adicionar dependencias base: `tokio`, `kube`, `k8s-openapi`, `serde`, `schemars`, `thiserror`, `tracing`, `axum`, `sqlx` e AWS SDK.
- [ ] `MVP` Gerar estrutura base com `src/api`, `src/controller`, `src/database`, `src/aws`, `src/http` e `src/telemetry`.
- [ ] `MVP` Configurar comandos de desenvolvimento via `Makefile` ou `justfile`.
- [ ] `MVP` Configurar `.gitignore` para binarios, builds e artefatos temporarios.
- [ ] `MVP` Criar `Dockerfile` do manager.
- [ ] `MVP` Criar README inicial com objetivo do projeto.
- [ ] `MVP` Garantir que `cargo test` roda sem falhas no projeto vazio.
- [ ] `MVP` Garantir que `cargo clippy` roda sem warnings relevantes.
- [ ] `MVP` Garantir que CRDs sao exportados a partir dos tipos Rust.

## 1. API Kubernetes v1alpha1

### 1.1 DatabaseInstance

- [ ] `MVP` Criar tipo `DatabaseInstance`.
- [ ] `MVP` Criar `DatabaseInstanceSpec`.
- [ ] `MVP` Adicionar campo `engine`.
- [ ] `MVP` Adicionar campo `region`.
- [ ] `MVP` Adicionar campo `host`.
- [ ] `MVP` Adicionar campo `port`.
- [ ] `MVP` Adicionar campo `adminSecretArn`.
- [ ] `MVP` Adicionar campo `secretPrefix`.
- [ ] `MVP` Adicionar campo `allowedNamespaces`.
- [ ] `MVP` Criar `DatabaseInstanceStatus` minimo.
- [ ] `MVP` Adicionar validacoes OpenAPI para campos obrigatorios.
- [ ] `MVP` Restringir `engine` inicialmente para `postgres`.
- [ ] `MVP` Gerar sample `config/samples/database_v1alpha1_databaseinstance.yaml`.

### 1.2 DatabaseAccess

- [ ] `MVP` Criar tipo `DatabaseAccess`.
- [ ] `MVP` Criar `DatabaseAccessSpec`.
- [ ] `MVP` Adicionar `instanceRef.name`.
- [ ] `MVP` Adicionar campo `database`.
- [ ] `MVP` Adicionar campo `schemas`.
- [ ] `MVP` Adicionar lista `users`.
- [ ] `MVP` Adicionar `users[].name`.
- [ ] `MVP` Adicionar `users[].permissions`.
- [ ] `MVP` Adicionar `users[].secretName` como opcional.
- [ ] `MVP` Restringir `permissions` para `readonly` e `readwrite`.
- [ ] `MVP` Criar `DatabaseAccessStatus`.
- [ ] `MVP` Adicionar `status.phase`.
- [ ] `MVP` Adicionar `status.observedGeneration`.
- [ ] `MVP` Adicionar `status.users[].secretArn`.
- [ ] `MVP` Adicionar `status.conditions`.
- [ ] `MVP` Adicionar colunas para `kubectl get databaseaccess`.
- [ ] `MVP` Gerar sample `config/samples/database_v1alpha1_databaseaccess.yaml`.

### 1.3 Validacao da API

- [ ] `MVP` Validar nomes de database com regex segura.
- [ ] `MVP` Validar nomes de schema com regex segura.
- [ ] `MVP` Validar nomes de usuario com regex segura.
- [ ] `MVP` Definir default de `schemas` como `public`.
- [ ] `Later` Implementar webhook de validacao para regras que OpenAPI nao cobre bem.

## 2. Controller Skeleton

- [ ] `MVP` Criar reconciler para `DatabaseAccess`.
- [ ] `MVP` Registrar reconciler no manager.
- [ ] `MVP` Configurar RBAC para ler `DatabaseInstance`.
- [ ] `MVP` Configurar RBAC para ler e atualizar `DatabaseAccess`.
- [ ] `MVP` Configurar RBAC para atualizar `DatabaseAccess/status`.
- [ ] `MVP` Configurar eventos Kubernetes.
- [ ] `MVP` Implementar lookup de `DatabaseInstance` por `instanceRef.name`.
- [ ] `MVP` Implementar validacao de `allowedNamespaces`.
- [ ] `MVP` Atualizar condition `Ready=False` quando `DatabaseInstance` nao existir.
- [ ] `MVP` Atualizar condition `Ready=False` quando namespace nao for permitido.
- [ ] `MVP` Atualizar `observedGeneration` apos reconciliacao.
- [ ] `MVP` Criar testes de reconciler com fake client para estados basicos.

## 3. Condicoes, Eventos e Status

- [ ] `MVP` Definir constantes de conditions.
- [ ] `MVP` Definir phases iniciais: `Pending`, `Ready`, `Error`.
- [ ] `MVP` Criar helper para setar `Ready=True`.
- [ ] `MVP` Criar helper para setar `Ready=False`.
- [ ] `MVP` Garantir que mensagens de erro nao vazam senha.
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
- [ ] `MVP` Criar client de Secrets Manager.
- [ ] `MVP` Implementar `GetSecretValue` para admin secret.
- [ ] `MVP` Implementar `DescribeSecret`.
- [ ] `MVP` Implementar `CreateSecret`.
- [ ] `MVP` Implementar `PutSecretValue`.
- [ ] `MVP` Implementar tags padrao nos secrets criados.
- [ ] `MVP` Tratar secret existente de forma idempotente.
- [ ] `MVP` Criar mocks/fakes para testes.

### 4.3 Nomes de Secrets

- [ ] `MVP` Implementar gerador de nome padrao `{secretPrefix}/{namespace}/{database}/{username}`.
- [ ] `MVP` Permitir override via `users[].secretName`.
- [ ] `MVP` Validar que o secret gerado nao sai do prefixo permitido.
- [ ] `Later` Permitir template de nomes por `DatabaseInstance`.

## 5. Geracao de Senhas

- [ ] `MVP` Criar pacote `internal/password`.
- [ ] `MVP` Gerar senha criptograficamente segura.
- [ ] `MVP` Definir tamanho minimo da senha.
- [ ] `MVP` Evitar caracteres problematicos para URL/connection string quando necessario.
- [ ] `MVP` Criar testes para tamanho e variedade.
- [ ] `MVP` Garantir que senha nunca aparece em logs.

## 6. PostgreSQL Provisioner

### 6.1 Contratos

- [ ] `MVP` Criar interface `DatabaseProvisioner`.
- [ ] `MVP` Criar modelo de entrada com instance, database, schemas e users.
- [ ] `MVP` Criar modelo de resultado por usuario.
- [ ] `MVP` Definir erros tipados para conflito, credencial invalida e permissao insuficiente.

### 6.2 Conexao

- [ ] `MVP` Adicionar `sqlx` com suporte a PostgreSQL e rustls.
- [ ] `MVP` Implementar conexao no database administrativo.
- [ ] `MVP` Implementar conexao no database alvo.
- [ ] `MVP` Configurar timeout de conexao.
- [ ] `MVP` Configurar TLS/SSL mode adequado para RDS.
- [ ] `Later` Suportar CA bundle customizado.

### 6.3 SQL Seguro

- [ ] `MVP` Implementar quote seguro de identifiers.
- [ ] `MVP` Validar identifiers antes de montar SQL.
- [ ] `MVP` Usar parametros para valores sempre que possivel.
- [ ] `MVP` Criar testes unitarios para quoting.

### 6.4 Database e Schemas

- [ ] `MVP` Verificar se database existe.
- [ ] `MVP` Criar database quando nao existir.
- [ ] `MVP` Reconectar no database alvo apos criacao.
- [ ] `MVP` Verificar se schema existe.
- [ ] `MVP` Criar schema quando nao existir.

### 6.5 Usuarios

- [ ] `MVP` Verificar se usuario existe.
- [ ] `MVP` Criar usuario com senha gerada quando nao existir.
- [ ] `MVP` Atualizar senha quando fluxo exigir criacao de novo secret.
- [ ] `MVP` Nao rotacionar senha automaticamente se secret ja existe.
- [ ] `MVP` Marcar erro claro quando usuario existe mas secret esperado nao existe.

### 6.6 Grants Readonly

- [ ] `MVP` Aplicar `GRANT CONNECT`.
- [ ] `MVP` Aplicar `GRANT USAGE ON SCHEMA`.
- [ ] `MVP` Aplicar `GRANT SELECT ON ALL TABLES`.
- [ ] `MVP` Aplicar default privileges para tabelas futuras.
- [ ] `MVP` Criar teste comprovando que readonly consegue ler.
- [ ] `MVP` Criar teste comprovando que readonly nao consegue escrever.

### 6.7 Grants Readwrite

- [ ] `MVP` Aplicar `GRANT CONNECT`.
- [ ] `MVP` Aplicar `GRANT USAGE, CREATE ON SCHEMA`.
- [ ] `MVP` Aplicar grants de tabelas.
- [ ] `MVP` Aplicar grants de sequencias.
- [ ] `MVP` Aplicar default privileges para tabelas futuras.
- [ ] `MVP` Aplicar default privileges para sequencias futuras.
- [ ] `MVP` Criar teste comprovando que readwrite consegue ler e escrever.

## 7. Integracao do Reconciler

- [ ] `MVP` Conectar reconciler ao `SecretStore`.
- [ ] `MVP` Conectar reconciler ao `DatabaseProvisioner`.
- [ ] `MVP` Ler admin secret durante reconciliacao.
- [ ] `MVP` Gerar senha apenas quando necessario.
- [ ] `MVP` Criar secret da aplicacao apos provisionar usuario.
- [ ] `MVP` Atualizar `status.users` com `secretArn`.
- [ ] `MVP` Reaplicar grants em reconciliacoes repetidas.
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

- [ ] `MVP` Criar build multi-stage no Dockerfile.
- [ ] `MVP` Rodar manager como usuario nao-root.
- [ ] `MVP` Publicar imagem com tag versionada.
- [ ] `MVP` Documentar variaveis de ambiente do manager.

### 10.2 Manifests

- [ ] `MVP` Gerar manifests `config/default`.
- [ ] `MVP` Validar RBAC gerado.
- [ ] `MVP` Validar leader election.
- [ ] `MVP` Validar namespace de instalacao.

### 10.3 Helm Chart

- [ ] `MVP` Criar chart `charts/cloudvibe-database-operator`.
- [ ] `MVP` Templatear deployment do manager.
- [ ] `MVP` Templatear service account.
- [ ] `MVP` Templatear RBAC.
- [ ] `MVP` Templatear CRDs ou documentar instalacao separada.
- [ ] `MVP` Suportar annotation de IRSA/EKS Pod Identity no service account.
- [ ] `MVP` Criar `values.yaml` padrao.
- [ ] `MVP` Rodar `helm lint`.

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

- [ ] `MVP` Testar validacao de nomes.
- [ ] `MVP` Testar geracao de nomes de secrets.
- [ ] `MVP` Testar serializacao de app secret.
- [ ] `MVP` Testar parsing de admin secret.
- [ ] `MVP` Testar geracao de senha.
- [ ] `MVP` Testar SQL/quote de identifiers.

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

- [ ] `Later` Criar teste com kind.
- [ ] `Later` Instalar operator no kind.
- [ ] `Later` Aplicar CRDs reais.
- [ ] `Later` Validar reconciliacao completa com fakes ou LocalStack.

## 14. CI/CD

- [ ] `MVP` Criar workflow de CI para pull requests.
- [ ] `MVP` Rodar `cargo fmt --check`.
- [ ] `MVP` Rodar `cargo clippy --all-targets --all-features`.
- [ ] `MVP` Rodar `cargo test --all-features`.
- [ ] `MVP` Rodar geracao de CRDs e verificar diff limpo.
- [ ] `MVP` Rodar `helm lint`.
- [ ] `MVP` Buildar imagem Docker.
- [ ] `Later` Publicar imagem em GHCR.
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
- [ ] `Later` Suporte a Aurora PostgreSQL com configuracoes especificas.
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
- Um `DatabaseInstance` define um RDS PostgreSQL alvo.
- Um `DatabaseAccess` cria database, schemas, usuarios e grants.
- Credenciais sao salvas no AWS Secrets Manager.
- O `status` do `DatabaseAccess` exibe os ARNs dos secrets.
- Reaplicar o mesmo manifest e seguro.
- Senhas nao aparecem em logs, eventos ou status.
- Testes unitarios, controller tests e testes de PostgreSQL passam.
- Documentacao explica o fluxo de ponta a ponta.
