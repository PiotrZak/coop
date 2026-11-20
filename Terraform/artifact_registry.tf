resource "google_artifact_registry_repository" "coop" {
  project       = var.project_id
  location      = var.gar_location
  repository_id = var.service_name
  format        = "DOCKER"
  description   = "Coop A2A edge images"
}
