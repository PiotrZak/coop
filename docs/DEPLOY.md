# Deploy Coop (GCP Cloud Run)

Shape mirrors Lindle’s Artifact Registry + Cloud Run + GitHub Actions flow.
**Do not copy Lindle Action secrets** — create Coop’s own GCP project (or SA) and secrets.

## 1. GCP (once)

1. Create a GCP project (or reuse a non-Lindle project).
2. Create deployer SA, e.g. `github-actions-deployer`, with roles:
   - `roles/run.developer`
   - `roles/iam.serviceAccountUser`
   - `roles/artifactregistry.writer`  
   Or apply `Terraform/` after filling `terraform.tfvars` from the example.
3. Create a JSON key for that SA **out of band** (or prefer Workload Identity Federation later).

## 2. GitHub repo settings

**Variables** (Settings → Variables):

| Name | Example |
|------|---------|
| `GCP_PROJECT_ID` | your project id |
| `GAR_LOCATION` | `europe-central2` |
| `GCP_REGION` | `europe-central2` |

**Secrets** (Settings → Secrets):

| Name | Purpose |
|------|---------|
| `GCP_SA_KEY` | Deployer SA JSON (required for deploy workflow) |

## 3. Terraform

```bash
cd Terraform
cp terraform.tfvars.example terraform.tfvars   # edit — gitignored
terraform init
terraform plan
terraform apply
```

## 4. CI / CD

- `ci.yml` — fmt, clippy, test on every PR / push to `main`.
- `deploy.yml` — builds Docker image, pushes to Artifact Registry, deploys Cloud Run when `GCP_PROJECT_ID` is set and `GCP_SA_KEY` is present.

## 5. Local

```bash
cargo run -p coop-a2a --bin coop-supplier
cargo run -p coop-a2a --bin coop-negotiation
cargo run -p coop-a2a --bin coop-gateway
cargo run -p coop-a2a --bin coop-demo -- --peer-url http://127.0.0.1:7090
```

Lindle orchestration needs only `AGENTS_SERVICE_URL` + `AGENTS_INTERNAL_SECRET`.
