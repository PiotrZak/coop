# Prefer iam_member over iam_binding so we do not clobber other role members.
# No google_service_account_key here — create / rotate keys out-of-band and store
# only in GitHub Actions secret GCP_SA_KEY (never in git or tfvars).

resource "google_project_iam_member" "cloudrun_deployer" {
  project = var.project_id
  role    = "roles/run.developer"
  member  = "serviceAccount:${var.github_sa_email}"
}

resource "google_project_iam_member" "cloudrun_sa_user" {
  project = var.project_id
  role    = "roles/iam.serviceAccountUser"
  member  = "serviceAccount:${var.github_sa_email}"
}

resource "google_project_iam_member" "artifact_registry_writer" {
  project = var.project_id
  role    = "roles/artifactregistry.writer"
  member  = "serviceAccount:${var.github_sa_email}"
}
