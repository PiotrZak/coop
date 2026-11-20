output "artifact_registry" {
  value = google_artifact_registry_repository.coop.name
}

output "cloud_run_uri" {
  value = google_cloud_run_v2_service.coop.uri
}

output "image_prefix" {
  value = "${var.gar_location}-docker.pkg.dev/${var.project_id}/${var.service_name}"
}
