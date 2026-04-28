# CloudVibe Database Operator - Plano de Implementacao

## Objetivo

Criar um Kubernetes Operator para provisionar databases, usuarios, permissoes e credenciais de aplicacoes de forma declarativa, usando Aurora PostgreSQL, recursos Kubernetes e AWS Secrets Manager.

O operador deve permitir que um time de aplicacao declare algo como:

```yaml
apiVersion: database.cloudvibe.dev/v1alpha1
kind: DatabaseAccess
metadata:
  name: orders-db
  namespace: orders
spec:
  instanceRef:
    name: prod-main-postgres
  database: orders
  schemas:
    - public
  users:
    - name: orders_api_rw
      permissions: readwrite
    - name: orders_api_ro
      permissions: readonly
```

A partir dessa declaracao, o controller deve:

1. Localizar o cluster Aurora PostgreSQL configurado.
2. Ler a credencial administrativa no AWS Secrets Manager.
3. Criar o database caso nao exista.
4. Criar ou atualizar usuarios por aplicacao.
5. Aplicar permissoes padronizadas.
6. Gerar senhas fortes.
7. Salvar as credenciais finais no AWS Secrets Manager.
8. Atualizar o `status` do recurso com os ARNs dos secrets criados.

O objetivo principal e manter a aplicacao simples: ela nao precisa saber criar usuario, aplicar grants, gerar senha, chamar AWS SDK nem lidar com credencial administrativa do banco.

## Nome do Projeto

- Nome de mercado: CloudVibe Database Operator
- Nome do repositorio: `cloudvibe-database-operator`
- API group: `database.cloudvibe.dev`
- Primeira versao da API: `v1alpha1`

## Principios

- Declarativo: o estado desejado vem de manifests Kubernetes.
- Idempotente: aplicar o mesmo recurso varias vezes deve ser seguro.
- Seguro por padrao: nunca registrar senhas em logs, eventos ou status.
- Menor privilegio: aplicacoes so acessam os secrets delas.
- Separacao de responsabilidades: plataforma define instancias; aplicacoes declaram acessos.
- Evolutivo: comecar com Aurora PostgreSQL e manter compatibilidade natural com RDS PostgreSQL.
- GitOps-friendly: funcionar bem com Helm, Argo CD e `kubectl apply`.

## Arquitetura Proposta

```text
Helm / kubectl apply
        |
        v
Kubernetes API
        |
        v
DatabaseAccess CR
        |
        v
CloudVibe Database Operator
        |
        +--> le DatabaseInstance
        +--> le admin secret no AWS Secrets Manager
        +--> conecta no Aurora PostgreSQL
        +--> cria database, schemas, roles e grants
        +--> salva credenciais no AWS Secrets Manager
        +--> atualiza status do DatabaseAccess
```

Componentes internos:

```text
src/main.rs
src/api/v1alpha1
src/controller
src/database/postgres
src/aws/secretsmanager
src/http
src/telemetry
src/password
src/naming
deploy/crds
deploy/rbac
deploy/samples
```

## CRDs

### DatabaseInstance

Recurso administrado pelo time de plataforma. Representa uma instancia de banco disponivel para uso.

Exemplo:

```yaml
apiVersion: database.cloudvibe.dev/v1alpha1
kind: DatabaseInstance
metadata:
  name: prod-main-postgres
spec:
  engine: aurora-postgres
  region: us-east-1
  host: prod-main.cluster-xxxxxx.us-east-1.rds.amazonaws.com
  port: 5432
  adminSecretArn: arn:aws:secretsmanager:us-east-1:123456789012:secret:rds/prod-main/admin-AbCdEf
  secretPrefix: rds/prod-main
  allowedNamespaces:
    - orders
    - billing
```

Campos iniciais:

- `spec.engine`: inicialmente `aurora-postgres`.
- `spec.region`: regiao AWS do Aurora e Secrets Manager.
- `spec.host`: endpoint writer do cluster Aurora PostgreSQL.
- `spec.port`: porta do banco.
- `spec.adminSecretArn`: secret com credencial administrativa.
- `spec.secretPrefix`: prefixo usado para criar secrets das aplicacoes.
- `spec.allowedNamespaces`: namespaces autorizados a usar essa instancia.

Formato esperado do admin secret:

```json
{
  "username": "postgres",
  "password": "senha-admin",
  "database": "postgres"
}
```

### DatabaseAccess

Recurso criado pelo time da aplicacao. Representa o desejo de ter database e usuarios.

Exemplo:

```yaml
apiVersion: database.cloudvibe.dev/v1alpha1
kind: DatabaseAccess
metadata:
  name: orders-db
  namespace: orders
spec:
  instanceRef:
    name: prod-main-postgres
  database: orders
  schemas:
    - public
  users:
    - name: orders_api_rw
      permissions: readwrite
    - name: orders_api_ro
      permissions: readonly
```

Campos iniciais:

- `spec.instanceRef.name`: nome do `DatabaseInstance`.
- `spec.database`: database que deve existir.
- `spec.schemas`: lista de schemas gerenciados, inicialmente default `public`.
- `spec.users[].name`: usuario a ser criado.
- `spec.users[].permissions`: `readonly` ou `readwrite`.
- `spec.users[].secretName`: opcional; se ausente, o operador gera o nome.

Status esperado:

```yaml
status:
  observedGeneration: 1
  phase: Ready
  database: orders
  users:
    - name: orders_api_rw
      permissions: readwrite
      secretArn: arn:aws:secretsmanager:us-east-1:123456789012:secret:rds/prod-main/orders/orders_api_rw-AbCdEf
    - name: orders_api_ro
      permissions: readonly
      secretArn: arn:aws:secretsmanager:us-east-1:123456789012:secret:rds/prod-main/orders/orders_api_ro-XyZ123
  conditions:
    - type: Ready
      status: "True"
      reason: Provisioned
      message: Database and users are ready
```

## Permissoes PostgreSQL

### Usuario readonly

Permissoes:

- Conectar no database.
- Usar schemas configurados.
- Ler tabelas existentes.
- Ler tabelas criadas no futuro via default privileges.

SQL conceitual:

```sql
GRANT CONNECT ON DATABASE "{database}" TO "{user}";
GRANT USAGE ON SCHEMA "{schema}" TO "{user}";
GRANT SELECT ON ALL TABLES IN SCHEMA "{schema}" TO "{user}";
ALTER DEFAULT PRIVILEGES IN SCHEMA "{schema}"
GRANT SELECT ON TABLES TO "{user}";
```

### Usuario readwrite

Permissoes:

- Conectar no database.
- Usar schemas configurados.
- Criar objetos no schema, se configurado.
- Ler, inserir, atualizar e apagar dados.
- Usar sequencias.
- Receber permissoes futuras via default privileges.

SQL conceitual:

```sql
GRANT CONNECT ON DATABASE "{database}" TO "{user}";
GRANT USAGE, CREATE ON SCHEMA "{schema}" TO "{user}";
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA "{schema}" TO "{user}";
GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA "{schema}" TO "{user}";
ALTER DEFAULT PRIVILEGES IN SCHEMA "{schema}"
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO "{user}";
ALTER DEFAULT PRIVILEGES IN SCHEMA "{schema}"
GRANT USAGE, SELECT, UPDATE ON SEQUENCES TO "{user}";
```

## Formato dos Secrets Criados

Nome padrao:

```text
{secretPrefix}/{namespace}/{database}/{username}
```

Exemplo:

```text
rds/prod-main/orders/orders/orders_api_rw
```

Conteudo:

```json
{
  "engine": "aurora-postgres",
  "host": "prod-main.cluster-xxxxxx.us-east-1.rds.amazonaws.com",
  "port": 5432,
  "database": "orders",
  "username": "orders_api_rw",
  "password": "senha-gerada",
  "jdbcUrl": "jdbc:postgresql://prod-main.cluster-xxxxxx.us-east-1.rds.amazonaws.com:5432/orders",
  "uri": "postgresql://orders_api_rw:senha-gerada@prod-main.cluster-xxxxxx.us-east-1.rds.amazonaws.com:5432/orders"
}
```

Observacao: o operador nunca deve gravar `password` no `status`, em eventos Kubernetes ou em logs.

## Fluxo de Reconciliacao

Para cada `DatabaseAccess`:

1. Validar `spec`.
2. Buscar `DatabaseInstance` referenciado.
3. Validar se o namespace do `DatabaseAccess` esta permitido.
4. Ler secret administrativo no AWS Secrets Manager.
5. Conectar no banco administrativo.
6. Criar database se nao existir.
7. Conectar no database alvo.
8. Garantir schemas.
9. Para cada usuario:
   - Validar nome.
   - Verificar se secret da aplicacao ja existe.
   - Se nao existir, gerar senha forte.
   - Criar usuario se nao existir.
   - Se necessario, atualizar senha.
   - Aplicar grants conforme `permissions`.
   - Criar ou atualizar secret no AWS Secrets Manager.
10. Atualizar `status.users` com ARNs.
11. Marcar condition `Ready=True`.

Em caso de erro:

- Marcar `Ready=False`.
- Preencher `reason` e `message` sem dados sensiveis.
- Reenfileirar reconciliacao com backoff.

## Idempotencia

O operador deve tolerar:

- Database ja existente.
- Usuario ja existente.
- Grants ja aplicados.
- Secret ja existente.
- Reconciliacoes repetidas.
- Falha parcial entre criar usuario e criar secret.

Regras iniciais:

- Se o secret ja existe, nao gerar nova senha automaticamente.
- Se o usuario existe e o secret existe, reaplicar grants.
- Se o usuario existe mas o secret nao existe, marcar erro e exigir acao manual ou implementar recuperacao explicita.
- Rotacao de senha deve ser um fluxo separado.

## Seguranca

### IAM do Operator

A role do operator precisa:

- Ler o admin secret das instancias autorizadas.
- Criar, ler e atualizar secrets de aplicacao dentro de prefixos permitidos.
- Opcionalmente aplicar tags nos secrets.

Permissoes conceituais:

```json
{
  "Effect": "Allow",
  "Action": [
    "secretsmanager:GetSecretValue",
    "secretsmanager:CreateSecret",
    "secretsmanager:PutSecretValue",
    "secretsmanager:DescribeSecret",
    "secretsmanager:TagResource"
  ],
  "Resource": "*"
}
```

Na implementacao real, restringir `Resource` por ARN/prefixo.

### Kubernetes RBAC

O operator precisa:

- Ler e observar `DatabaseInstance`.
- Ler, observar e atualizar status de `DatabaseAccess`.
- Criar eventos Kubernetes.
- Usar leader election.

### Validacoes

Validar nomes de database, schema e usuario com allowlist:

```text
^[a-zA-Z_][a-zA-Z0-9_]{0,62}$
```

Nao montar SQL por concatenacao livre. Usar funcoes de quote identifier e parametros onde aplicavel.

## Delecao e Finalizers

Na primeira versao, a delecao deve ser conservadora.

Comportamento proposto para v1alpha1:

- Adicionar finalizer em `DatabaseAccess`.
- Ao deletar, por padrao nao apagar database nem usuario automaticamente.
- Marcar no status/evento que os recursos externos permanecem.
- Futuramente suportar `spec.deletionPolicy`.

Politicas futuras:

```yaml
spec:
  deletionPolicy: Retain
```

Valores possiveis:

- `Retain`: nao remove nada externo.
- `RevokeUsers`: revoga/remove usuarios criados.
- `DropDatabase`: remove database, apenas para ambientes controlados.

## Rotacao de Senhas

Nao precisa entrar na primeira entrega, mas o design deve prever.

Opcoes futuras:

- Campo `spec.users[].rotation.enabled`.
- Campo `spec.users[].rotation.interval`.
- Anotacao manual:

```yaml
metadata:
  annotations:
    database.cloudvibe.dev/rotate-at: "2026-04-28T10:00:00Z"
```

Fluxo de rotacao:

1. Gerar nova senha.
2. Alterar senha do usuario no banco.
3. Criar nova versao no AWS Secrets Manager.
4. Atualizar status com data da ultima rotacao.

## Entrega para Aplicacoes

O operator deve salvar os secrets no AWS Secrets Manager. A entrega para o pod pode ser feita por:

1. External Secrets Operator, recomendado para Kubernetes.
2. AWS Secrets Manager CSI Driver.
3. Aplicacao lendo diretamente o secret por ARN, quando fizer sentido.

Exemplo com External Secrets Operator:

```yaml
apiVersion: external-secrets.io/v1
kind: ExternalSecret
metadata:
  name: orders-db
  namespace: orders
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secretsmanager
    kind: ClusterSecretStore
  target:
    name: orders-db
  dataFrom:
    - extract:
        key: rds/prod-main/orders/orders/orders_api_rw
```

## Stack Tecnica

Recomendacao inicial:

- Linguagem: Rust
- Runtime async: Tokio
- Kubernetes controller: kube-rs
- HTTP auxiliar: Axum
- OpenTelemetry: tracing, tracing-opentelemetry, opentelemetry-otlp
- Banco inicial: Aurora PostgreSQL
- AWS SDK: AWS SDK for Rust
- Driver PostgreSQL: `sqlx`
- Testes Kubernetes: testes unitarios com fakes e testes e2e com kind futuramente
- Testes E2E locais: Docker Compose com PostgreSQL e LocalStack
- Testes de banco: Docker Compose com PostgreSQL real
- Testes AWS: LocalStack simulando AWS Secrets Manager
- Build de imagem: Docker
- Deploy: Helm chart e manifests Kubernetes

O Axum nao deve ser a interface principal de provisionamento no MVP. A interface principal e a API Kubernetes via CRDs. O Axum deve expor endpoints operacionais:

```text
GET /healthz
GET /readyz
GET /metrics
```

O servidor Axum deve ser instrumentado com OpenTelemetry para traces, logs correlacionados e metricas. Reconciliacoes do controller, chamadas ao AWS Secrets Manager e operacoes PostgreSQL tambem devem criar spans.

O ambiente de testes E2E local deve usar Docker Compose. A composicao minima deve subir:

```text
postgres
localstack
cloudvibe-database-operator
```

O LocalStack deve expor o Secrets Manager para validar leitura do admin secret e criacao dos secrets de aplicacao sem depender de AWS real. O PostgreSQL deve validar database, schema, usuario e grants com permissoes reais.

## Estrutura Inicial do Repositorio

```text
.
├── src/
│   ├── main.rs
│   ├── api/
│   │   └── v1alpha1/
│   ├── aws/
│   │   └── secretsmanager.rs
│   ├── controller/
│   ├── database/
│   │   └── postgres/
│   ├── http/
│   ├── naming.rs
│   ├── password.rs
│   └── telemetry.rs
├── deploy/
│   ├── crds/
│   ├── rbac/
│   └── samples/
├── charts/
│   └── cloudvibe-database-operator/
├── docs/
├── tests/
├── Cargo.toml
├── Dockerfile
└── PLAN.md
```

## Roadmap

### Fase 0 - Fundacao

Objetivo: criar esqueleto do projeto.

Tarefas:

- Inicializar projeto Rust.
- Definir crate e metadata do Cargo.
- Adicionar dependencias base: `tokio`, `kube`, `k8s-openapi`, `serde`, `schemars`, `thiserror`, `tracing`, `axum`, `sqlx` e AWS SDK.
- Criar tipos `DatabaseInstance` e `DatabaseAccess`.
- Gerar/exportar CRDs a partir dos tipos Rust.
- Criar manifests de RBAC.
- Criar samples basicos.
- Configurar lint/test/build.

Criterio de pronto:

- `cargo test` executa com sucesso.
- `cargo clippy` executa sem warnings relevantes.
- CRDs sao gerados.
- Manager sobe sem reconciliar recursos reais.

### Fase 1 - API v1alpha1

Objetivo: estabilizar o contrato declarativo inicial.

Tarefas:

- Modelar `DatabaseInstanceSpec`.
- Modelar `DatabaseAccessSpec`.
- Modelar `DatabaseAccessStatus`.
- Adicionar validacoes OpenAPI no CRD.
- Definir conditions padrao.
- Documentar exemplos.

Criterio de pronto:

- Manifests invalidos sao rejeitados pelo Kubernetes quando possivel.
- `kubectl get databaseaccess` mostra colunas uteis.

### Fase 2 - AWS Secrets Manager

Objetivo: integrar leitura e escrita de secrets.

Tarefas:

- Implementar client de Secrets Manager.
- Ler admin secret.
- Criar secret de aplicacao.
- Atualizar secret existente sem vazar senha.
- Aplicar tags.
- Criar testes unitarios com interface mockavel.

Criterio de pronto:

- O core consegue criar/atualizar secrets com contrato definido.
- Erros AWS sao propagados para conditions.

### Fase 3 - Aurora PostgreSQL Provisioner

Objetivo: criar database, usuarios e grants.

Tarefas:

- Implementar conexao administrativa.
- Criar database se nao existir.
- Conectar ao database alvo.
- Criar schemas.
- Criar usuarios.
- Aplicar grants `readonly`.
- Aplicar grants `readwrite`.
- Validar quoting seguro de identifiers.
- Criar testes com PostgreSQL real em container.

Criterio de pronto:

- Um teste cria database e usuarios reais em PostgreSQL.
- Usuarios readonly nao conseguem escrever.
- Usuarios readwrite conseguem ler e escrever.

### Fase 3.5 - E2E Local com Docker Compose

Objetivo: validar o fluxo completo sem AWS real.

Tarefas:

- Criar `docker-compose.e2e.yaml`.
- Subir PostgreSQL real.
- Subir LocalStack com Secrets Manager.
- Criar script de bootstrap do admin secret no LocalStack.
- Rodar o operator apontando para PostgreSQL e LocalStack.
- Aplicar samples de `DatabaseInstance` e `DatabaseAccess`.
- Validar que o secret de aplicacao foi criado no LocalStack.
- Validar que usuarios conseguem acessar o PostgreSQL com as permissoes esperadas.

Criterio de pronto:

- Um comando local executa o fluxo completo de ponta a ponta.
- O teste prova criacao de database, usuarios, grants e secrets.
- O fluxo nao exige credenciais AWS reais.

### Fase 4 - Reconciler DatabaseAccess

Objetivo: unir Kubernetes, AWS e PostgreSQL.

Tarefas:

- Observar `DatabaseAccess`.
- Buscar `DatabaseInstance`.
- Validar namespace permitido.
- Reconciliar estado externo.
- Atualizar `status`.
- Emitir eventos.
- Implementar backoff e tratamento de erro.

Criterio de pronto:

- Aplicar um `DatabaseAccess` cria recursos externos.
- `status.phase` vira `Ready`.
- Reaplicar o mesmo manifest nao quebra nem recria senha.

### Fase 5 - Empacotamento

Objetivo: instalar o operator em cluster.

Tarefas:

- Criar imagem Docker.
- Criar Helm chart.
- Configurar ServiceAccount.
- Documentar IRSA/EKS Pod Identity.
- Documentar permissoes IAM minimas.
- Criar guia de instalacao.

Criterio de pronto:

- Operator instala via Helm.
- Operator roda em EKS usando IAM role.

### Fase 6 - Operacao

Objetivo: deixar o projeto usavel por times.

Tarefas:

- Criar docs de uso por time de plataforma.
- Criar docs de uso por time de aplicacao.
- Criar troubleshooting.
- Criar exemplos com External Secrets Operator.
- Adicionar metricas basicas.
- Adicionar logs estruturados.

Criterio de pronto:

- Um time consegue criar um database e receber secret seguindo a documentacao.

## MVP

Escopo minimo:

- `DatabaseInstance`
- `DatabaseAccess`
- PostgreSQL
- AWS Secrets Manager
- LocalStack para E2E de Secrets Manager
- Docker Compose para E2E local
- Permissoes `readonly` e `readwrite`
- Database e schema `public`
- Status com ARNs
- Helm chart para instalar o operator

## Riscos e Decisoes Importantes

- Criar database automaticamente e util, mas sensivel em producao; validar `allowedNamespaces` e evoluir para policies.
- Deletar database por acidente e perigoso; a politica inicial deve ser `Retain`.
- Usuario existente sem secret e estado ambiguo; falhar claramente e exigir intervencao manual.
- Default privileges no PostgreSQL dependem de quem cria objetos; migrations podem exigir usuario proprio no futuro.
- Dois `DatabaseAccess` podem disputar o mesmo database ou usuario; validar nomes e tratar conflitos.
## Primeira Entrega Recomendada

1. Inicializar projeto Rust com kube-rs, Axum, sqlx e OpenTelemetry.
2. Criar CRDs `DatabaseInstance` e `DatabaseAccess`.
3. Implementar reconciler com validacao e status falso, sem tocar AWS/banco.
4. Implementar AWS Secrets Manager client.
5. Implementar PostgreSQL provisioner com testes.
6. Conectar reconciler ao provisioner.
7. Empacotar imagem e Helm chart.
8. Testar em um cluster sandbox contra um Aurora PostgreSQL real.
